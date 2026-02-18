fn load_theme_mode() -> ThemeMode {
  let stored = web_sys::window()
    .and_then(|window| {
      window
        .local_storage()
        .ok()
        .flatten()
    })
    .and_then(|storage| {
      storage
        .get_item(THEME_STORAGE_KEY)
        .ok()
        .flatten()
    });

  match stored.as_deref() {
    | Some("night") => ThemeMode::Night,
    | _ => ThemeMode::Day
  }
}

fn save_theme_mode(theme: ThemeMode) {
  if let Some(storage) =
    web_sys::window().and_then(
      |window| {
        window
          .local_storage()
          .ok()
          .flatten()
      }
    )
  {
    let _ = storage.set_item(
      THEME_STORAGE_KEY,
      theme.storage_value()
    );
  }
}

fn load_workspace_tab() -> String {
  let stored = web_sys::window()
    .and_then(|window| {
      window
        .local_storage()
        .ok()
        .flatten()
    })
    .and_then(|storage| {
      storage
        .get_item(
          WORKSPACE_TAB_STORAGE_KEY
        )
        .ok()
        .flatten()
    });

  match stored.as_deref() {
    | Some("kanban") => {
      "kanban".to_string()
    }
    | Some("calendar") => {
      "calendar".to_string()
    }
    | _ => "tasks".to_string()
  }
}

fn load_due_notification_config()
-> DueNotificationConfig {
  let stored = web_sys::window()
    .and_then(|window| {
      window
        .local_storage()
        .ok()
        .flatten()
    })
    .and_then(|storage| {
      storage
        .get_item(
          DUE_NOTIFICATION_SETTINGS_STORAGE_KEY
        )
        .ok()
        .flatten()
    });

  if let Some(raw) = stored {
    match serde_json::from_str::<
      DueNotificationConfig
    >(&raw)
    {
      | Ok(mut config) => {
        if config.pre_notify_minutes
          == 0
        {
          config.pre_notify_minutes =
            15;
        }
        config.pre_notify_minutes =
          config.pre_notify_minutes
            .min(43_200);
        return config;
      }
      | Err(error) => {
        tracing::error!(
          %error,
          "failed parsing due \
           notification config \
           from local storage"
        );
      }
    }
  }

  DueNotificationConfig::default()
}

fn save_due_notification_config(
  config: &DueNotificationConfig
) {
  if let Some(storage) =
    web_sys::window().and_then(
      |window| {
        window
          .local_storage()
          .ok()
          .flatten()
      }
    )
    && let Ok(json) =
      serde_json::to_string(config)
  {
    let _ = storage.set_item(
      DUE_NOTIFICATION_SETTINGS_STORAGE_KEY,
      &json
    );
  }
}

fn load_due_notification_sent()
-> BTreeSet<String> {
  let stored = web_sys::window()
    .and_then(|window| {
      window
        .local_storage()
        .ok()
        .flatten()
    })
    .and_then(|storage| {
      storage
        .get_item(
          DUE_NOTIFICATION_SENT_STORAGE_KEY
        )
        .ok()
        .flatten()
    });

  if let Some(raw) = stored {
    match serde_json::from_str::<
      BTreeSet<String>
    >(&raw)
    {
      | Ok(values) => return values,
      | Err(error) => {
        tracing::error!(
          %error,
          "failed parsing due \
           notification registry \
           from local storage"
        );
      }
    }
  }

  BTreeSet::new()
}

fn save_due_notification_sent(
  sent: &BTreeSet<String>
) {
  if let Some(storage) =
    web_sys::window().and_then(
      |window| {
        window
          .local_storage()
          .ok()
          .flatten()
      }
    )
    && let Ok(json) =
      serde_json::to_string(sent)
  {
    let _ = storage.set_item(
      DUE_NOTIFICATION_SENT_STORAGE_KEY,
      &json
    );
  }
}

fn save_workspace_tab(tab: &str) {
  if let Some(storage) =
    web_sys::window().and_then(
      |window| {
        window
          .local_storage()
          .ok()
          .flatten()
      }
    )
  {
    let _ = storage.set_item(
      WORKSPACE_TAB_STORAGE_KEY,
      tab
    );
  }
}

fn load_external_calendars()
-> Vec<ExternalCalendarSource> {
  let stored = web_sys::window()
    .and_then(|window| {
      window
        .local_storage()
        .ok()
        .flatten()
    })
    .and_then(|storage| {
      storage
        .get_item(
          EXTERNAL_CALENDARS_STORAGE_KEY
        )
        .ok()
        .flatten()
    });

  if let Some(raw) = stored {
    match serde_json::from_str::<
      Vec<ExternalCalendarSource>
    >(&raw)
    {
      | Ok(mut sources) => {
        sources.retain(|source| {
          !source.id.trim().is_empty()
            && !source
              .name
              .trim()
              .is_empty()
            && !source
              .location
              .trim()
              .is_empty()
        });
        for source in &mut sources {
          if source
            .location
            .trim()
            .to_ascii_lowercase()
            .starts_with("file://")
          {
            source.imported_ics_file =
              true;
            if source
              .refresh_minutes
              > 0
            {
              source
                .refresh_minutes = 0;
            }
          }
        }
        assign_unique_external_calendar_colors(
          &mut sources
        );
        return sources;
      }
      | Err(error) => {
        tracing::error!(
          %error,
          "failed parsing external \
           calendars from local storage"
        );
      }
    }
  }

  Vec::new()
}

fn save_external_calendars(
  sources: &[ExternalCalendarSource]
) {
  if let Some(storage) =
    web_sys::window().and_then(
      |window| {
        window
          .local_storage()
          .ok()
          .flatten()
      }
    )
    && let Ok(json) =
      serde_json::to_string(sources)
  {
    let _ = storage.set_item(
      EXTERNAL_CALENDARS_STORAGE_KEY,
      &json
    );
  }
}

fn new_external_calendar_source()
-> ExternalCalendarSource {
  ExternalCalendarSource {
    id:              Uuid::new_v4()
      .to_string(),
    name:            String::new(),
    color:           default_external_calendar_color(),
    location:        String::new(),
    refresh_minutes: 30,
    enabled:         true,
    imported_ics_file: false,
    read_only:       true,
    show_reminders:  true,
    offline_support: true
  }
}

fn load_kanban_boards()
-> Vec<KanbanBoardDef> {
  let stored = web_sys::window()
    .and_then(|window| {
      window
        .local_storage()
        .ok()
        .flatten()
    })
    .and_then(|storage| {
      storage
        .get_item(
          KANBAN_BOARDS_STORAGE_KEY
        )
        .ok()
        .flatten()
    });

  if let Some(raw) = stored {
    match serde_json::from_str::<
      Vec<KanbanBoardDef>
    >(&raw)
    {
      | Ok(mut boards) => {
        boards.retain(|board| {
          !board.id.trim().is_empty()
            && !board
              .name
              .trim()
              .is_empty()
        });
        assign_unique_board_colors(
          &mut boards
        );
        if !boards.is_empty() {
          return boards;
        }
      }
      | Err(err) => {
        tracing::error!(
          error = %err,
          "failed parsing kanban \
           boards from storage"
        );
      }
    }
  }

  vec![KanbanBoardDef {
    id:    Uuid::new_v4().to_string(),
    name:  "Main".to_string(),
    color: default_board_color()
  }]
}

fn save_kanban_boards(
  boards: &[KanbanBoardDef]
) {
  if let Some(storage) =
    web_sys::window().and_then(
      |window| {
        window
          .local_storage()
          .ok()
          .flatten()
      }
    )
    && let Ok(json) =
      serde_json::to_string(boards)
  {
    let _ = storage.set_item(
      KANBAN_BOARDS_STORAGE_KEY,
      &json
    );
  }
}

fn load_active_kanban_board(
  boards: &[KanbanBoardDef]
) -> Option<String> {
  let stored = web_sys::window()
    .and_then(|window| {
      window
        .local_storage()
        .ok()
        .flatten()
    })
    .and_then(|storage| {
      storage
        .get_item(
          KANBAN_ACTIVE_BOARD_STORAGE_KEY
        )
        .ok()
        .flatten()
    });

  if let Some(id) = stored
    && boards
      .iter()
      .any(|board| board.id == id)
  {
    return Some(id);
  }

  boards
    .first()
    .map(|board| board.id.clone())
}

fn save_active_kanban_board(
  board_id: Option<&str>
) {
  if let Some(storage) =
    web_sys::window().and_then(
      |window| {
        window
          .local_storage()
          .ok()
          .flatten()
      }
    )
  {
    match board_id {
      | Some(id) => {
        let _ = storage.set_item(
          KANBAN_ACTIVE_BOARD_STORAGE_KEY,
          id
        );
      }
      | None => {
        let _ = storage.remove_item(
          KANBAN_ACTIVE_BOARD_STORAGE_KEY
        );
      }
    }
  }
}

fn default_board_color() -> String {
  "hsl(212 74% 54%)".to_string()
}

fn board_color_candidate(
  seed: usize
) -> String {
  let hue =
    seed.saturating_mul(47) % 360;
  format!("hsl({hue} 74% 54%)")
}

fn next_board_color(
  boards: &[KanbanBoardDef]
) -> String {
  let mut used =
    BTreeSet::<String>::new();
  for board in boards {
    used.insert(
      board
        .color
        .trim()
        .to_ascii_lowercase()
    );
  }

  for offset in 0_usize..512_usize {
    let candidate =
      board_color_candidate(
        boards
          .len()
          .saturating_add(offset)
      );
    if !used.contains(
      &candidate.to_ascii_lowercase()
    ) {
      return candidate;
    }
  }

  default_board_color()
}

fn assign_unique_board_colors(
  boards: &mut [KanbanBoardDef]
) {
  let mut used =
    BTreeSet::<String>::new();
  for (index, board) in
    boards.iter_mut().enumerate()
  {
    let mut color =
      board.color.trim().to_string();
    if color.is_empty() {
      color =
        board_color_candidate(index);
    }

    let mut key =
      color.to_ascii_lowercase();
    if used.contains(&key) {
      for offset in 0_usize..512_usize {
        let candidate =
          board_color_candidate(
            index
              .saturating_add(offset)
          );
        let candidate_key = candidate
          .to_ascii_lowercase();
        if !used
          .contains(&candidate_key)
        {
          color = candidate;
          key = candidate_key;
          break;
        }
      }
    }

    board.color = color;
    used.insert(key);
  }
}

fn make_unique_board_name(
  boards: &[KanbanBoardDef],
  requested: &str
) -> String {
  make_unique_board_name_except(
    boards, requested, ""
  )
}

fn make_unique_board_name_except(
  boards: &[KanbanBoardDef],
  requested: &str,
  except_board_id: &str
) -> String {
  let base = requested.trim();
  if base.is_empty() {
    return "Board".to_string();
  }

  let mut candidate = base.to_string();
  let mut suffix = 2_u32;
  while boards.iter().any(|board| {
    board.id != except_board_id
      && board
        .name
        .eq_ignore_ascii_case(
          &candidate
        )
  }) {
    candidate =
      format!("{base} {suffix}");
    suffix = suffix.saturating_add(1);
  }

  candidate
}

fn board_id_from_task_tags(
  boards: &[KanbanBoardDef],
  tags: &[String]
) -> Option<String> {
  let board_id = first_tag_value(
    tags,
    BOARD_TAG_KEY
  )?
  .to_string();
  boards
    .iter()
    .find(|board| board.id == board_id)
    .map(|board| board.id.clone())
}

fn build_kanban_board_color_map(
  boards: &[KanbanBoardDef]
) -> BTreeMap<String, String> {
  boards
    .iter()
    .map(|board| {
      (
        board.id.clone(),
        normalize_marker_color(
          board.color.as_str()
        )
      )
    })
    .collect()
}

fn build_external_calendar_color_map(
  calendars: &[ExternalCalendarSource]
) -> BTreeMap<String, String> {
  calendars
    .iter()
    .map(|source| {
      (
        source.id.clone(),
        normalize_marker_color(
          source.color.as_str()
        )
      )
    })
    .collect()
}

fn default_external_calendar_color()
-> String {
  external_calendar_color_candidate(0)
}

fn external_calendar_color_candidate(
  seed: usize
) -> String {
  let hue =
    seed.saturating_mul(53)
      .saturating_add(12)
      % 360;
  hsl_to_hex_color(
    hue as f32,
    0.72,
    0.52
  )
}

fn hsl_to_hex_color(
  hue: f32,
  saturation: f32,
  lightness: f32
) -> String {
  let c = (1.0
    - (2.0 * lightness - 1.0).abs())
    * saturation;
  let h_prime = hue / 60.0;
  let x = c
    * (1.0
      - ((h_prime % 2.0) - 1.0).abs());

  let (r1, g1, b1) = if (0.0..1.0)
    .contains(&h_prime)
  {
    (c, x, 0.0)
  } else if (1.0..2.0)
    .contains(&h_prime)
  {
    (x, c, 0.0)
  } else if (2.0..3.0)
    .contains(&h_prime)
  {
    (0.0, c, x)
  } else if (3.0..4.0)
    .contains(&h_prime)
  {
    (0.0, x, c)
  } else if (4.0..5.0)
    .contains(&h_prime)
  {
    (x, 0.0, c)
  } else {
    (c, 0.0, x)
  };

  let m = lightness - c / 2.0;
  let to_u8 = |value: f32| -> u8 {
    ((value + m) * 255.0)
      .round()
      .clamp(0.0, 255.0)
      as u8
  };

  format!(
    "#{:02x}{:02x}{:02x}",
    to_u8(r1),
    to_u8(g1),
    to_u8(b1)
  )
}

fn next_external_calendar_color(
  sources: &[ExternalCalendarSource]
) -> String {
  let mut used =
    BTreeSet::<String>::new();
  for source in sources {
    let normalized =
      normalize_marker_color(
        source.color.as_str()
      );
    used.insert(
      normalized
        .trim()
        .to_ascii_lowercase()
    );
  }

  for offset in 0_usize..512_usize {
    let candidate =
      external_calendar_color_candidate(
        sources
          .len()
          .saturating_add(offset)
      );
    if !used.contains(
      &candidate.to_ascii_lowercase()
    ) {
      return candidate;
    }
  }

  default_external_calendar_color()
}

fn assign_unique_external_calendar_colors(
  sources: &mut [ExternalCalendarSource]
) {
  let mut used =
    BTreeSet::<String>::new();
  for (index, source) in
    sources.iter_mut().enumerate()
  {
    let mut color =
      source.color.trim().to_string();
    if color.is_empty() {
      color =
        external_calendar_color_candidate(
          index
        );
    }

    let mut key =
      normalize_marker_color(&color)
        .trim()
        .to_ascii_lowercase();
    if used.contains(&key) {
      for offset in 0_usize..512_usize {
        let candidate =
          external_calendar_color_candidate(
            index
              .saturating_add(offset)
          );
        let candidate_key = candidate
          .to_ascii_lowercase();
        if !used
          .contains(&candidate_key)
        {
          color = candidate;
          key = candidate_key;
          break;
        }
      }
    }

    source.color = color;
    used.insert(key);
  }
}

fn normalize_marker_color(
  value: &str
) -> String {
  let trimmed = value.trim();
  if trimmed.is_empty() {
    return CALENDAR_UNAFFILIATED_COLOR
      .to_string();
  }

  if let Some(hex) =
    normalize_hex_color(trimmed)
  {
    return hex;
  }

  trimmed.to_string()
}

fn normalize_hex_color(
  value: &str
) -> Option<String> {
  let raw = value
    .trim()
    .trim_start_matches('#');
  if raw.len() == 3
    && raw
      .chars()
      .all(|ch| ch.is_ascii_hexdigit())
  {
    let mut expanded =
      String::with_capacity(7);
    expanded.push('#');
    for ch in raw.chars() {
      expanded.push(ch);
      expanded.push(ch);
    }
    return Some(
      expanded.to_ascii_lowercase()
    );
  }

  if raw.len() == 6
    && raw
      .chars()
      .all(|ch| ch.is_ascii_hexdigit())
  {
    return Some(format!(
      "#{}",
      raw.to_ascii_lowercase()
    ));
  }

  None
}
