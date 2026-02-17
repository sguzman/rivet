use std::path::PathBuf;

use anyhow::Context;
use chrono::Utc;
use parking_lot::Mutex;
use rivet_core::datastore::DataStore;
use rivet_core::datetime::parse_date_expr;
use rivet_core::task::{Status, Task};
use rivet_gui_shared::{
    TaskCreate, TaskDto, TaskPatch, TaskPriority, TaskStatus, TaskUpdateArgs, TasksListArgs,
};
use tracing::{debug, instrument};
use uuid::Uuid;

const KANBAN_LANE_KEY: &str = "kanban";
const DEFAULT_KANBAN_LANE: &str = "todo";

pub struct AppState {
    store: Mutex<DataStore>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let data_dir = resolve_gui_data_dir();
        let store = DataStore::open(&data_dir)
            .with_context(|| format!("failed to open GUI datastore at {}", data_dir.display()))?;
        Ok(Self {
            store: Mutex::new(store),
        })
    }

    #[instrument(skip(self))]
    pub fn list(&self, args: TasksListArgs) -> anyhow::Result<Vec<TaskDto>> {
        let store = self.store.lock();
        let mut tasks = store.load_pending()?;
        tasks.extend(store.load_completed()?);
        let now = Utc::now();

        let filtered = tasks
            .into_iter()
            .filter(|task| {
                if let Some(status) = args.status.as_ref()
                    && task_status_for_view(task, now) != *status
                {
                    return false;
                }

                if let Some(project) = args.project.as_ref()
                    && task.project.as_deref() != Some(project.as_str())
                {
                    return false;
                }

                if let Some(tag) = args.tag.as_ref()
                    && !task.tags.iter().any(|t| t == tag)
                {
                    return false;
                }

                if let Some(query) = args.query.as_ref() {
                    let q = query.to_ascii_lowercase();
                    if !task.description.to_ascii_lowercase().contains(&q) {
                        return false;
                    }
                }

                true
            })
            .map(task_to_dto)
            .collect();

        Ok(filtered)
    }

    #[instrument(skip(self))]
    pub fn add(&self, create: TaskCreate) -> anyhow::Result<TaskDto> {
        let now = Utc::now();
        let store = self.store.lock();
        let mut pending = store.load_pending()?;
        let next_id = store.next_id(&pending);

        let mut task = Task::new_pending(create.description, now, next_id);
        task.project = create.project;
        task.tags = create.tags;
        ensure_default_kanban_lane_tag(&mut task.tags);
        task.priority = create.priority.map(priority_to_core);

        if let Some(due) = create.due {
            task.due = Some(parse_date_expr(&due, now)?);
        }
        if let Some(wait) = create.wait {
            let parsed = parse_date_expr(&wait, now)?;
            task.wait = Some(parsed);
        }
        if let Some(scheduled) = create.scheduled {
            task.scheduled = Some(parse_date_expr(&scheduled, now)?);
        }

        pending.push(task.clone());
        pending.sort_by_key(|t| t.id.unwrap_or(u64::MAX));
        store.save_pending(&pending)?;

        Ok(task_to_dto(task))
    }

    #[instrument(skip(self))]
    pub fn update(&self, update: TaskUpdateArgs) -> anyhow::Result<TaskDto> {
        let now = Utc::now();
        let store = self.store.lock();
        let mut pending = store.load_pending()?;

        let updated_task = {
            let task = pending
                .iter_mut()
                .find(|task| task.uuid == update.uuid)
                .ok_or_else(|| anyhow::anyhow!("task not found"))?;

            apply_patch(task, update.patch, now)?;
            task.modified = now;
            task.clone()
        };

        store.save_pending(&pending)?;
        Ok(task_to_dto(updated_task))
    }

    #[instrument(skip(self))]
    pub fn done(&self, uuid: Uuid) -> anyhow::Result<TaskDto> {
        let now = Utc::now();
        let store = self.store.lock();
        let mut pending = store.load_pending()?;
        let mut completed = store.load_completed()?;

        let idx = pending
            .iter()
            .position(|task| task.uuid == uuid)
            .ok_or_else(|| anyhow::anyhow!("task not found"))?;

        let mut task = pending.remove(idx);
        task.status = Status::Completed;
        task.end = Some(now);
        task.modified = now;

        completed.push(task.clone());

        store.save_pending(&pending)?;
        store.save_completed(&completed)?;

        Ok(task_to_dto(task))
    }

    #[instrument(skip(self))]
    pub fn delete(&self, uuid: Uuid) -> anyhow::Result<()> {
        let now = Utc::now();
        let store = self.store.lock();
        let mut pending = store.load_pending()?;

        let task = pending
            .iter_mut()
            .find(|task| task.uuid == uuid)
            .ok_or_else(|| anyhow::anyhow!("task not found"))?;

        task.status = Status::Deleted;
        task.modified = now;

        store.save_pending(&pending)?;
        Ok(())
    }
}

fn resolve_gui_data_dir() -> PathBuf {
    if let Ok(path) = std::env::var("RIVET_GUI_DATA") {
        return PathBuf::from(path);
    }

    if let Ok(cwd) = std::env::current_dir() {
        return cwd.join(".rivet_gui_data");
    }

    PathBuf::from(".rivet_gui_data")
}

fn priority_to_core(priority: TaskPriority) -> String {
    match priority {
        TaskPriority::Low => "L".to_string(),
        TaskPriority::Medium => "M".to_string(),
        TaskPriority::High => "H".to_string(),
    }
}

fn priority_from_core(priority: Option<String>) -> Option<TaskPriority> {
    match priority.as_deref() {
        Some("L") | Some("low") => Some(TaskPriority::Low),
        Some("M") | Some("med") | Some("medium") => Some(TaskPriority::Medium),
        Some("H") | Some("high") => Some(TaskPriority::High),
        _ => None,
    }
}

fn map_status(status: Status) -> TaskStatus {
    match status {
        Status::Pending => TaskStatus::Pending,
        Status::Completed => TaskStatus::Completed,
        Status::Deleted => TaskStatus::Deleted,
        Status::Waiting => TaskStatus::Waiting,
    }
}

fn task_status_for_view(task: &Task, now: chrono::DateTime<Utc>) -> TaskStatus {
    if task.status == Status::Pending && task.wait.map(|wait| wait > now).unwrap_or(false) {
        return TaskStatus::Waiting;
    }
    map_status(task.status.clone())
}

fn task_to_dto(task: Task) -> TaskDto {
    let status = task_status_for_view(&task, Utc::now());

    TaskDto {
        uuid: task.uuid,
        id: task.id,
        description: task.description,
        status,
        project: task.project,
        tags: task.tags,
        priority: priority_from_core(task.priority),
        due: task.due.map(|d| d.format("%Y%m%dT%H%M%SZ").to_string()),
        wait: task.wait.map(|d| d.format("%Y%m%dT%H%M%SZ").to_string()),
        scheduled: task
            .scheduled
            .map(|d| d.format("%Y%m%dT%H%M%SZ").to_string()),
        created: Some(task.entry.format("%Y%m%dT%H%M%SZ").to_string()),
        modified: Some(task.modified.format("%Y%m%dT%H%M%SZ").to_string()),
    }
}

fn apply_patch(
    task: &mut Task,
    patch: TaskPatch,
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    if let Some(description) = patch.description {
        task.description = description;
    }
    if let Some(project) = patch.project {
        task.project = project;
    }
    if let Some(tags) = patch.tags {
        task.tags = tags;
    }
    if let Some(priority) = patch.priority {
        task.priority = priority.map(priority_to_core);
    }

    if let Some(due) = patch.due {
        task.due = due
            .as_deref()
            .map(|value| parse_date_expr(value, now))
            .transpose()?;
    }
    if let Some(wait) = patch.wait {
        task.wait = wait
            .as_deref()
            .map(|value| parse_date_expr(value, now))
            .transpose()?;
    }
    if let Some(scheduled) = patch.scheduled {
        task.scheduled = scheduled
            .as_deref()
            .map(|value| parse_date_expr(value, now))
            .transpose()?;
    }

    if task.wait.map(|wait| wait <= now).unwrap_or(false) && task.status == Status::Waiting {
        task.status = Status::Pending;
    }

    debug!(uuid = %task.uuid, id = ?task.id, "task patch applied");

    Ok(())
}

fn ensure_default_kanban_lane_tag(tags: &mut Vec<String>) {
    if tags.iter().any(|tag| {
        tag.split_once(':')
            .is_some_and(|(key, _)| key == KANBAN_LANE_KEY)
    }) {
        return;
    }

    tags.push(format!(
        "{KANBAN_LANE_KEY}:{DEFAULT_KANBAN_LANE}"
    ));
}
