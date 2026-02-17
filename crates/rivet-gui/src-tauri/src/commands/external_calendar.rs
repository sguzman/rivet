#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ExternalCalendarSourceArg {
  pub id:              String,
  pub name:            String,
  pub color:           String,
  pub location:        String,
  pub refresh_minutes: u32,
  pub enabled:         bool,
  pub read_only:       bool,
  pub show_reminders:  bool,
  pub offline_support: bool
}

#[derive(Debug, Clone, Serialize)]
pub struct ExternalCalendarSyncResult {
  pub calendar_id:     String,
  pub created:         usize,
  pub updated:         usize,
  pub deleted:         usize,
  pub remote_events:   usize,
  pub refresh_minutes: u32
}

#[derive(Debug, Clone)]
struct ExternalCalendarEvent {
  uid:         String,
  title:       String,
  description: String,
  due_rfc3339: String,
  tags:        Vec<String>
}

#[tauri::command]
#[instrument(skip(state), fields(calendar_id = %args.id, name = %args.name, enabled = args.enabled))]
pub async fn external_calendar_sync(
  state: State<'_, AppState>,
  args: ExternalCalendarSourceArg
) -> Result<
  ExternalCalendarSyncResult,
  String
> {
  let calendar_id = args.id.clone();
  let refresh_minutes =
    args.refresh_minutes;

  if !args.enabled {
    return Ok(
      ExternalCalendarSyncResult {
        calendar_id,
        created: 0,
        updated: 0,
        deleted: 0,
        remote_events: 0,
        refresh_minutes
      }
    );
  }

  let ics_text = fetch_ics_document(
    args.location.as_str()
  )
  .await
  .map_err(err_to_string)?;
  let events =
    parse_ics_events(&ics_text, &args)
      .map_err(err_to_string)?;
  let remote_events = events.len();

  let mut created = 0_usize;
  let mut updated = 0_usize;
  let mut deleted = 0_usize;

  let all_tasks = state
    .list(TasksListArgs {
      query:   None,
      status:  None,
      project: None,
      tag:     None
    })
    .map_err(err_to_string)?;
  let mut existing_by_uid =
    BTreeMap::<String, TaskDto>::new();
  for task in all_tasks {
    if !task_has_tag_value(
      &task.tags,
      CAL_SOURCE_TAG_KEY,
      &normalize_tag_value(&args.id)
    ) {
      continue;
    }
    if task.status
      == TaskStatus::Deleted
    {
      continue;
    }
    if let Some(event_uid) =
      first_tag_value(
        &task.tags,
        CAL_EVENT_TAG_KEY
      )
    {
      existing_by_uid
        .insert(event_uid, task);
    }
  }

  let mut seen_uids = BTreeSet::new();
  for event in events {
    seen_uids.insert(event.uid.clone());
    match existing_by_uid
      .get(&event.uid)
    {
      | Some(existing)
        if matches!(
          existing.status,
          TaskStatus::Pending
            | TaskStatus::Waiting
        ) =>
      {
        let merged_tags =
          merge_calendar_tags(
            &existing.tags,
            &event.tags
          );
        let update = TaskUpdateArgs {
          uuid:  existing.uuid,
          patch: TaskPatch {
            title: Some(
              event.title.clone()
            ),
            description: Some(
              event.description.clone()
            ),
            project: Some(Some(
              format!(
                "calendar/{}",
                args.name.trim()
              )
            )),
            tags: Some(merged_tags),
            due: Some(Some(
              event.due_rfc3339.clone()
            )),
            wait: Some(None),
            scheduled: Some(None),
            ..TaskPatch::default()
          }
        };

        state
          .update(update)
          .map_err(err_to_string)?;
        updated =
          updated.saturating_add(1);
      }
      | Some(existing) => {
        info!(
          calendar_id = %args.id,
          event_uid = %event.uid,
          uuid = %existing.uuid,
          status = ?existing.status,
          "external calendar event already tracked by non-editable task status; skipping create/update"
        );
      }
      | None => {
        let create = TaskCreate {
          title:       event
            .title
            .clone(),
          description: event
            .description
            .clone(),
          project:     Some(format!(
            "calendar/{}",
            args.name.trim()
          )),
          tags:        event
            .tags
            .clone(),
          priority:    None,
          due:         Some(
            event.due_rfc3339.clone()
          ),
          wait:        None,
          scheduled:   None
        };

        state
          .add(create)
          .map_err(err_to_string)?;
        created =
          created.saturating_add(1);
      }
    }
  }

  for (uid, task) in existing_by_uid {
    if seen_uids.contains(&uid) {
      continue;
    }
    if task.status
      != TaskStatus::Deleted
    {
      state
        .delete(task.uuid)
        .map_err(err_to_string)?;
      deleted =
        deleted.saturating_add(1);
    }
  }

  info!(
    calendar_id = %args.id,
    created,
    updated,
    deleted,
    remote_events,
    "external calendar sync completed"
  );

  Ok(ExternalCalendarSyncResult {
    calendar_id: args.id,
    created,
    updated,
    deleted,
    remote_events,
    refresh_minutes: args
      .refresh_minutes
  })
}

async fn fetch_ics_document(
  location: &str
) -> anyhow::Result<String> {
  let trimmed = location.trim();
  if trimmed.is_empty() {
    anyhow::bail!(
      "calendar location URL is empty"
    );
  }

  let client =
    reqwest::Client::builder()
      .timeout(Duration::from_secs(30))
      .build()
      .context(
        "failed building HTTP client \
         for calendar sync"
      )?;

  let response = client
    .get(trimmed)
    .send()
    .await
    .with_context(|| {
      format!(
        "failed requesting calendar \
         URL: {trimmed}"
      )
    })?;

  let status = response.status();
  if !status.is_success() {
    anyhow::bail!(
      "calendar URL returned HTTP {}",
      status
    );
  }

  response.text().await.context(
    "failed reading calendar response \
     body"
  )
}

fn parse_ics_events(
  ics_text: &str,
  source: &ExternalCalendarSourceArg
) -> anyhow::Result<
  Vec<ExternalCalendarEvent>
> {
  let mut events = Vec::new();
  let reader =
    BufReader::new(ics_text.as_bytes());
  let parser = IcalParser::new(reader);

  for calendar in parser {
    let calendar = calendar.context(
      "failed parsing iCalendar \
       payload"
    )?;
    for event in calendar.events {
      if let Some(normalized) =
        normalize_ical_event(
          &event, source
        )
      {
        events.push(normalized);
      }
    }
  }

  Ok(events)
}

fn normalize_ical_event(
  event: &IcalEvent,
  source: &ExternalCalendarSourceArg
) -> Option<ExternalCalendarEvent> {
  let uid_raw = property_value(
    &event.properties,
    "UID"
  )?;
  let uid =
    normalize_tag_value(&uid_raw);
  let title = property_value(
    &event.properties,
    "SUMMARY"
  )
  .unwrap_or_else(|| {
    "Calendar Event".to_string()
  });

  let description = {
    let mut parts = Vec::new();
    if let Some(desc) = property_value(
      &event.properties,
      "DESCRIPTION"
    ) && !desc.trim().is_empty()
    {
      parts
        .push(desc.trim().to_string());
    }
    if let Some(location) =
      property_value(
        &event.properties,
        "LOCATION"
      )
      && !location.trim().is_empty()
    {
      parts.push(format!(
        "Location: {}",
        location.trim()
      ));
    }
    parts.join("\n")
  };

  let dtstart_prop = find_property(
    &event.properties,
    "DTSTART"
  )?;
  let due_utc =
    parse_ics_dtstart(dtstart_prop)?;

  let mut tags = vec![
    format!(
      "{CAL_SOURCE_TAG_KEY}:{}",
      normalize_tag_value(&source.id)
    ),
    format!(
      "{CAL_EVENT_TAG_KEY}:{uid}"
    ),
    format!(
      "{CAL_COLOR_TAG_KEY}:{}",
      normalize_tag_value(
        source
          .color
          .trim_start_matches('#')
      )
    ),
  ];

  if let Some(rrule) = property_value(
    &event.properties,
    "RRULE"
  ) {
    append_rrule_tags(
      &mut tags, &rrule, due_utc
    );
  }

  Some(ExternalCalendarEvent {
    uid,
    title,
    description,
    due_rfc3339: due_utc.to_rfc3339(),
    tags
  })
}

fn parse_ics_dtstart(
  property: &Property
) -> Option<DateTime<Utc>> {
  let raw =
    property.value.as_ref()?.trim();
  if raw.is_empty() {
    return None;
  }

  if let Ok(parsed) =
    DateTime::parse_from_rfc3339(raw)
  {
    return Some(
      parsed.with_timezone(&Utc)
    );
  }

  if raw.ends_with('Z')
    && let Ok(naive) =
      NaiveDateTime::parse_from_str(
        raw,
        "%Y%m%dT%H%M%SZ"
      )
  {
    return Some(
      DateTime::<Utc>::from_naive_utc_and_offset(
        naive, Utc
      )
    );
  }

  if raw.len() == 8
    && let Ok(date) =
      NaiveDate::parse_from_str(
        raw, "%Y%m%d"
      )
  {
    let naive =
      date.and_hms_opt(0, 0, 0)?;
    return local_naive_to_utc(
      timezone_from_property(property),
      naive
    );
  }

  if let Ok(naive) =
    NaiveDateTime::parse_from_str(
      raw,
      "%Y%m%dT%H%M%S"
    )
  {
    return local_naive_to_utc(
      timezone_from_property(property),
      naive
    );
  }

  None
}

fn timezone_from_property(
  property: &Property
) -> Tz {
  let Some(params) =
    property.params.as_ref()
  else {
    return *project_timezone();
  };
  for (key, values) in params {
    if key != "TZID" {
      continue;
    }
    let Some(value) = values.first()
    else {
      continue;
    };
    match value.trim().parse::<Tz>() {
      | Ok(tz) => return tz,
      | Err(error) => {
        warn!(
          tzid = %value,
          error = %error,
          "invalid TZID in ICS; using project timezone"
        );
      }
    }
  }
  *project_timezone()
}

fn local_naive_to_utc(
  timezone: Tz,
  naive: NaiveDateTime
) -> Option<DateTime<Utc>> {
  match timezone
    .from_local_datetime(&naive)
  {
    | LocalResult::Single(dt) => {
      Some(dt.with_timezone(&Utc))
    }
    | LocalResult::Ambiguous(
      first,
      second
    ) => {
      let chosen = if first <= second {
        first
      } else {
        second
      };
      Some(chosen.with_timezone(&Utc))
    }
    | LocalResult::None => None
  }
}

fn append_rrule_tags(
  tags: &mut Vec<String>,
  rrule: &str,
  due_utc: DateTime<Utc>
) {
  let rule_map = parse_rrule(rrule);
  let Some(freq) =
    rule_map.get("FREQ").cloned()
  else {
    return;
  };
  let frequency = match freq
    .to_ascii_lowercase()
    .as_str()
  {
    | "daily" => "daily",
    | "weekly" => "weekly",
    | "monthly" => "monthly",
    | "yearly" => "yearly",
    | _ => return
  };

  push_tag_unique(
    tags,
    format!(
      "{RECUR_TAG_KEY}:{frequency}"
    )
  );

  let local_due = due_utc
    .with_timezone(project_timezone());
  push_tag_unique(
    tags,
    format!(
      "{RECUR_TIME_TAG_KEY}:{}",
      local_due.format("%H:%M")
    )
  );

  if frequency == "weekly" {
    let days = rule_map
      .get("BYDAY")
      .map(|value| {
        value
          .split(',')
          .filter_map(rrule_day_to_key)
          .collect::<Vec<_>>()
      })
      .unwrap_or_else(|| {
        vec![weekday_to_key(
          local_due.weekday()
        )]
      });
    if !days.is_empty() {
      push_tag_unique(
        tags,
        format!(
          "{RECUR_DAYS_TAG_KEY}:{}",
          days.join(",")
        )
      );
    }
  }

  if frequency == "monthly"
    || frequency == "yearly"
  {
    if let Some(by_month) =
      rule_map.get("BYMONTH")
    {
      let months = by_month
        .split(',')
        .filter_map(|token| {
          token
            .trim()
            .parse::<u32>()
            .ok()
        })
        .filter_map(month_number_to_key)
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
    } else if frequency == "yearly" {
      let month_key =
        month_number_to_key(
          local_due.month()
        )
        .unwrap_or("jan");
      push_tag_unique(
        tags,
        format!(
          "{RECUR_MONTHS_TAG_KEY}:\
           {month_key}"
        )
      );
    }

    if let Some(by_month_day) =
      rule_map.get("BYMONTHDAY")
      && !by_month_day.trim().is_empty()
    {
      push_tag_unique(
        tags,
        format!(
          "{RECUR_MONTH_DAY_TAG_KEY}:\
           {}",
          by_month_day.trim()
        )
      );
    }
  }
}

fn parse_rrule(
  rrule: &str
) -> BTreeMap<String, String> {
  let mut values = BTreeMap::new();
  for token in rrule.split(';') {
    let Some((key, value)) =
      token.split_once('=')
    else {
      continue;
    };
    let key =
      key.trim().to_ascii_uppercase();
    let value = value.trim();
    if key.is_empty()
      || value.is_empty()
    {
      continue;
    }
    values
      .insert(key, value.to_string());
  }
  values
}

fn find_property<'a>(
  properties: &'a [Property],
  name: &str
) -> Option<&'a Property> {
  properties.iter().find(|property| {
    property.name == name
  })
}

fn property_value(
  properties: &[Property],
  name: &str
) -> Option<String> {
  find_property(properties, name)?
    .value
    .as_ref()
    .map(|value| {
      value.trim().to_string()
    })
}

fn normalize_tag_value(
  value: &str
) -> String {
  let mut out = String::new();
  for ch in value.chars() {
    if ch.is_ascii_alphanumeric()
      || ch == '-'
      || ch == '_'
      || ch == '.'
    {
      out.push(ch);
    } else {
      out.push('_');
    }
  }
  let collapsed = out
    .split('_')
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join("_");
  if collapsed.is_empty() {
    "value".to_string()
  } else {
    collapsed
  }
}

fn first_tag_value(
  tags: &[String],
  key: &str
) -> Option<String> {
  tags.iter().find_map(|tag| {
    let (tag_key, value) =
      tag.split_once(':')?;
    if tag_key == key {
      Some(value.to_string())
    } else {
      None
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
      Some((tag_key, tag_value))
        if tag_key == key
          && tag_value == value
    )
  })
}

fn merge_calendar_tags(
  existing: &[String],
  managed: &[String]
) -> Vec<String> {
  let mut tags = existing
    .iter()
    .filter(|tag| {
      !is_calendar_managed_tag(tag)
    })
    .cloned()
    .collect::<Vec<_>>();
  for tag in managed {
    push_tag_unique(
      &mut tags,
      tag.clone()
    );
  }
  tags
}

fn is_calendar_managed_tag(
  tag: &str
) -> bool {
  matches!(
    tag.split_once(':'),
    Some((key, _))
      if key == CAL_SOURCE_TAG_KEY
      || key == CAL_EVENT_TAG_KEY
      || key == CAL_COLOR_TAG_KEY
      || key == RECUR_TAG_KEY
      || key == RECUR_TIME_TAG_KEY
      || key == RECUR_DAYS_TAG_KEY
      || key == RECUR_MONTHS_TAG_KEY
      || key == RECUR_MONTH_DAY_TAG_KEY
  )
}

fn push_tag_unique(
  tags: &mut Vec<String>,
  tag: String
) {
  if !tags
    .iter()
    .any(|existing| existing == &tag)
  {
    tags.push(tag);
  }
}

fn month_number_to_key(
  month: u32
) -> Option<&'static str> {
  match month {
    | 1 => Some("jan"),
    | 2 => Some("feb"),
    | 3 => Some("mar"),
    | 4 => Some("apr"),
    | 5 => Some("may"),
    | 6 => Some("jun"),
    | 7 => Some("jul"),
    | 8 => Some("aug"),
    | 9 => Some("sep"),
    | 10 => Some("oct"),
    | 11 => Some("nov"),
    | 12 => Some("dec"),
    | _ => None
  }
}

fn rrule_day_to_key(
  token: &str
) -> Option<&'static str> {
  match token.trim() {
    | "MO" => Some("mon"),
    | "TU" => Some("tue"),
    | "WE" => Some("wed"),
    | "TH" => Some("thu"),
    | "FR" => Some("fri"),
    | "SA" => Some("sat"),
    | "SU" => Some("sun"),
    | _ => None
  }
}

fn weekday_to_key(
  day: chrono::Weekday
) -> &'static str {
  match day {
    | chrono::Weekday::Mon => "mon",
    | chrono::Weekday::Tue => "tue",
    | chrono::Weekday::Wed => "wed",
    | chrono::Weekday::Thu => "thu",
    | chrono::Weekday::Fri => "fri",
    | chrono::Weekday::Sat => "sat",
    | chrono::Weekday::Sun => "sun"
  }
}
