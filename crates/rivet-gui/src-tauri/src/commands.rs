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
#[instrument(skip(state), fields(description_len = create.description.len()))]
pub async fn task_add(state: State<'_, AppState>, create: TaskCreate) -> Result<TaskDto, String> {
    info!(
        description_len = create.description.len(),
        has_project = create.project.is_some(),
        tag_count = create.tags.len(),
        has_due = create.due.is_some(),
        "task_add command invoked"
    );
    let result = state.add(create);
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
#[instrument(skip(state), fields(uuid = %arg.uuid))]
pub async fn task_done(state: State<'_, AppState>, arg: TaskIdArg) -> Result<TaskDto, String> {
    info!(uuid = %arg.uuid, "task_done command invoked");
    let result = state.done(arg.uuid);
    if let Err(err) = result.as_ref() {
        error!(error = %err, "task_done command failed");
    }
    result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(skip(state), fields(uuid = %arg.uuid))]
pub async fn task_delete(state: State<'_, AppState>, arg: TaskIdArg) -> Result<(), String> {
    info!(uuid = %arg.uuid, "task_delete command invoked");
    let result = state.delete(arg.uuid);
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
#[instrument(fields(event = %arg.event))]
pub async fn ui_log(arg: UiLogArg) -> Result<(), String> {
    info!(event = %arg.event, detail = %arg.detail, "ui interaction");
    Ok(())
}
