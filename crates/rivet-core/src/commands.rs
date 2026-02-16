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
use crate::task::{Status, Task};

pub fn known_command_names() -> Vec<&'static str> {
    vec![
        "add",
        "append",
        "prepend",
        "list",
        "next",
        "info",
        "modify",
        "done",
        "delete",
        "export",
        "import",
        "projects",
        "tags",
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

    debug!(
        command,
        filter = ?inv.filter_terms,
        args = ?inv.command_args,
        "dispatching command"
    );

    match command {
        "add" => cmd_add(store, cfg, renderer, &inv.command_args, now),
        "append" => cmd_append(store, &inv.filter_terms, &inv.command_args, now),
        "prepend" => cmd_prepend(store, &inv.filter_terms, &inv.command_args, now),
        "list" | "next" => cmd_list(store, cfg, renderer, &inv.filter_terms, now),
        "info" => cmd_info(store, renderer, &inv.filter_terms, now),
        "modify" => cmd_modify(store, &inv.filter_terms, &inv.command_args, now),
        "done" => cmd_done(store, &inv.filter_terms, now),
        "delete" => cmd_delete(store, &inv.filter_terms, now),
        "export" => cmd_export(store, &inv.filter_terms, now),
        "import" => cmd_import(store),
        "projects" => cmd_projects(store),
        "tags" => cmd_tags(store),
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
    let next_id = store.next_id(&pending);

    let (description, mods) = parse_desc_and_mods(args, now)?;
    let mut task = Task::new_pending(description, now, next_id);
    apply_mods(&mut task, &mods, now)?;

    pending = store.add_task(pending, task.clone())?;
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
    let mut changed = 0_u64;

    for task in &mut pending {
        if task.status != Status::Pending {
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

    store.update_pending(&pending)?;
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
    let mut changed = 0_u64;

    for task in &mut pending {
        if task.status != Status::Pending {
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

    store.update_pending(&pending)?;
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
    let filter = Filter::parse(filter_terms, now)?;
    let mods = parse_mods(args, now)?;

    let mut changed = 0_u64;
    for task in &mut pending {
        if task.status != Status::Pending && task.status != Status::Waiting {
            continue;
        }
        if filter.matches(task, now) {
            apply_mods(task, &mods, now)?;
            task.modified = now;
            changed += 1;
        }
    }

    store.update_pending(&pending)?;
    println!("Modified {changed} task(s).");
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

    let filter = Filter::parse(filter_terms, now)?;

    let mut moved = 0_u64;
    let mut keep = Vec::with_capacity(pending.len());

    for mut task in pending.drain(..) {
        if (task.status == Status::Pending || task.status == Status::Waiting)
            && filter.matches(&task, now)
        {
            task.status = Status::Completed;
            task.end = Some(now);
            task.modified = now;
            completed.push(task);
            moved += 1;
        } else {
            keep.push(task);
        }
    }

    store.save_pending(&keep)?;
    store.save_completed(&completed)?;
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
    let filter = Filter::parse(filter_terms, now)?;

    let mut deleted = 0_u64;
    for task in &mut pending {
        if (task.status == Status::Pending || task.status == Status::Waiting)
            && filter.matches(task, now)
        {
            task.status = Status::Deleted;
            task.modified = now;
            deleted += 1;
        }
    }

    store.save_pending(&pending)?;
    println!("Deleted {deleted} task(s) (soft-delete).");
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
        .filter(|task| filter.matches(task, now))
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

    if trimmed.starts_with('[') {
        let tasks: Vec<Task> =
            serde_json::from_str(trimmed).context("failed parsing JSON array")?;
        for task in tasks {
            match task.status {
                Status::Completed => completed.push(task),
                Status::Pending | Status::Waiting | Status::Deleted => pending.push(task),
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
            match task.status {
                Status::Completed => completed.push(task),
                Status::Pending | Status::Waiting | Status::Deleted => pending.push(task),
            }
        }
    }

    store.save_pending(&pending)?;
    store.save_completed(&completed)?;

    println!("Imported tasks.");
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
        "Implemented commands: add, append, prepend, list/next, info, modify, done, delete, export, import, projects, tags"
    );
    Ok(())
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
        return Err(anyhow!("add: description is required"));
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
                if *dt > now {
                    task.status = Status::Waiting;
                } else if task.status == Status::Waiting {
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
