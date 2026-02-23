#[derive(Debug, Clone, Deserialize)]
pub struct ConfigEntryUpdateArg {
  pub section: String,
  pub key:     String,
  pub value:   serde_json::Value
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigApplyArg {
  pub updates: Vec<ConfigEntryUpdateArg>
}

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
    let mut cursor =
      Some(cwd.as_path());
    while let Some(path) = cursor {
      candidates.push(path.join(rel_path));
      cursor = path.parent();
    }
  }

  candidates.push(
    std::path::PathBuf::from(rel_path)
  );

  let mut unique =
    Vec::<std::path::PathBuf>::new();
  for candidate in candidates {
    if unique
      .iter()
      .any(|existing| {
        existing == &candidate
      })
    {
      continue;
    }
    unique.push(candidate);
  }
  unique
}

fn resolve_config_path(
  rel_path: &str
) -> std::path::PathBuf {
  let paths =
    candidate_config_paths(rel_path);
  if let Some(found) =
    paths.iter().find(|path| {
      path.is_file()
    })
  {
    return found.clone();
  }
  if let Some(first) = paths.first() {
    return first.clone();
  }
  std::path::PathBuf::from(rel_path)
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

fn ensure_valid_config_identifier(
  label: &str,
  value: &str
) -> anyhow::Result<()> {
  let trimmed = value.trim();
  if trimmed.is_empty() {
    anyhow::bail!(
      "{label} cannot be empty"
    );
  }
  if !trimmed
    .chars()
    .all(|ch| {
      ch.is_ascii_alphanumeric()
        || ch == '_'
        || ch == '-'
        || ch == '.'
    })
  {
    anyhow::bail!(
      "invalid {label}: {trimmed}"
    );
  }
  Ok(())
}

fn json_to_toml_literal(
  value: &serde_json::Value
) -> anyhow::Result<String> {
  if let Some(boolean) =
    value.as_bool()
  {
    return Ok(if boolean {
      "true".to_string()
    } else {
      "false".to_string()
    });
  }

  if let Some(integer) =
    value.as_i64()
  {
    return Ok(integer.to_string());
  }

  if let Some(number) =
    value.as_f64()
  {
    return Ok(number.to_string());
  }

  if let Some(text) =
    value.as_str()
  {
    let escaped = text
      .replace('\\', "\\\\")
      .replace('"', "\\\"");
    return Ok(format!(
      "\"{escaped}\""
    ));
  }

  anyhow::bail!(
    "unsupported config value type: {}",
    value
  )
}

fn section_name_from_header(
  line: &str
) -> Option<&str> {
  let trimmed = line.trim();
  if !trimmed.starts_with('[')
    || !trimmed.ends_with(']')
  {
    return None;
  }
  let inner =
    &trimmed[1..trimmed.len() - 1];
  Some(inner.trim())
}

fn line_key_name(
  line: &str
) -> Option<String> {
  let trimmed = line.trim();
  if trimmed.starts_with('#')
    || trimmed.is_empty()
  {
    return None;
  }
  let (key, _rest) =
    trimmed.split_once('=')?;
  let normalized =
    key.trim().to_string();
  if normalized.is_empty() {
    return None;
  }
  Some(normalized)
}

fn apply_single_update(
  raw: &str,
  section: &str,
  key: &str,
  literal: &str
) -> String {
  let mut lines = raw
    .lines()
    .map(ToString::to_string)
    .collect::<Vec<_>>();

  let section_start = lines
    .iter()
    .position(|line| {
      section_name_from_header(line)
        == Some(section)
    });

  let assignment =
    format!("{key} = {literal}");

  if let Some(start_index) =
    section_start
  {
    let section_end = lines
      .iter()
      .enumerate()
      .skip(start_index + 1)
      .find_map(|(index, line)| {
        if section_name_from_header(line)
          .is_some()
        {
          Some(index)
        } else {
          None
        }
      })
      .unwrap_or(lines.len());

    if let Some(existing_index) = lines
      .iter()
      .enumerate()
      .skip(start_index + 1)
      .take(section_end - start_index - 1)
      .find_map(|(index, line)| {
        if line_key_name(line)
          .as_deref()
          == Some(key)
        {
          Some(index)
        } else {
          None
        }
      })
    {
      lines[existing_index] =
        assignment;
    } else {
      lines.insert(section_end, assignment);
    }
  } else {
    if !lines.is_empty()
      && lines.last().is_some_and(
        |line| !line.trim().is_empty()
      )
    {
      lines.push(String::new());
    }
    lines.push(format!(
      "[{section}]"
    ));
    lines.push(assignment);
  }

  let mut output = lines.join("\n");
  if raw.ends_with('\n') {
    output.push('\n');
  }
  output
}

fn write_toml_updates(
  rel_path: &str,
  updates: &[ConfigEntryUpdateArg]
) -> anyhow::Result<
  serde_json::Value
> {
  if updates.is_empty() {
    return read_toml_snapshot(rel_path);
  }

  for update in updates {
    ensure_valid_config_identifier(
      "section",
      &update.section,
    )?;
    ensure_valid_config_identifier(
      "key",
      &update.key,
    )?;
  }

  let path =
    resolve_config_path(rel_path);
  if let Some(parent) = path.parent()
    && !parent.as_os_str().is_empty()
  {
    std::fs::create_dir_all(parent)
      .map_err(anyhow::Error::new)
      .with_context(|| {
        format!(
          "failed to create config directory {}",
          parent.display()
        )
      })?;
  }

  let mut raw =
    if path.exists() {
      std::fs::read_to_string(&path)
        .map_err(anyhow::Error::new)
        .with_context(|| {
          format!(
            "failed to read config file {}",
            path.display()
          )
        })?
    } else {
      String::new()
    };

  for update in updates {
    let literal = json_to_toml_literal(
      &update.value
    )?;
    raw = apply_single_update(
      &raw,
      update.section.trim(),
      update.key.trim(),
      literal.as_str(),
    );
  }

  std::fs::write(&path, raw)
    .map_err(anyhow::Error::new)
    .with_context(|| {
      format!(
        "failed to write config file {}",
        path.display()
      )
    })?;

  read_toml_snapshot(rel_path)
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

#[tauri::command]
#[instrument(skip(args), fields(request_id = ?request_id, update_count = args.updates.len()))]
pub async fn config_apply_updates(
  args: ConfigApplyArg,
  request_id: Option<String>
) -> Result<serde_json::Value, String>
{
  tracing::info!(
    request_id = ?request_id,
    updates = args.updates.len(),
    "config_apply_updates command invoked"
  );
  write_toml_updates(
    "rivet.toml",
    &args.updates,
  )
  .map_err(err_to_string)
}
