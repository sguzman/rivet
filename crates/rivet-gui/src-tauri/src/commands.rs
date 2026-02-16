use rivet_gui_shared::{TaskCreate, TaskDto, TaskIdArg, TaskUpdateArgs, TasksListArgs};
use tauri::State;
use tracing::instrument;

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
    state.list(args).map_err(err_to_string)
}

#[tauri::command]
#[instrument(skip(state), fields(description_len = create.description.len()))]
pub async fn task_add(state: State<'_, AppState>, create: TaskCreate) -> Result<TaskDto, String> {
    state.add(create).map_err(err_to_string)
}

#[tauri::command]
#[instrument(skip(state), fields(uuid = %args.uuid))]
pub async fn task_update(
    state: State<'_, AppState>,
    args: TaskUpdateArgs,
) -> Result<TaskDto, String> {
    state.update(args).map_err(err_to_string)
}

#[tauri::command]
#[instrument(skip(state), fields(uuid = %arg.uuid))]
pub async fn task_done(state: State<'_, AppState>, arg: TaskIdArg) -> Result<TaskDto, String> {
    state.done(arg.uuid).map_err(err_to_string)
}

#[tauri::command]
#[instrument(skip(state), fields(uuid = %arg.uuid))]
pub async fn task_delete(state: State<'_, AppState>, arg: TaskIdArg) -> Result<(), String> {
    state.delete(arg.uuid).map_err(err_to_string)
}
