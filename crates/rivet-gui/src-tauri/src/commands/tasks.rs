#[tauri::command]
#[instrument(skip(state), fields(request_id = ?request_id, status = ?args.status, project = ?args.project, tag = ?args.tag))]
pub async fn tasks_list(
  state: State<'_, AppState>,
  args: TasksListArgs,
  request_id: Option<String>
) -> Result<Vec<TaskDto>, String> {
  info!(
      request_id = ?request_id,
      status = ?args.status,
      project = ?args.project,
      tag = ?args.tag,
      query = ?args.query,
      "tasks_list command invoked"
  );
  let result = state.list(args);
  if let Err(err) = result.as_ref() {
    error!(request_id = ?request_id, error = %err, "tasks_list command failed");
  }
  result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(skip(state), fields(request_id = ?request_id, title_len = args.title.len(), description_len = args.description.len()))]
pub async fn task_add(
  state: State<'_, AppState>,
  args: TaskCreate,
  request_id: Option<String>
) -> Result<TaskDto, String> {
  info!(
    request_id = ?request_id,
    title_len = args.title.len(),
    description_len =
      args.description.len(),
    has_project =
      args.project.is_some(),
    tag_count = args.tags.len(),
    has_due = args.due.is_some(),
    "task_add command invoked"
  );
  let result = state.add(args);
  if let Err(err) = result.as_ref() {
    error!(request_id = ?request_id, error = %err, "task_add command failed");
  }
  result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(skip(state), fields(request_id = ?request_id, uuid = %args.uuid))]
pub async fn task_update(
  state: State<'_, AppState>,
  args: TaskUpdateArgs,
  request_id: Option<String>
) -> Result<TaskDto, String> {
  info!(request_id = ?request_id, uuid = %args.uuid, "task_update command invoked");
  let result = state.update(args);
  if let Err(err) = result.as_ref() {
    error!(request_id = ?request_id, error = %err, "task_update command failed");
  }
  result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(skip(state), fields(request_id = ?request_id, uuid = %args.uuid))]
pub async fn task_done(
  state: State<'_, AppState>,
  args: TaskIdArg,
  request_id: Option<String>
) -> Result<TaskDto, String> {
  info!(request_id = ?request_id, uuid = %args.uuid, "task_done command invoked");
  let result = state.done(args.uuid);
  if let Err(err) = result.as_ref() {
    error!(request_id = ?request_id, error = %err, "task_done command failed");
  }
  result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(skip(state), fields(request_id = ?request_id, uuid = %args.uuid))]
pub async fn task_uncomplete(
  state: State<'_, AppState>,
  args: TaskIdArg,
  request_id: Option<String>
) -> Result<TaskDto, String> {
  info!(request_id = ?request_id, uuid = %args.uuid, "task_uncomplete command invoked");
  let result = state.uncomplete(args.uuid);
  if let Err(err) = result.as_ref() {
    error!(request_id = ?request_id, error = %err, "task_uncomplete command failed");
  }
  result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(skip(state), fields(request_id = ?request_id, uuid = %args.uuid))]
pub async fn task_delete(
  state: State<'_, AppState>,
  args: TaskIdArg,
  request_id: Option<String>
) -> Result<(), String> {
  info!(request_id = ?request_id, uuid = %args.uuid, "task_delete command invoked");
  let result = state.delete(args.uuid);
  if let Err(err) = result.as_ref() {
    error!(request_id = ?request_id, error = %err, "task_delete command failed");
  }
  result.map_err(err_to_string)
}

#[derive(Debug, Deserialize)]
pub struct UiLogArg {
  pub event:  String,
  pub detail: String
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id, event = %args.event))]
pub async fn ui_log(
  args: UiLogArg,
  request_id: Option<String>
) -> Result<(), String> {
  info!(request_id = ?request_id, event = %args.event, detail = %args.detail, "ui interaction");
  Ok(())
}
