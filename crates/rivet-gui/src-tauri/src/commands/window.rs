#[tauri::command]
pub async fn window_minimize(
  window: tauri::Window
) -> Result<(), String> {
  window
    .minimize()
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn window_toggle_maximize(
  window: tauri::Window
) -> Result<(), String> {
  let is_maximized = window
    .is_maximized()
    .map_err(|err| err.to_string())?;
  if is_maximized {
    window
      .unmaximize()
      .map_err(|err| err.to_string())
  } else {
    window
      .maximize()
      .map_err(|err| err.to_string())
  }
}

#[tauri::command]
pub async fn window_close(
  window: tauri::Window
) -> Result<(), String> {
  window
    .close()
    .map_err(|err| err.to_string())
}
