use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Read};

use anyhow::{Context, anyhow};
use chrono::{Local, Utc};
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, info, instrument, warn};

use crate::cli::Invocation;
use crate::config::Config;
use crate::datastore::DataStore;
use crate::datetime::parse_date_expr;
use crate::filter::Filter;
use crate::hooks::HookRunner;
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
    let hooks = HookRunner::new(cfg, &store.data_dir);
    hooks.run_on_launch()?;
    let command = inv.command.as_str();
    let effective_filters = resolve_effective_filter_terms(store, cfg, command, &inv.filter_terms)?;

    debug!(
        command,
        filter = ?inv.filter_terms,
        args = ?inv.command_args,
        "dispatching command"
    );

    match command {
        "add" => cmd_add(store, &hooks, cfg, renderer, &inv.command_args, now),
        "append" => cmd_append(store, &hooks, &effective_filters, &inv.command_args, now),
        "prepend" => cmd_prepend(store, &hooks, &effective_filters, &inv.command_args, now),
        "list" | "next" => cmd_list(store, cfg, renderer, command, &effective_filters, now),
        "info" => cmd_info(store, renderer, &effective_filters, now),
        "modify" => cmd_modify(store, &hooks, &effective_filters, &inv.command_args, now),
        "start" => cmd_start(store, &hooks, &effective_filters, now),
        "stop" => cmd_stop(store, &hooks, &effective_filters, now),
        "annotate" => cmd_annotate(store, &hooks, &effective_filters, &inv.command_args, now),
        "denotate" => cmd_denotate(store, &hooks, &effective_filters, &inv.command_args, now),
        "duplicate" => cmd_duplicate(store, &hooks, &effective_filters, now),
        "log" => cmd_log(store, &hooks, &inv.command_args, now),
        "done" => cmd_done(store, &hooks, &effective_filters, now),
        "delete" => cmd_delete(store, &hooks, &effective_filters, now),
        "undo" => cmd_undo(store),
        "export" => cmd_export(store, &effective_filters, now),
        "import" => cmd_import(store, &hooks),
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
        other => {
            if is_report_command(cfg, other) {
                cmd_report(store, cfg, renderer, other, &effective_filters, now)
            } else {
                Err(anyhow!("unknown command: {other}"))
            }
        }
    }
}

#[instrument(skip(store, hooks, _cfg, _renderer, args, now))]
fn cmd_add(
    store: &mut DataStore,
    hooks: &HookRunner,
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
    task = hooks.apply_on_add(&task)?;
    if task.id.is_none() {
        task.id = Some(next_id);
    }

    pending = store.add_task(pending, task.clone())?;
    store.push_undo_snapshot(&pending_before, &completed)?;

    debug!(pending_count = pending.len(), "task added");
    println!("Created task {}.", task.id.unwrap_or(next_id));
    Ok(())
}

#[instrument(skip(store, hooks, filter_terms, args, now))]
fn cmd_append(
    store: &mut DataStore,
    hooks: &HookRunner,
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
            let old = task.clone();
            task.description = format!("{} {}", task.description, suffix)
                .trim()
                .to_string();
            task.modified = now;
            *task = hooks.apply_on_modify(&old, task)?;
            changed += 1;
        }
    }

    if include_non_pending {
        for task in &mut completed {
            if filter.matches(task, now) {
                let old = task.clone();
                task.description = format!("{} {}", task.description, suffix)
                    .trim()
                    .to_string();
                task.modified = now;
                *task = hooks.apply_on_modify(&old, task)?;
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

#[instrument(skip(store, hooks, filter_terms, args, now))]
fn cmd_prepend(
    store: &mut DataStore,
    hooks: &HookRunner,
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
            let old = task.clone();
            task.description = format!("{} {}", prefix, task.description)
                .trim()
                .to_string();
            task.modified = now;
            *task = hooks.apply_on_modify(&old, task)?;
            changed += 1;
        }
    }

    if include_non_pending {
        for task in &mut completed {
            if filter.matches(task, now) {
                let old = task.clone();
                task.description = format!("{} {}", prefix, task.description)
                    .trim()
                    .to_string();
                task.modified = now;
                *task = hooks.apply_on_modify(&old, task)?;
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

#[instrument(skip(store, cfg, renderer, report_name, filter_terms, now))]
fn cmd_list(
    store: &mut DataStore,
    cfg: &Config,
    renderer: &mut Renderer,
    report_name: &str,
    filter_terms: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!("command list/next");

    let effective_report_name = if report_name == "list" {
        "next"
    } else {
        report_name
    };
    if let Some(spec) = load_report_spec(cfg, effective_report_name) {
        return run_report(store, renderer, &spec, filter_terms, now);
    }

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

#[instrument(skip(store, cfg, renderer, filter_terms, now))]
fn cmd_report(
    store: &mut DataStore,
    cfg: &Config,
    renderer: &mut Renderer,
    report_name: &str,
    filter_terms: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    let spec = load_report_spec(cfg, report_name)
        .ok_or_else(|| anyhow!("unknown report: {report_name}"))?;
    run_report(store, renderer, &spec, filter_terms, now)
}

#[instrument(skip(store, renderer, spec, cli_filter_terms, now))]
fn run_report(
    store: &mut DataStore,
    renderer: &mut Renderer,
    spec: &ReportSpec,
    cli_filter_terms: &[String],
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    info!(report = %spec.name, "command report");

    let pending = store.load_pending()?;
    let completed = store.load_completed()?;

    let mut effective_filter_terms = spec.filter_terms.clone();
    effective_filter_terms.extend(cli_filter_terms.iter().cloned());

    let filter = Filter::parse(&effective_filter_terms, now)?;

    let mut rows: Vec<Task> = pending
        .into_iter()
        .chain(completed)
        .filter(|task| filter.matches(task, now))
        .collect();

    rows.sort_by(|a, b| compare_tasks_for_report(a, b, &spec.sort, now));
    if let Some(limit) = spec.limit {
        rows.truncate(limit);
    }

    let table_rows: Vec<Vec<String>> = rows
        .iter()
        .map(|task| {
            spec.columns
                .iter()
                .map(|col| format_report_cell(task, *col, now))
                .collect()
        })
        .collect();

    renderer.print_report_table(&spec.labels, &table_rows)?;
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

#[instrument(skip(store, hooks, filter_terms, args, now))]
fn cmd_modify(
    store: &mut DataStore,
    hooks: &HookRunner,
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
            let old = task.clone();
            apply_mods(task, &mods, now)?;
            task.modified = now;
            *task = hooks.apply_on_modify(&old, task)?;
            changed += 1;
        }
    }

    if include_non_pending {
        for task in &mut completed {
            if filter.matches(task, now) {
                let old = task.clone();
                apply_mods(task, &mods, now)?;
                task.modified = now;
                *task = hooks.apply_on_modify(&old, task)?;
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

#[instrument(skip(store, hooks, filter_terms, now))]
fn cmd_start(
    store: &mut DataStore,
    hooks: &HookRunner,
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
            let old = task.clone();
            task.start = Some(now);
            task.modified = now;
            *task = hooks.apply_on_modify(&old, task)?;
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

#[instrument(skip(store, hooks, filter_terms, now))]
fn cmd_stop(
    store: &mut DataStore,
    hooks: &HookRunner,
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
            let old = task.clone();
            task.start = None;
            task.modified = now;
            *task = hooks.apply_on_modify(&old, task)?;
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

#[instrument(skip(store, hooks, filter_terms, args, now))]
fn cmd_annotate(
    store: &mut DataStore,
    hooks: &HookRunner,
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
            let old = task.clone();
            task.annotations.push(Annotation {
                entry: now,
                description: note.clone(),
            });
            task.modified = now;
            *task = hooks.apply_on_modify(&old, task)?;
            touched += 1;
        }
    }

    for task in &mut completed {
        if filter.matches(task, now) {
            let old = task.clone();
            task.annotations.push(Annotation {
                entry: now,
                description: note.clone(),
            });
            task.modified = now;
            *task = hooks.apply_on_modify(&old, task)?;
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

#[instrument(skip(store, hooks, filter_terms, args, now))]
fn cmd_denotate(
    store: &mut DataStore,
    hooks: &HookRunner,
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
        hooks,
        &filter,
        selector_idx,
        selector_text.as_deref(),
        now,
    )?;
    let (tasks_touched_c, removed_c) = denotate_tasks(
        &mut completed,
        hooks,
        &filter,
        selector_idx,
        selector_text.as_deref(),
        now,
    )?;

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
    hooks: &HookRunner,
    filter: &Filter,
    selector_idx: Option<usize>,
    selector_text: Option<&str>,
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<(u64, u64)> {
    let mut touched = 0_u64;
    let mut removed = 0_u64;

    for task in tasks {
        if !filter.matches(task, now) {
            continue;
        }

        let old = task.clone();
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
            *task = hooks.apply_on_modify(&old, task)?;
        }
    }

    Ok((touched, removed))
}

#[instrument(skip(store, hooks, filter_terms, now))]
fn cmd_duplicate(
    store: &mut DataStore,
    hooks: &HookRunner,
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
            duplicate = hooks.apply_on_add(&duplicate)?;
            if duplicate.id.is_none() {
                duplicate.id = Some(next_id);
            }
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

#[instrument(skip(store, hooks, args, now))]
fn cmd_log(
    store: &mut DataStore,
    hooks: &HookRunner,
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
    task = hooks.apply_on_add(&task)?;
    if task.id.is_none() {
        task.id = Some(next_id);
    }

    let mut completed_new = completed;
    completed_new.push(task.clone());
    completed_new.sort_by_key(|t| (t.end, t.id));

    store.push_undo_snapshot(&pending_before, &completed_before)?;
    store.save_completed(&completed_new)?;

    println!("Logged task {}.", task.id.unwrap_or(next_id));
    Ok(())
}

#[instrument(skip(store, hooks, filter_terms, now))]
fn cmd_done(
    store: &mut DataStore,
    hooks: &HookRunner,
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
            let old = task.clone();
            task.status = Status::Completed;
            task.end = Some(now);
            task.start = None;
            task.modified = now;
            task = hooks.apply_on_modify(&old, &task)?;

            match task.status {
                Status::Completed => completed.push(task),
                Status::Deleted | Status::Pending | Status::Waiting => keep.push(task),
            }
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

#[instrument(skip(store, hooks, filter_terms, now))]
fn cmd_delete(
    store: &mut DataStore,
    hooks: &HookRunner,
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
            let old = task.clone();
            task.status = Status::Deleted;
            task.start = None;
            task.end = Some(now);
            task.modified = now;
            *task = hooks.apply_on_modify(&old, task)?;
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

#[derive(Debug, Clone, Deserialize)]
struct ImportTask {
    #[serde(default)]
    uuid: Option<uuid::Uuid>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    status: Option<Status>,
    #[serde(default, with = "crate::datetime::taskwarrior_date_serde::option")]
    entry: Option<chrono::DateTime<Utc>>,
    #[serde(default, with = "crate::datetime::taskwarrior_date_serde::option")]
    modified: Option<chrono::DateTime<Utc>>,
    #[serde(default, with = "crate::datetime::taskwarrior_date_serde::option")]
    end: Option<chrono::DateTime<Utc>>,
    #[serde(default, with = "crate::datetime::taskwarrior_date_serde::option")]
    start: Option<chrono::DateTime<Utc>>,
    #[serde(default)]
    project: Option<String>,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default, with = "crate::datetime::taskwarrior_date_serde::option")]
    due: Option<chrono::DateTime<Utc>>,
    #[serde(default, with = "crate::datetime::taskwarrior_date_serde::option")]
    scheduled: Option<chrono::DateTime<Utc>>,
    #[serde(default, with = "crate::datetime::taskwarrior_date_serde::option")]
    wait: Option<chrono::DateTime<Utc>>,
    #[serde(default)]
    depends: Vec<uuid::Uuid>,
    #[serde(default)]
    annotations: Vec<Annotation>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

#[instrument(skip(store, hooks))]
fn cmd_import(store: &mut DataStore, hooks: &HookRunner) -> anyhow::Result<()> {
    info!("command import");
    let now = Utc::now();

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

    let imported = parse_import_items(trimmed)?;
    let mut adds = 0_u64;
    let mut mods = 0_u64;

    for row in imported {
        let existing = row
            .uuid
            .and_then(|uuid| find_task_by_uuid(&pending, &completed, uuid));
        let mut task = normalize_import_item(row, now);
        normalize_import_identity_and_status(&mut task, existing.as_ref(), store.next_id(&pending));

        if let Some(old) = existing.as_ref() {
            task = hooks.apply_on_modify(old, &task)?;
            mods += 1;
        } else {
            task = hooks.apply_on_add(&task)?;
            adds += 1;
        }
        normalize_import_identity_and_status(&mut task, existing.as_ref(), store.next_id(&pending));
        upsert_imported_task(
            &mut pending,
            &mut completed,
            task,
            existing.as_ref().map(|t| t.uuid),
        );
    }

    let imported_count = adds + mods;
    if imported_count > 0 {
        store.push_undo_snapshot(&pending_before, &completed_before)?;
        store.save_pending(&pending)?;
        store.save_completed(&completed)?;
    }

    println!("Imported {imported_count} task(s).");
    Ok(())
}

fn parse_import_items(trimmed: &str) -> anyhow::Result<Vec<ImportTask>> {
    if trimmed.starts_with('[') {
        return serde_json::from_str(trimmed).context("failed parsing JSON array");
    }

    if trimmed.starts_with('{') {
        if let Ok(item) = serde_json::from_str::<ImportTask>(trimmed) {
            return Ok(vec![item]);
        }
    }

    let mut out = Vec::new();
    for (idx, line) in trimmed.lines().enumerate() {
        let token = line.trim();
        if token.is_empty() {
            continue;
        }
        let item: ImportTask = serde_json::from_str(token)
            .with_context(|| format!("failed parsing import line {}", idx + 1))?;
        out.push(item);
    }

    if out.is_empty() {
        return Err(anyhow!("import: empty input"));
    }

    Ok(out)
}

fn normalize_import_item(item: ImportTask, now: chrono::DateTime<Utc>) -> Task {
    let status = item.status.unwrap_or(Status::Pending);
    let entry = item.entry.unwrap_or(now);
    let modified = item.modified.unwrap_or(now);
    let mut task = Task {
        uuid: item.uuid.unwrap_or_else(uuid::Uuid::new_v4),
        id: None,
        description: item.description.unwrap_or_default(),
        status,
        entry,
        modified,
        end: item.end,
        start: item.start,
        project: item.project,
        priority: item.priority,
        tags: item.tags,
        due: item.due,
        scheduled: item.scheduled,
        wait: item.wait,
        depends: item.depends,
        annotations: item.annotations,
        extra: item.extra,
    };
    normalize_import_status(&mut task);
    task
}

fn normalize_import_status(task: &mut Task) {
    if task.status == Status::Waiting {
        task.status = Status::Pending;
    }

    match task.status {
        Status::Pending => {
            task.end = None;
        }
        Status::Completed | Status::Deleted => {
            if task.end.is_none() {
                task.end = Some(task.modified);
            }
        }
        Status::Waiting => {}
    }
}

fn normalize_import_identity_and_status(task: &mut Task, old: Option<&Task>, next_id: u64) {
    normalize_import_status(task);
    match task.status {
        Status::Pending => {
            task.id = old
                .filter(|prev| prev.status == Status::Pending || prev.status == Status::Waiting)
                .and_then(|prev| prev.id)
                .or(Some(next_id));
        }
        Status::Completed | Status::Deleted => {
            task.id = None;
        }
        Status::Waiting => {}
    }
}

fn find_task_by_uuid(pending: &[Task], completed: &[Task], uuid: uuid::Uuid) -> Option<Task> {
    pending
        .iter()
        .find(|task| task.uuid == uuid)
        .cloned()
        .or_else(|| completed.iter().find(|task| task.uuid == uuid).cloned())
}

fn upsert_imported_task(
    pending: &mut Vec<Task>,
    completed: &mut Vec<Task>,
    task: Task,
    old_uuid: Option<uuid::Uuid>,
) {
    let old_uuid = old_uuid.unwrap_or(task.uuid);
    pending.retain(|row| row.uuid != old_uuid && row.uuid != task.uuid);
    completed.retain(|row| row.uuid != old_uuid && row.uuid != task.uuid);

    match task.status {
        Status::Completed => completed.push(task),
        Status::Pending | Status::Deleted | Status::Waiting => pending.push(task),
    }

    pending.sort_by_key(|row| row.id.unwrap_or(u64::MAX));
    completed.sort_by_key(|row| (row.end, row.id));
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
    if !command_uses_filter(cfg, command) {
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

fn command_uses_filter(cfg: &Config, command: &str) -> bool {
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
    ) || is_report_command(cfg, command)
}

#[derive(Debug, Clone, Copy)]
enum ReportColumn {
    Id,
    Uuid,
    Status,
    Project,
    Tags,
    Priority,
    Due,
    Scheduled,
    Wait,
    Entry,
    Modified,
    End,
    Start,
    Description,
    Urgency,
}

impl ReportColumn {
    fn parse(token: &str) -> Option<Self> {
        match token.to_ascii_lowercase().as_str() {
            "id" => Some(Self::Id),
            "uuid" => Some(Self::Uuid),
            "status" => Some(Self::Status),
            "project" => Some(Self::Project),
            "tags" | "tag" => Some(Self::Tags),
            "priority" | "pri" => Some(Self::Priority),
            "due" => Some(Self::Due),
            "scheduled" => Some(Self::Scheduled),
            "wait" => Some(Self::Wait),
            "entry" => Some(Self::Entry),
            "modified" => Some(Self::Modified),
            "end" => Some(Self::End),
            "start" => Some(Self::Start),
            "description" | "desc" => Some(Self::Description),
            "urgency" => Some(Self::Urgency),
            _ => None,
        }
    }

    fn default_label(&self) -> &'static str {
        match self {
            Self::Id => "ID",
            Self::Uuid => "UUID",
            Self::Status => "Status",
            Self::Project => "Project",
            Self::Tags => "Tags",
            Self::Priority => "Pri",
            Self::Due => "Due",
            Self::Scheduled => "Scheduled",
            Self::Wait => "Wait",
            Self::Entry => "Entry",
            Self::Modified => "Modified",
            Self::End => "End",
            Self::Start => "Start",
            Self::Description => "Description",
            Self::Urgency => "Urgency",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SortSpec {
    column: ReportColumn,
    descending: bool,
}

#[derive(Debug, Clone)]
struct ReportSpec {
    name: String,
    columns: Vec<ReportColumn>,
    labels: Vec<String>,
    sort: Vec<SortSpec>,
    filter_terms: Vec<String>,
    limit: Option<usize>,
}

fn is_report_command(cfg: &Config, command: &str) -> bool {
    cfg.get(&format!("report.{command}.columns")).is_some()
}

fn load_report_spec(cfg: &Config, report_name: &str) -> Option<ReportSpec> {
    let columns_raw = cfg.get(&format!("report.{report_name}.columns"))?;
    let columns: Vec<ReportColumn> = parse_config_list(&columns_raw)
        .into_iter()
        .filter_map(|token| ReportColumn::parse(&token))
        .collect();
    if columns.is_empty() {
        return None;
    }

    let labels_key = format!("report.{report_name}.labels");
    let mut labels = cfg
        .get(&labels_key)
        .map(|raw| parse_config_list(&raw))
        .unwrap_or_default();
    while labels.len() < columns.len() {
        labels.push(columns[labels.len()].default_label().to_string());
    }
    labels.truncate(columns.len());

    let sort = parse_sort_specs(cfg.get(&format!("report.{report_name}.sort")));
    let filter_terms = cfg
        .get(&format!("report.{report_name}.filter"))
        .map(|raw| raw.split_whitespace().map(ToString::to_string).collect())
        .unwrap_or_default();
    let limit = cfg
        .get(&format!("report.{report_name}.limit"))
        .and_then(|raw| raw.parse::<usize>().ok())
        .filter(|value| *value > 0);

    Some(ReportSpec {
        name: report_name.to_string(),
        columns,
        labels,
        sort,
        filter_terms,
        limit,
    })
}

fn parse_config_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .flat_map(str::split_whitespace)
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn parse_sort_specs(raw: Option<String>) -> Vec<SortSpec> {
    let Some(raw) = raw else {
        return Vec::new();
    };

    parse_config_list(&raw)
        .into_iter()
        .filter_map(|token| {
            let (field, descending) = if let Some(field) = token.strip_suffix('-') {
                (field, true)
            } else if let Some(field) = token.strip_suffix('+') {
                (field, false)
            } else {
                (token.as_str(), false)
            };
            let column = ReportColumn::parse(field)?;
            Some(SortSpec { column, descending })
        })
        .collect()
}

fn compare_tasks_for_report(
    a: &Task,
    b: &Task,
    sort_specs: &[SortSpec],
    now: chrono::DateTime<Utc>,
) -> Ordering {
    for sort_spec in sort_specs {
        let ordering = compare_tasks_on_column(a, b, sort_spec.column, now);
        if ordering != Ordering::Equal {
            return if sort_spec.descending {
                ordering.reverse()
            } else {
                ordering
            };
        }
    }

    a.id.unwrap_or(u64::MAX)
        .cmp(&b.id.unwrap_or(u64::MAX))
        .then_with(|| a.uuid.cmp(&b.uuid))
}

fn compare_tasks_on_column(
    a: &Task,
    b: &Task,
    column: ReportColumn,
    now: chrono::DateTime<Utc>,
) -> Ordering {
    match column {
        ReportColumn::Id => cmp_optional(a.id.as_ref(), b.id.as_ref()),
        ReportColumn::Uuid => a.uuid.cmp(&b.uuid),
        ReportColumn::Status => display_status(a, now).cmp(display_status(b, now)),
        ReportColumn::Project => cmp_optional(a.project.as_ref(), b.project.as_ref()),
        ReportColumn::Tags => a.tags.join(" ").cmp(&b.tags.join(" ")),
        ReportColumn::Priority => cmp_optional(a.priority.as_ref(), b.priority.as_ref()),
        ReportColumn::Due => cmp_optional(a.due.as_ref(), b.due.as_ref()),
        ReportColumn::Scheduled => cmp_optional(a.scheduled.as_ref(), b.scheduled.as_ref()),
        ReportColumn::Wait => cmp_optional(a.wait.as_ref(), b.wait.as_ref()),
        ReportColumn::Entry => a.entry.cmp(&b.entry),
        ReportColumn::Modified => a.modified.cmp(&b.modified),
        ReportColumn::End => cmp_optional(a.end.as_ref(), b.end.as_ref()),
        ReportColumn::Start => cmp_optional(a.start.as_ref(), b.start.as_ref()),
        ReportColumn::Description => a
            .description
            .to_ascii_lowercase()
            .cmp(&b.description.to_ascii_lowercase()),
        ReportColumn::Urgency => task_urgency(a, now)
            .partial_cmp(&task_urgency(b, now))
            .unwrap_or(Ordering::Equal),
    }
}

fn cmp_optional<T: Ord>(left: Option<&T>, right: Option<&T>) -> Ordering {
    match (left, right) {
        (Some(a), Some(b)) => a.cmp(b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn format_report_cell(task: &Task, column: ReportColumn, now: chrono::DateTime<Utc>) -> String {
    match column {
        ReportColumn::Id => task
            .id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "-".to_string()),
        ReportColumn::Uuid => task.uuid.to_string(),
        ReportColumn::Status => display_status(task, now).to_string(),
        ReportColumn::Project => task.project.clone().unwrap_or_default(),
        ReportColumn::Tags => task
            .tags
            .iter()
            .map(|tag| format!("+{tag}"))
            .collect::<Vec<_>>()
            .join(" "),
        ReportColumn::Priority => task.priority.clone().unwrap_or_default(),
        ReportColumn::Due => format_report_date(task.due),
        ReportColumn::Scheduled => format_report_date(task.scheduled),
        ReportColumn::Wait => format_report_date(task.wait),
        ReportColumn::Entry => task
            .entry
            .with_timezone(&Local)
            .format("%Y-%m-%d")
            .to_string(),
        ReportColumn::Modified => task
            .modified
            .with_timezone(&Local)
            .format("%Y-%m-%d")
            .to_string(),
        ReportColumn::End => format_report_date(task.end),
        ReportColumn::Start => format_report_date(task.start),
        ReportColumn::Description => task.description.clone(),
        ReportColumn::Urgency => format!("{:.3}", task_urgency(task, now)),
    }
}

fn format_report_date(date: Option<chrono::DateTime<Utc>>) -> String {
    date.map(|value| value.with_timezone(&Local).format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

fn display_status(task: &Task, now: chrono::DateTime<Utc>) -> &'static str {
    if task.status == Status::Pending && task.is_waiting(now) {
        "waiting"
    } else {
        match task.status {
            Status::Pending => "pending",
            Status::Completed => "completed",
            Status::Deleted => "deleted",
            Status::Waiting => "waiting",
        }
    }
}

fn task_urgency(task: &Task, now: chrono::DateTime<Utc>) -> f64 {
    if matches!(task.status, Status::Completed | Status::Deleted) {
        return 0.0;
    }

    let mut urgency = 0.0;

    urgency += task.tags.len() as f64 * 0.8;

    if let Some(priority) = task.priority.as_deref() {
        urgency += match priority.to_ascii_uppercase().as_str() {
            "H" => 6.0,
            "M" => 3.9,
            "L" => 1.8,
            _ => 0.0,
        };
    }

    if task.start.is_some() && !task.is_waiting(now) {
        urgency += 4.0;
    }
    if task.is_waiting(now) {
        urgency -= 3.0;
    }
    if !task.depends.is_empty() {
        urgency -= 5.0;
    }

    if let Some(due) = task.due {
        let delta = due - now;
        let days = delta.num_minutes() as f64 / (60.0 * 24.0);
        urgency += if days <= -1.0 {
            9.7
        } else if days <= 0.0 {
            9.3
        } else if days <= 1.0 {
            8.8
        } else if days <= 2.0 {
            8.4
        } else if days <= 7.0 {
            6.0
        } else {
            3.0
        };
    }

    urgency
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
