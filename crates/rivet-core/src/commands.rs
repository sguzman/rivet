use std::collections::BTreeSet;
use std::io::{self, Read};

use anyhow::{Context, anyhow};
use chrono::Utc;
use tracing::{debug, info, instrument, warn};

use crate::cli::Invocation;
use crate::config::Config;
use crate::datastore::DataStore;
use crate::datetime::parse_date_expr;
use crate::filter::Filter;
use crate::render::Renderer;
use crate::task::{Annotation, Status, Task};

pub fn known_command_names() -> Vec<&'static str> {
    vec![
        "add",
        "append",
        "prepend",
        "list",
        "next",
        "info",
        "modify",
        "start",
        "stop",
        "annotate",
        "denotate",
        "duplicate",
        "log",
        "done",
        "delete",
        "undo",
        "export",
        "import",
        "projects",
        "tags",
        "context",
        "contexts",
        "_commands",
        "_show",
        "_unique",
        "help",
        "version",
    ]
}

pub fn expand_command_abbrev<'a>(token: &'a str, known: &[&'a str]) -> Option<&'a str> {
    if known.contains(&token) {
        return Some(token);
    }

    let mut matches = known.iter().copied().filter(|name| name.starts_with(token));
    let first = matches.next()?;
    if matches.next().is_some() {
        None
    } else {
        Some(first)
    }
}

#[instrument(skip(store, cfg, renderer, inv))]
pub fn dispatch(
    store: &mut DataStore,
    cfg: &Config,
    renderer: &mut Renderer,
    inv: Invocation,
) -> anyhow::Result<()> {
    let now = Utc::now();
    let command = inv.command.as_str();
    let effective_filters = resolve_effective_filter_terms(store, cfg, command, &inv.filter_terms)?;

    debug!(
        command,
        filter = ?inv.filter_terms,
        args = ?inv.command_args,
        "dispatching command"
    );

    match command {
        "add" => cmd_add(store, cfg, renderer, &inv.command_args, now),
        "append" => cmd_append(store, &effective_filters, &inv.command_args, now),
        "prepend" => cmd_prepend(store, &effective_filters, &inv.command_args, now),
        "list" | "next" => cmd_list(store, cfg, renderer, &effective_filters, now),
        "info" => cmd_info(store, renderer, &effective_filters, now),
        "modify" => cmd_modify(store, &effective_filters, &inv.command_args, now),
        "start" => cmd_start(store, &effective_filters, now),
        "stop" => cmd_stop(store, &effective_filters, now),
        "annotate" => cmd_annotate(store, &effective_filters, &inv.command_args, now),
        "denotate" => cmd_denotate(store, &effective_filters, &inv.command_args, now),
        "duplicate" => cmd_duplicate(store, &effective_filters, now),
        "log" => cmd_log(store, &inv.command_args, now),
        "done" => cmd_done(store, &effective_filters, now),
        "delete" => cmd_delete(store, &effective_filters, now),
        "undo" => cmd_undo(store),
        "export" => cmd_export(store, &effective_filters, now),
        "import" => cmd_import(store),
        "projects" => cmd_projects(store),
        "tags" => cmd_tags(store),
        "context" | "contexts" => cmd_context(store, cfg, &inv.command_args),
        "_commands" => cmd_commands(),
        "_show" => cmd_show(cfg),
        "_unique" => cmd_unique(store, &inv.command_args),
        "help" => cmd_help(),
        "version" => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        other => Err(anyhow!("unknown command: {other}")),
    }
}

#[instrument(skip(store, _cfg, _renderer, args, now))]
fn cmd_add(
    store: &mut DataStore,
    _cfg: &Config,
    _renderer: &mut Renderer,
    args: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!("command add");

    let mut pending = store.load_pending()?;
    let completed = store.load_completed()?;
    let pending_before = pending.clone();

    let next_id = store.next_id(&pending);
    let (description, mods) = parse_desc_and_mods(args, now)?;
    let mut task = Task::new_pending(description, now, next_id);
    apply_mods(&mut task, &mods, now)?;

    pending = store.add_task(pending, task.clone())?;
    store.push_undo_snapshot(&pending_before, &completed)?;

    debug!(pending_count = pending.len(), "task added");
    println!("Created task {}.", task.id.unwrap_or(next_id));
    Ok(())
}

#[instrument(skip(store, filter_terms, args, now))]
fn cmd_append(
    store: &mut DataStore,
    filter_terms: &[String],
    args: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!("command append");

    if args.is_empty() {
        return Err(anyhow!("append requires text argument"));
    }
    let suffix = args.join(" ");

    let filter = Filter::parse(filter_terms, now)?;
    let mut pending = store.load_pending()?;
    let mut completed = store.load_completed()?;
    let pending_before = pending.clone();
    let completed_before = completed.clone();
    let include_non_pending = filter.has_explicit_status_filter() || filter.has_identity_selector();

    let mut changed = 0_u64;
    for task in &mut pending {
        if !include_non_pending && task.status != Status::Pending && task.status != Status::Waiting
        {
            continue;
        }
        if filter.matches(task, now) {
            task.description = format!("{} {}", task.description, suffix)
                .trim()
                .to_string();
            task.modified = now;
            changed += 1;
        }
    }

    if include_non_pending {
        for task in &mut completed {
            if filter.matches(task, now) {
                task.description = format!("{} {}", task.description, suffix)
                    .trim()
                    .to_string();
                task.modified = now;
                changed += 1;
            }
        }
    }

    if changed > 0 {
        store.push_undo_snapshot(&pending_before, &completed_before)?;
        store.save_pending(&pending)?;
        if include_non_pending {
            store.save_completed(&completed)?;
        }
    }

    println!("Modified {changed} task(s).");
    Ok(())
}

#[instrument(skip(store, filter_terms, args, now))]
fn cmd_prepend(
    store: &mut DataStore,
    filter_terms: &[String],
    args: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!("command prepend");

    if args.is_empty() {
        return Err(anyhow!("prepend requires text argument"));
    }
    let prefix = args.join(" ");

    let filter = Filter::parse(filter_terms, now)?;
    let mut pending = store.load_pending()?;
    let mut completed = store.load_completed()?;
    let pending_before = pending.clone();
    let completed_before = completed.clone();
    let include_non_pending = filter.has_explicit_status_filter() || filter.has_identity_selector();

    let mut changed = 0_u64;
    for task in &mut pending {
        if !include_non_pending && task.status != Status::Pending && task.status != Status::Waiting
        {
            continue;
        }
        if filter.matches(task, now) {
            task.description = format!("{} {}", prefix, task.description)
                .trim()
                .to_string();
            task.modified = now;
            changed += 1;
        }
    }

    if include_non_pending {
        for task in &mut completed {
            if filter.matches(task, now) {
                task.description = format!("{} {}", prefix, task.description)
                    .trim()
                    .to_string();
                task.modified = now;
                changed += 1;
            }
        }
    }

    if changed > 0 {
        store.push_undo_snapshot(&pending_before, &completed_before)?;
        store.save_pending(&pending)?;
        if include_non_pending {
            store.save_completed(&completed)?;
        }
    }

    println!("Modified {changed} task(s).");
    Ok(())
}

#[instrument(skip(store, _cfg, renderer, filter_terms, now))]
fn cmd_list(
    store: &mut DataStore,
    _cfg: &Config,
    renderer: &mut Renderer,
    filter_terms: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!("command list/next");

    let mut pending = store.load_pending()?;
    pending.retain(|task| task.status == Status::Pending || task.status == Status::Waiting);

    let filter = Filter::parse(filter_terms, now)?;
    let mut rows: Vec<Task> = pending
        .into_iter()
        .filter(|task| filter.matches(task, now))
        .collect();

    rows.sort_by_key(|task| (task.due, task.id));
    renderer.print_task_table(&rows, now)?;
    Ok(())
}

#[instrument(skip(store, renderer, filter_terms, now))]
fn cmd_info(
    store: &mut DataStore,
    renderer: &mut Renderer,
    filter_terms: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!("command info");

    let pending = store.load_pending()?;
    let completed = store.load_completed()?;
    let filter = Filter::parse(filter_terms, now)?;

    let mut rows: Vec<Task> = pending
        .into_iter()
        .chain(completed)
        .filter(|task| filter.matches(task, now))
        .collect();

    rows.sort_by_key(|task| task.id.unwrap_or(u64::MAX));

    if rows.is_empty() {
        return Err(anyhow!("no matching tasks"));
    }

    for task in rows {
        renderer.print_task_info(&task)?;
        println!();
    }

    Ok(())
}

#[instrument(skip(store, filter_terms, args, now))]
fn cmd_modify(
    store: &mut DataStore,
    filter_terms: &[String],
    args: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!("command modify");

    let mut pending = store.load_pending()?;
    let mut completed = store.load_completed()?;
    let pending_before = pending.clone();
    let completed_before = completed.clone();

    let filter = Filter::parse(filter_terms, now)?;
    let include_non_pending = filter.has_explicit_status_filter() || filter.has_identity_selector();
    let mods = parse_mods(args, now)?;

    let mut changed = 0_u64;
    for task in &mut pending {
        if !include_non_pending && task.status != Status::Pending && task.status != Status::Waiting
        {
            continue;
        }
        if filter.matches(task, now) {
            apply_mods(task, &mods, now)?;
            task.modified = now;
            changed += 1;
        }
    }

    if include_non_pending {
        for task in &mut completed {
            if filter.matches(task, now) {
                apply_mods(task, &mods, now)?;
                task.modified = now;
                changed += 1;
            }
        }
    }

    if changed > 0 {
        store.push_undo_snapshot(&pending_before, &completed_before)?;
        store.save_pending(&pending)?;
        if include_non_pending {
            store.save_completed(&completed)?;
        }
    }

    println!("Modified {changed} task(s).");
    Ok(())
}

#[instrument(skip(store, filter_terms, now))]
fn cmd_start(
    store: &mut DataStore,
    filter_terms: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!("command start");

    let mut pending = store.load_pending()?;
    let pending_before = pending.clone();
    let filter = Filter::parse(filter_terms, now)?;

    let mut started = 0_u64;
    for task in &mut pending {
        if task.status != Status::Pending || task.is_waiting(now) {
            continue;
        }
        if filter.matches(task, now) && task.start.is_none() {
            task.start = Some(now);
            task.modified = now;
            started += 1;
        }
    }

    if started > 0 {
        let completed = store.load_completed()?;
        store.push_undo_snapshot(&pending_before, &completed)?;
        store.update_pending(&pending)?;
    }

    println!("Started {started} task(s).");
    Ok(())
}

#[instrument(skip(store, filter_terms, now))]
fn cmd_stop(
    store: &mut DataStore,
    filter_terms: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!("command stop");

    let mut pending = store.load_pending()?;
    let pending_before = pending.clone();
    let filter = Filter::parse(filter_terms, now)?;

    let mut stopped = 0_u64;
    for task in &mut pending {
        if task.status != Status::Pending {
            continue;
        }
        if filter.matches(task, now) && task.start.is_some() {
            task.start = None;
            task.modified = now;
            stopped += 1;
        }
    }

    if stopped > 0 {
        let completed = store.load_completed()?;
        store.push_undo_snapshot(&pending_before, &completed)?;
        store.update_pending(&pending)?;
    }

    println!("Stopped {stopped} task(s).");
    Ok(())
}

#[instrument(skip(store, filter_terms, args, now))]
fn cmd_annotate(
    store: &mut DataStore,
    filter_terms: &[String],
    args: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!("command annotate");

    if args.is_empty() {
        return Err(anyhow!("annotate requires annotation text"));
    }
    let note = args.join(" ");

    let mut pending = store.load_pending()?;
    let mut completed = store.load_completed()?;
    let pending_before = pending.clone();
    let completed_before = completed.clone();

    let filter = Filter::parse(filter_terms, now)?;
    let mut touched = 0_u64;

    for task in &mut pending {
        if filter.matches(task, now) {
            task.annotations.push(Annotation {
                entry: now,
                description: note.clone(),
            });
            task.modified = now;
            touched += 1;
        }
    }

    for task in &mut completed {
        if filter.matches(task, now) {
            task.annotations.push(Annotation {
                entry: now,
                description: note.clone(),
            });
            task.modified = now;
            touched += 1;
        }
    }

    if touched > 0 {
        store.push_undo_snapshot(&pending_before, &completed_before)?;
        store.save_pending(&pending)?;
        store.save_completed(&completed)?;
    }

    println!("Annotated {touched} task(s).");
    Ok(())
}

#[instrument(skip(store, filter_terms, args, now))]
fn cmd_denotate(
    store: &mut DataStore,
    filter_terms: &[String],
    args: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!("command denotate");

    if args.is_empty() {
        return Err(anyhow!("denotate requires an index or text selector"));
    }

    let selector_idx = if args.len() == 1 {
        args[0].parse::<usize>().ok().filter(|idx| *idx > 0)
    } else {
        None
    };
    let selector_text = if selector_idx.is_none() {
        Some(args.join(" ").to_ascii_lowercase())
    } else {
        None
    };

    let mut pending = store.load_pending()?;
    let mut completed = store.load_completed()?;
    let pending_before = pending.clone();
    let completed_before = completed.clone();

    let filter = Filter::parse(filter_terms, now)?;

    let (tasks_touched_p, removed_p) = denotate_tasks(
        &mut pending,
        &filter,
        selector_idx,
        selector_text.as_deref(),
        now,
    );
    let (tasks_touched_c, removed_c) = denotate_tasks(
        &mut completed,
        &filter,
        selector_idx,
        selector_text.as_deref(),
        now,
    );

    let tasks_touched = tasks_touched_p + tasks_touched_c;
    let removed = removed_p + removed_c;

    if removed > 0 {
        store.push_undo_snapshot(&pending_before, &completed_before)?;
        store.save_pending(&pending)?;
        store.save_completed(&completed)?;
    }

    println!("Removed {removed} annotation(s) from {tasks_touched} task(s).");
    Ok(())
}

fn denotate_tasks(
    tasks: &mut [Task],
    filter: &Filter,
    selector_idx: Option<usize>,
    selector_text: Option<&str>,
    now: chrono::DateTime<Utc>,
) -> (u64, u64) {
    let mut touched = 0_u64;
    let mut removed = 0_u64;

    for task in tasks {
        if !filter.matches(task, now) {
            continue;
        }

        let before = task.annotations.len();
        if let Some(idx) = selector_idx {
            if idx <= task.annotations.len() {
                task.annotations.remove(idx - 1);
            }
        } else if let Some(text) = selector_text {
            task.annotations
                .retain(|ann| !ann.description.to_ascii_lowercase().contains(text));
        }

        let after = task.annotations.len();
        if after < before {
            touched += 1;
            removed += (before - after) as u64;
            task.modified = now;
        }
    }

    (touched, removed)
}

#[instrument(skip(store, filter_terms, now))]
fn cmd_duplicate(
    store: &mut DataStore,
    filter_terms: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!("command duplicate");

    let mut pending = store.load_pending()?;
    let pending_before = pending.clone();
    let completed = store.load_completed()?;

    let filter = Filter::parse(filter_terms, now)?;
    let mut next_id = store.next_id(&pending);

    let mut clones = Vec::new();
    for task in &pending {
        if task.status == Status::Deleted {
            continue;
        }
        if filter.matches(task, now) {
            let mut duplicate = task.clone();
            duplicate.uuid = uuid::Uuid::new_v4();
            duplicate.id = Some(next_id);
            duplicate.status = Status::Pending;
            duplicate.entry = now;
            duplicate.modified = now;
            duplicate.end = None;
            duplicate.start = None;
            next_id += 1;
            clones.push(duplicate);
        }
    }

    let duplicated = clones.len() as u64;
    if duplicated > 0 {
        pending.extend(clones);
        pending.sort_by_key(|task| task.id.unwrap_or(u64::MAX));
        store.push_undo_snapshot(&pending_before, &completed)?;
        store.save_pending(&pending)?;
    }

    println!("Duplicated {duplicated} task(s).");
    Ok(())
}

#[instrument(skip(store, args, now))]
fn cmd_log(
    store: &mut DataStore,
    args: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!("command log");

    let pending = store.load_pending()?;
    let completed = store.load_completed()?;
    let pending_before = pending.clone();
    let completed_before = completed.clone();

    let next_id = store.next_id(&pending);
    let (description, mods) = parse_desc_and_mods(args, now)?;

    let mut task = Task::new_pending(description, now, next_id);
    apply_mods(&mut task, &mods, now)?;
    task.status = Status::Completed;
    task.end = Some(now);
    task.start = None;
    task.modified = now;

    let mut completed_new = completed;
    completed_new.push(task.clone());
    completed_new.sort_by_key(|t| (t.end, t.id));

    store.push_undo_snapshot(&pending_before, &completed_before)?;
    store.save_completed(&completed_new)?;

    println!("Logged task {}.", task.id.unwrap_or(next_id));
    Ok(())
}

#[instrument(skip(store, filter_terms, now))]
fn cmd_done(
    store: &mut DataStore,
    filter_terms: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!("command done");

    let mut pending = store.load_pending()?;
    let mut completed = store.load_completed()?;
    let pending_before = pending.clone();
    let completed_before = completed.clone();

    let filter = Filter::parse(filter_terms, now)?;

    let mut moved = 0_u64;
    let mut keep = Vec::with_capacity(pending.len());

    for mut task in pending.drain(..) {
        if (task.status == Status::Pending || task.status == Status::Waiting)
            && filter.matches(&task, now)
        {
            task.status = Status::Completed;
            task.end = Some(now);
            task.start = None;
            task.modified = now;
            completed.push(task);
            moved += 1;
        } else {
            keep.push(task);
        }
    }

    if moved > 0 {
        store.push_undo_snapshot(&pending_before, &completed_before)?;
        store.save_pending(&keep)?;
        store.save_completed(&completed)?;
    }

    println!("Completed {moved} task(s).");
    Ok(())
}

#[instrument(skip(store, filter_terms, now))]
fn cmd_delete(
    store: &mut DataStore,
    filter_terms: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!("command delete");

    let mut pending = store.load_pending()?;
    let pending_before = pending.clone();
    let filter = Filter::parse(filter_terms, now)?;

    let mut deleted = 0_u64;
    for task in &mut pending {
        if (task.status == Status::Pending || task.status == Status::Waiting)
            && filter.matches(task, now)
        {
            task.status = Status::Deleted;
            task.start = None;
            task.modified = now;
            deleted += 1;
        }
    }

    if deleted > 0 {
        let completed = store.load_completed()?;
        store.push_undo_snapshot(&pending_before, &completed)?;
        store.save_pending(&pending)?;
    }

    println!("Deleted {deleted} task(s) (soft-delete).");
    Ok(())
}

#[instrument(skip(store))]
fn cmd_undo(store: &mut DataStore) -> anyhow::Result<()> {
    info!("command undo");

    let Some((pending, completed)) = store.pop_undo_snapshot()? else {
        println!("No undo transactions available.");
        return Ok(());
    };

    store.save_pending(&pending)?;
    store.save_completed(&completed)?;

    println!("Undo completed.");
    Ok(())
}

#[instrument(skip(store, filter_terms, now))]
fn cmd_export(
    store: &mut DataStore,
    filter_terms: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!("command export");

    let pending = store.load_pending()?;
    let completed = store.load_completed()?;
    let filter = Filter::parse(filter_terms, now)?;

    let rows: Vec<Task> = pending
        .into_iter()
        .chain(completed)
        .filter(|task| filter.matches_without_waiting_guard(task, now))
        .collect();

    let out = serde_json::to_string(&rows)?;
    println!("{out}");
    Ok(())
}

#[instrument(skip(store))]
fn cmd_import(store: &mut DataStore) -> anyhow::Result<()> {
    info!("command import");

    let mut stdin = String::new();
    io::stdin()
        .read_to_string(&mut stdin)
        .context("failed reading stdin")?;

    let trimmed = stdin.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("import: empty input"));
    }

    let mut pending = store.load_pending()?;
    let mut completed = store.load_completed()?;
    let pending_before = pending.clone();
    let completed_before = completed.clone();

    let mut imported = 0_u64;
    if trimmed.starts_with('[') {
        let tasks: Vec<Task> =
            serde_json::from_str(trimmed).context("failed parsing JSON array")?;
        for task in tasks {
            imported += 1;
            match task.status {
                Status::Completed => completed.push(task),
                Status::Pending | Status::Deleted => pending.push(task),
                Status::Waiting => {
                    let mut task = task;
                    task.status = Status::Pending;
                    pending.push(task);
                }
            }
        }
    } else {
        for (idx, line) in trimmed.lines().enumerate() {
            let token = line.trim();
            if token.is_empty() {
                continue;
            }
            let task: Task = serde_json::from_str(token)
                .with_context(|| format!("failed parsing import line {}", idx + 1))?;
            imported += 1;
            match task.status {
                Status::Completed => completed.push(task),
                Status::Pending | Status::Deleted => pending.push(task),
                Status::Waiting => {
                    let mut task = task;
                    task.status = Status::Pending;
                    pending.push(task);
                }
            }
        }
    }

    if imported > 0 {
        store.push_undo_snapshot(&pending_before, &completed_before)?;
        store.save_pending(&pending)?;
        store.save_completed(&completed)?;
    }

    println!("Imported {imported} task(s).");
    Ok(())
}

#[instrument(skip(store))]
fn cmd_projects(store: &mut DataStore) -> anyhow::Result<()> {
    let pending = store.load_pending()?;
    let mut set = BTreeSet::new();
    for task in pending {
        if let Some(project) = task.project {
            set.insert(project);
        }
    }

    for project in set {
        println!("{project}");
    }
    Ok(())
}

#[instrument(skip(store))]
fn cmd_tags(store: &mut DataStore) -> anyhow::Result<()> {
    let pending = store.load_pending()?;
    let mut set = BTreeSet::new();
    for task in pending {
        for tag in task.tags {
            set.insert(tag);
        }
    }

    for tag in set {
        println!("{tag}");
    }
    Ok(())
}

#[instrument(skip(store, cfg, args))]
fn cmd_context(store: &mut DataStore, cfg: &Config, args: &[String]) -> anyhow::Result<()> {
    if args.is_empty() {
        let active = store.get_active_context()?;
        println!("active={}", active.unwrap_or_else(|| "none".to_string()));

        for (key, value) in cfg.iter() {
            if let Some(name) = key.strip_prefix("context.") {
                println!("{name} {value}");
            }
        }
        return Ok(());
    }

    let cmd = args[0].to_ascii_lowercase();
    if cmd == "none" || cmd == "clear" {
        store.set_active_context(None)?;
        println!("Context cleared.");
        return Ok(());
    }

    let name = args[0].as_str();
    let key = format!("context.{name}");
    if cfg.get(&key).is_none() {
        return Err(anyhow!("unknown context: {name}"));
    }

    store.set_active_context(Some(name))?;
    println!("Context set: {name}");
    Ok(())
}

fn cmd_commands() -> anyhow::Result<()> {
    for command in known_command_names() {
        println!("{command}");
    }
    Ok(())
}

fn cmd_show(cfg: &Config) -> anyhow::Result<()> {
    for (k, v) in cfg.iter() {
        println!("{k}={v}");
    }
    Ok(())
}

fn cmd_unique(store: &mut DataStore, args: &[String]) -> anyhow::Result<()> {
    if args.is_empty() {
        println!("project");
        println!("tag");
        println!("status");
        return Ok(());
    }

    match args[0].as_str() {
        "project" | "projects" => cmd_projects(store),
        "tag" | "tags" => cmd_tags(store),
        "status" => {
            println!("pending");
            println!("completed");
            println!("deleted");
            println!("waiting");
            Ok(())
        }
        _ => Ok(()),
    }
}

fn cmd_help() -> anyhow::Result<()> {
    println!(
        "Implemented commands: add, append, prepend, list/next, info, modify, start, stop, annotate, denotate, duplicate, log, done, delete, undo, export, import, projects, tags, context"
    );
    Ok(())
}

#[instrument(skip(store, cfg, command, filter_terms))]
fn resolve_effective_filter_terms(
    store: &DataStore,
    cfg: &Config,
    command: &str,
    filter_terms: &[String],
) -> anyhow::Result<Vec<String>> {
    if !command_uses_filter(command) {
        return Ok(filter_terms.to_vec());
    }

    let mut out = Vec::new();
    if let Some(active) = store.get_active_context()? {
        let key = format!("context.{active}");
        if let Some(expr) = cfg.get(&key) {
            out.extend(expr.split_whitespace().map(ToString::to_string));
        }
    }
    out.extend(filter_terms.iter().cloned());
    Ok(out)
}

fn command_uses_filter(command: &str) -> bool {
    matches!(
        command,
        "append"
            | "prepend"
            | "list"
            | "next"
            | "info"
            | "modify"
            | "start"
            | "stop"
            | "annotate"
            | "denotate"
            | "duplicate"
            | "done"
            | "delete"
    )
}

#[derive(Debug, Clone)]
enum Mod {
    TagAdd(String),
    TagRemove(String),
    Project(String),
    Priority(String),
    Due(chrono::DateTime<Utc>),
    Scheduled(chrono::DateTime<Utc>),
    Wait(chrono::DateTime<Utc>),
    Depends(uuid::Uuid),
}

#[instrument(skip(args, now))]
fn parse_desc_and_mods(
    args: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<(String, Vec<Mod>)> {
    let mut desc_parts = Vec::new();
    let mut mods = Vec::new();

    let mut literal = false;
    for arg in args {
        if arg == "--" {
            literal = true;
            continue;
        }

        if !literal && let Some(one_mod) = parse_one_mod(arg, now)? {
            mods.push(one_mod);
            continue;
        }

        desc_parts.push(arg.clone());
    }

    if desc_parts.is_empty() {
        return Err(anyhow!("add/log: description is required"));
    }

    Ok((desc_parts.join(" "), mods))
}

#[instrument(skip(args, now))]
fn parse_mods(args: &[String], now: chrono::DateTime<Utc>) -> anyhow::Result<Vec<Mod>> {
    let mut mods = Vec::new();
    for arg in args {
        if let Some(one_mod) = parse_one_mod(arg, now)? {
            mods.push(one_mod);
        } else {
            warn!(arg = %arg, "unrecognized modifier token ignored");
        }
    }
    Ok(mods)
}

fn parse_one_mod(tok: &str, now: chrono::DateTime<Utc>) -> anyhow::Result<Option<Mod>> {
    if let Some(tag) = tok.strip_prefix('+') {
        return Ok(Some(Mod::TagAdd(tag.to_string())));
    }
    if let Some(tag) = tok.strip_prefix('-') {
        return Ok(Some(Mod::TagRemove(tag.to_string())));
    }

    let (key, value) = if let Some((k, v)) = tok.split_once(':') {
        (k, v)
    } else if let Some((k, v)) = tok.split_once('=') {
        (k, v)
    } else {
        return Ok(None);
    };

    let key = key.to_ascii_lowercase();

    match key.as_str() {
        "project" => Ok(Some(Mod::Project(value.to_string()))),
        "pri" | "priority" => Ok(Some(Mod::Priority(value.to_string()))),
        "due" => Ok(Some(Mod::Due(parse_date_expr(value, now)?))),
        "scheduled" => Ok(Some(Mod::Scheduled(parse_date_expr(value, now)?))),
        "wait" => Ok(Some(Mod::Wait(parse_date_expr(value, now)?))),
        "depends" => {
            let uuid = uuid::Uuid::parse_str(value)?;
            Ok(Some(Mod::Depends(uuid)))
        }
        _ => Ok(None),
    }
}

fn apply_mods(task: &mut Task, mods: &[Mod], now: chrono::DateTime<Utc>) -> anyhow::Result<()> {
    for one_mod in mods {
        match one_mod {
            Mod::TagAdd(tag) => {
                if task.tags.iter().all(|existing| existing != tag) {
                    task.tags.push(tag.clone());
                }
            }
            Mod::TagRemove(tag) => {
                task.tags.retain(|existing| existing != tag);
            }
            Mod::Project(project) => {
                task.project = Some(project.clone());
            }
            Mod::Priority(priority) => {
                task.priority = Some(priority.clone());
            }
            Mod::Due(dt) => {
                task.due = Some(*dt);
            }
            Mod::Scheduled(dt) => {
                task.scheduled = Some(*dt);
            }
            Mod::Wait(dt) => {
                task.wait = Some(*dt);
                if *dt <= now && task.status == Status::Waiting {
                    task.status = Status::Pending;
                }
            }
            Mod::Depends(dep) => {
                if task.depends.iter().all(|existing| existing != dep) {
                    task.depends.push(*dep);
                }
            }
        }
    }

    Ok(())
}
