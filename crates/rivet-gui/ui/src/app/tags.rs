fn load_tag_schema() -> TagSchema {
  match toml::from_str::<TagSchema>(
    TAG_SCHEMA_TOML
  ) {
    | Ok(schema)
      if !schema.keys.is_empty() =>
    {
      tracing::info!(
        version = schema.version,
        key_count = schema.keys.len(),
        "loaded tag schema"
      );
      schema
    }
    | Ok(_) => {
      tracing::warn!(
        "tag schema was empty; using \
         fallback schema"
      );
      TagSchema::default()
    }
    | Err(error) => {
      tracing::error!(%error, "failed to parse tag schema; using fallback schema");
      TagSchema::default()
    }
  }
}

fn kanban_columns_from_schema(
  schema: &TagSchema
) -> Vec<String> {
  let values = schema
    .key(KANBAN_TAG_KEY)
    .map(|entry| {
      entry
        .values
        .iter()
        .map(|value| {
          value.trim().to_string()
        })
        .filter(|value| {
          !value.is_empty()
        })
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();

  if values.is_empty() {
    vec![
      "todo".to_string(),
      "working".to_string(),
      "finished".to_string(),
    ]
  } else {
    values
  }
}

fn optional_text(
  text: &str
) -> Option<String> {
  let trimmed = text.trim();
  if trimmed.is_empty() {
    None
  } else {
    Some(trimmed.to_string())
  }
}

fn split_tags(
  text: &str
) -> Vec<String> {
  text
    .split_whitespace()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(ToString::to_string)
    .collect()
}

fn recurrence_from_tags(
  tags: &[String]
) -> (
  String,
  String,
  Vec<String>,
  Vec<String>,
  String
) {
  let pattern = first_tag_value(
    tags,
    RECUR_TAG_KEY
  )
  .map(normalize_recurrence_pattern)
  .unwrap_or_else(|| {
    "none".to_string()
  });

  let time = first_tag_value(
    tags,
    RECUR_TIME_TAG_KEY
  )
  .unwrap_or_default()
  .to_string();

  let days = split_csv_values(
    first_tag_value(
      tags,
      RECUR_DAYS_TAG_KEY
    )
    .unwrap_or_default()
  )
  .into_iter()
  .map(|entry| {
    entry
      .to_ascii_lowercase()
      .trim()
      .to_string()
  })
  .filter(|entry| {
    WEEKDAY_KEYS
      .iter()
      .any(|key| key == entry)
  })
  .collect::<Vec<_>>();

  let months = split_csv_values(
    first_tag_value(
      tags,
      RECUR_MONTHS_TAG_KEY
    )
    .unwrap_or_default()
  )
  .into_iter()
  .map(|entry| {
    entry
      .to_ascii_lowercase()
      .trim()
      .to_string()
  })
  .filter(|entry| {
    MONTH_KEYS
      .iter()
      .any(|key| key == entry)
  })
  .collect::<Vec<_>>();

  let month_day = first_tag_value(
    tags,
    RECUR_MONTH_DAY_TAG_KEY
  )
  .unwrap_or_default()
  .to_string();

  (
    pattern, time, days, months,
    month_day
  )
}

fn normalize_recurrence_pattern(
  value: &str
) -> String {
  match value
    .trim()
    .to_ascii_lowercase()
    .as_str()
  {
    | "daily" => "daily".to_string(),
    | "weekly" => "weekly".to_string(),
    | "months" => "months".to_string(),
    | "monthly" => {
      "monthly".to_string()
    }
    | "yearly" => "yearly".to_string(),
    | _ => "none".to_string()
  }
}

fn split_csv_values(
  value: &str
) -> Vec<String> {
  value
    .split(',')
    .map(str::trim)
    .filter(|entry| !entry.is_empty())
    .map(ToString::to_string)
    .collect()
}

fn is_recurrence_tag(
  tag: &str
) -> bool {
  matches!(
    tag.split_once(':'),
    Some((key, _))
      if key == RECUR_TAG_KEY
      || key == RECUR_TIME_TAG_KEY
      || key == RECUR_DAYS_TAG_KEY
      || key == RECUR_MONTHS_TAG_KEY
      || key == RECUR_MONTH_DAY_TAG_KEY
  )
}

fn append_recurrence_tags(
  tags: &mut Vec<String>,
  state: &ModalState
) {
  remove_tags_for_key(
    tags,
    RECUR_TAG_KEY
  );
  remove_tags_for_key(
    tags,
    RECUR_TIME_TAG_KEY
  );
  remove_tags_for_key(
    tags,
    RECUR_DAYS_TAG_KEY
  );
  remove_tags_for_key(
    tags,
    RECUR_MONTHS_TAG_KEY
  );
  remove_tags_for_key(
    tags,
    RECUR_MONTH_DAY_TAG_KEY
  );

  let pattern =
    normalize_recurrence_pattern(
      &state.recurrence_pattern
    );
  if pattern == "none" {
    return;
  }

  push_tag_unique(
    tags,
    format!(
      "{RECUR_TAG_KEY}:{pattern}"
    )
  );

  let recurrence_time =
    state.recurrence_time.trim();
  if !recurrence_time.is_empty() {
    push_tag_unique(
      tags,
      format!(
        "{RECUR_TIME_TAG_KEY}:\
         {recurrence_time}"
      )
    );
  }

  if pattern == "weekly"
    && !state.recurrence_days.is_empty()
  {
    let values = state
      .recurrence_days
      .iter()
      .map(|value| {
        value
          .trim()
          .to_ascii_lowercase()
      })
      .filter(|value| {
        WEEKDAY_KEYS
          .iter()
          .any(|key| key == value)
      })
      .collect::<Vec<_>>();
    if !values.is_empty() {
      push_tag_unique(
        tags,
        format!(
          "{RECUR_DAYS_TAG_KEY}:{}",
          values.join(",")
        )
      );
    }
  }

  if pattern == "monthly"
    || pattern == "months"
    || pattern == "yearly"
  {
    let months = state
      .recurrence_months
      .iter()
      .map(|value| {
        value
          .trim()
          .to_ascii_lowercase()
      })
      .filter(|value| {
        MONTH_KEYS
          .iter()
          .any(|key| key == value)
      })
      .collect::<Vec<_>>();
    if !months.is_empty() {
      push_tag_unique(
        tags,
        format!(
          "{RECUR_MONTHS_TAG_KEY}:{}",
          months.join(",")
        )
      );
    }

    let month_day =
      state.recurrence_month_day.trim();
    if !month_day.is_empty() {
      push_tag_unique(
        tags,
        format!(
          "{RECUR_MONTH_DAY_TAG_KEY}:\
           {month_day}"
        )
      );
    }
  }
}

fn collect_tags_for_submit(
  state: &ModalState,
  board_tag: Option<String>,
  allow_recurrence: bool,
  ensure_kanban_lane: bool,
  default_kanban_lane: &str
) -> Vec<String> {
  let mut tags =
    state.draft_tags.clone();
  for tag in
    split_tags(&state.draft_custom_tag)
  {
    push_tag_unique(&mut tags, tag);
  }

  remove_tags_for_key(
    &mut tags,
    BOARD_TAG_KEY
  );
  if let Some(tag) = board_tag {
    push_tag_unique(&mut tags, tag);
  }

  if allow_recurrence {
    append_recurrence_tags(
      &mut tags, state
    );
  }

  if ensure_kanban_lane
    && !tags.iter().any(|tag| {
      tag.starts_with(&format!(
        "{KANBAN_TAG_KEY}:"
      ))
    })
  {
    push_tag_unique(
      &mut tags,
      format!(
        "{KANBAN_TAG_KEY}:{}",
        default_kanban_lane
      )
    );
  }

  tags
}

fn push_tag_unique(
  tags: &mut Vec<String>,
  tag: String
) -> bool {
  let trimmed = tag.trim();
  if trimmed.is_empty() {
    return false;
  }

  if tags
    .iter()
    .any(|existing| existing == trimmed)
  {
    return false;
  }

  tags.push(trimmed.to_string());
  true
}

fn first_value_for_key(
  schema: &TagSchema,
  key: &str
) -> Option<String> {
  schema.key(key).and_then(|entry| {
    entry.values.first().cloned()
  })
}

fn is_single_select_key(
  schema: &TagSchema,
  key: &str
) -> bool {
  schema
    .key(key)
    .and_then(|entry| {
      entry.selection.as_deref()
    })
    .is_some_and(|selection| {
      selection
        .eq_ignore_ascii_case("single")
    })
}

fn remove_tags_for_key(
  tags: &mut Vec<String>,
  key: &str
) {
  tags.retain(|existing| {
    match existing.split_once(':') {
      | Some((existing_key, _)) => {
        existing_key != key
      }
      | None => true
    }
  });
}

fn first_tag_value<'a>(
  tags: &'a [String],
  key: &str
) -> Option<&'a str> {
  tags.iter().find_map(|tag| {
    match tag.split_once(':') {
      | Some((existing_key, value))
        if existing_key == key =>
      {
        Some(value)
      }
      | _ => None
    }
  })
}

fn task_has_tag_value(
  tags: &[String],
  key: &str,
  value: &str
) -> bool {
  tags.iter().any(|tag| {
    matches!(
      tag.split_once(':'),
      Some((existing_key, existing_value))
        if existing_key == key
          && existing_value == value
    )
  })
}

fn remove_board_tag_for_id(
  tags: &mut Vec<String>,
  board_id: &str
) {
  tags.retain(|tag| {
    match tag.split_once(':') {
      | Some((key, value)) => {
        !(key == BOARD_TAG_KEY
          && value == board_id)
      }
      | None => true
    }
  });
}

fn tag_chip_style(
  schema: &TagSchema,
  tag: &str,
  tag_colors: &BTreeMap<String, String>
) -> String {
  let Some((key, _value)) =
    tag.split_once(':')
  else {
    return String::new();
  };

  let color = schema
    .key(key)
    .and_then(|entry| {
      entry.color.as_deref()
    })
    .map(ToString::to_string)
    .or_else(|| {
      tag_colors.get(key).cloned()
    })
    .unwrap_or_else(|| {
      deterministic_tag_key_color(key)
    });

  format!("--tag-key-color:{color};")
}

fn deterministic_tag_key_color(
  key: &str
) -> String {
  let mut hash: u32 = 0x811c9dc5;
  for byte in key.as_bytes() {
    hash ^= u32::from(*byte);
    hash = hash.wrapping_mul(16777619);
  }
  let hue = hash % 360;
  format!("hsl({hue} 72% 54%)")
}

fn tag_badge_style(
  tag: &str,
  tag_colors: &BTreeMap<String, String>
) -> String {
  let Some((key, _)) =
    tag.split_once(':')
  else {
    return String::new();
  };

  let color = tag_colors
    .get(key)
    .cloned()
    .unwrap_or_else(|| {
      deterministic_tag_key_color(key)
    });

  format!("--tag-key-color:{color};")
}

fn tag_color_for_schema_key(
  key: &TagKey
) -> Option<String> {
  if let Some(color) =
    key.color.as_ref()
  {
    return Some(color.clone());
  }
  if key.id.trim().is_empty() {
    return None;
  }
  Some(deterministic_tag_key_color(
    &key.id
  ))
}

fn build_tag_color_map(
  schema: &TagSchema
) -> BTreeMap<String, String> {
  schema
    .keys
    .iter()
    .filter_map(|key| {
      tag_color_for_schema_key(key).map(
        |color| (key.id.clone(), color)
      )
    })
    .collect()
}

#[cfg(test)]
mod tags_tests {
  use super::*;

  fn base_modal_state() -> ModalState {
    ModalState {
      mode:                 ModalMode::Add,
      draft_title:          "title".to_string(),
      draft_desc:           String::new(),
      draft_project:        String::new(),
      draft_board_id:       None,
      lock_board_selection: false,
      draft_custom_tag:     String::new(),
      draft_tags:           vec![],
      picker_key:           None,
      picker_value:         None,
      draft_due:            String::new(),
      recurrence_pattern:   "none"
        .to_string(),
      recurrence_time:      String::new(),
      recurrence_days:      vec![],
      recurrence_months:    vec![],
      recurrence_month_day: String::new(),
      error:                None
    }
  }

  #[test]
  fn recurrence_tags_round_trip_weekly()
  {
    let mut state = base_modal_state();
    state.recurrence_pattern =
      "weekly".to_string();
    state.recurrence_time =
      "15:23".to_string();
    state.recurrence_days = vec![
      "mon".to_string(),
      "wed".to_string(),
      "fri".to_string(),
    ];

    let mut tags = vec![
      "area:software".to_string()
    ];
    append_recurrence_tags(
      &mut tags, &state
    );

    let (
      pattern,
      time,
      days,
      months,
      month_day,
    ) = recurrence_from_tags(&tags);
    assert_eq!(pattern, "weekly");
    assert_eq!(time, "15:23");
    assert_eq!(
      days,
      vec![
        "mon".to_string(),
        "wed".to_string(),
        "fri".to_string()
      ]
    );
    assert!(months.is_empty());
    assert!(month_day.is_empty());
  }

  #[test]
  fn collect_tags_for_submit_overwrites_board_and_preserves_existing_lane(
  ) {
    let mut state = base_modal_state();
    state.draft_tags = vec![
      "area:software".to_string(),
      "board:old".to_string(),
      "kanban:working".to_string(),
    ];
    state.draft_custom_tag =
      "topic:rust urgent".to_string();

    let tags = collect_tags_for_submit(
      &state,
      Some("board:ops".to_string()),
      true,
      true,
      "todo",
    );

    assert!(task_has_tag_value(
      &tags, BOARD_TAG_KEY, "ops"
    ));
    assert!(!task_has_tag_value(
      &tags, BOARD_TAG_KEY, "old"
    ));
    assert!(task_has_tag_value(
      &tags, KANBAN_TAG_KEY, "working"
    ));
    assert!(tags.iter().any(|tag| {
      tag == "topic:rust"
    }));
    assert!(tags.iter().any(|tag| {
      tag == "urgent"
    }));
  }

  #[test]
  fn collect_tags_for_submit_adds_default_lane_when_missing(
  ) {
    let state = base_modal_state();
    let tags = collect_tags_for_submit(
      &state, None, true, true,
      "todo"
    );
    assert!(task_has_tag_value(
      &tags, KANBAN_TAG_KEY, "todo"
    ));
  }
}
