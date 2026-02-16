use rivet_gui_shared::{TaskCreate, TaskDto, TaskIdArg, TaskUpdateArgs, TasksListArgs};
use serde::Deserialize;
use tauri::State;
use tracing::{error, info, instrument};

use crate::state::AppState;

fn err_to_string(err: anyhow::Error) -> String {
    err.to_string()
}

#[tauri::command]
#[instrument(skip(state), fields(status = ?args.status, project = ?args.project, tag = ?args.tag))]
pub async fn tasks_list(
    state: State<'_, AppState>,
    args: TasksListArgs,
) -> Result<Vec<TaskDto>, String> {
    info!(
        status = ?args.status,
        project = ?args.project,
        tag = ?args.tag,
        query = ?args.query,
        "tasks_list command invoked"
    );
    let result = state.list(args);
    if let Err(err) = result.as_ref() {
        error!(error = %err, "tasks_list command failed");
    }
    result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(skip(state), fields(description_len = args.description.len()))]
pub async fn task_add(state: State<'_, AppState>, args: TaskCreate) -> Result<TaskDto, String> {
    info!(
        description_len = args.description.len(),
        has_project = args.project.is_some(),
        tag_count = args.tags.len(),
        has_due = args.due.is_some(),
        "task_add command invoked"
    );
    let result = state.add(args);
    if let Err(err) = result.as_ref() {
        error!(error = %err, "task_add command failed");
    }
    result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(skip(state), fields(uuid = %args.uuid))]
pub async fn task_update(
    state: State<'_, AppState>,
    args: TaskUpdateArgs,
) -> Result<TaskDto, String> {
    info!(uuid = %args.uuid, "task_update command invoked");
    let result = state.update(args);
    if let Err(err) = result.as_ref() {
        error!(error = %err, "task_update command failed");
    }
    result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(skip(state), fields(uuid = %args.uuid))]
pub async fn task_done(state: State<'_, AppState>, args: TaskIdArg) -> Result<TaskDto, String> {
    info!(uuid = %args.uuid, "task_done command invoked");
    let result = state.done(args.uuid);
    if let Err(err) = result.as_ref() {
        error!(error = %err, "task_done command failed");
    }
    result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(skip(state), fields(uuid = %args.uuid))]
pub async fn task_delete(state: State<'_, AppState>, args: TaskIdArg) -> Result<(), String> {
    info!(uuid = %args.uuid, "task_delete command invoked");
    let result = state.delete(args.uuid);
    if let Err(err) = result.as_ref() {
        error!(error = %err, "task_delete command failed");
    }
    result.map_err(err_to_string)
}

#[derive(Debug, Deserialize)]
pub struct UiLogArg {
    pub event: String,
    pub detail: String,
}

#[tauri::command]
#[instrument(fields(event = %args.event))]
pub async fn ui_log(args: UiLogArg) -> Result<(), String> {
    info!(event = %args.event, detail = %args.detail, "ui interaction");
    Ok(())
}
