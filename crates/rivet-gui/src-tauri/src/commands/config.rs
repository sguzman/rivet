fn candidate_config_paths(
  rel_path: &str
) -> Vec<std::path::PathBuf> {
  let mut candidates =
    Vec::<std::path::PathBuf>::new();

  if let Ok(path) = std::env::var(
    "RIVET_CONFIG"
  ) {
    let trimmed = path.trim();
    if !trimmed.is_empty() {
      candidates
        .push(std::path::PathBuf::from(
          trimmed
        ));
    }
  }

  if let Ok(cwd) = std::env::current_dir()
  {
    candidates.push(cwd.join(rel_path));
    if let Some(parent) = cwd.parent() {
      candidates.push(parent.join(rel_path));
    }
  }

  candidates.push(
    std::path::PathBuf::from(rel_path)
  );
  candidates
}

fn read_toml_snapshot(
  rel_path: &str
) -> anyhow::Result<serde_json::Value> {
  let paths =
    candidate_config_paths(rel_path);
  let Some(found_path) =
    paths.into_iter().find(|path| {
      path.is_file()
    })
  else {
    anyhow::bail!(
      "config file not found for {}",
      rel_path
    );
  };

  tracing::debug!(
    path = %found_path.display(),
    "reading TOML snapshot"
  );

  let raw = std::fs::read_to_string(
    &found_path
  )
  .map_err(anyhow::Error::new)
  .with_context(|| {
    format!(
      "failed to read {}",
      found_path.display()
    )
  })?;

  let value = toml::from_str::<
    toml::Value,
  >(&raw)
  .map_err(anyhow::Error::new)
  .with_context(|| {
    format!(
      "failed to parse TOML {}",
      found_path.display()
    )
  })?;

  serde_json::to_value(value)
    .map_err(anyhow::Error::new)
    .with_context(|| {
      format!(
        "failed to convert TOML to JSON {}",
        found_path.display()
      )
    })
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id))]
pub async fn config_snapshot(
  request_id: Option<String>
) -> Result<serde_json::Value, String>
{
  tracing::info!(request_id = ?request_id, "config_snapshot command invoked");
  read_toml_snapshot("rivet.toml")
    .map_err(err_to_string)
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id))]
pub async fn tag_schema_snapshot(
  request_id: Option<String>
) -> Result<serde_json::Value, String>
{
  tracing::info!(request_id = ?request_id, "tag_schema_snapshot command invoked");
  read_toml_snapshot(
    "crates/rivet-gui/ui/assets/tags.toml",
  )
  .map_err(err_to_string)
}
