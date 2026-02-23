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
  #[serde(default)]
  pub imported_ics_file: bool,
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

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ExternalCalendarImportArg {
  pub source:   ExternalCalendarSourceArg,
  pub ics_text: String
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ExternalCalendarImportCachedArg {
  pub source:   ExternalCalendarSourceArg,
  pub cache_id: String
}

#[derive(Debug, Clone)]
struct ExternalCalendarEvent {
  uid:         String,
  title:       String,
  description: String,
  due_rfc3339: String,
  tags:        Vec<String>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
struct ExternalCalendarCacheRecord {
  cache_id: String,
  source_id: String,
  name: String,
  location: String,
  color: String,
  cached_at: String,
  kind: String,
  ics_path: String
}

#[derive(Debug, Clone, Serialize)]
pub struct ExternalCalendarCacheEntry {
  pub cache_id: String,
  pub name:     String,
  pub location: String,
  pub color:    String,
  pub cached_at: String,
  pub kind:     String
}

#[tauri::command]
#[instrument(skip(state), fields(request_id = ?request_id, calendar_id = %args.id, name = %args.name, enabled = args.enabled))]
pub async fn external_calendar_sync(
  state: State<'_, AppState>,
  args: ExternalCalendarSourceArg,
  request_id: Option<String>
) -> Result<
  ExternalCalendarSyncResult,
  String
> {
  info!(request_id = ?request_id, calendar_id = %args.id, "external_calendar_sync command invoked");
  if args.imported_ics_file {
    return Err(
      "Imported ICS calendars are \
       local snapshots. Re-import \
       the file to update."
        .to_string(),
    );
  }

  if !args.enabled {
    return Ok(ExternalCalendarSyncResult {
      calendar_id: args.id,
      created: 0,
      updated: 0,
      deleted: 0,
      remote_events: 0,
      refresh_minutes: args
        .refresh_minutes
    });
  }

  let ics_text = fetch_ics_document(
    args.location.as_str()
  )
  .await
  .map_err(err_to_string)?;
  if let Err(error) =
    cache_ics_payload(
      &args,
      &ics_text,
      "remote_sync",
    )
  {
    warn!(
      calendar_id = %args.id,
      error = %error,
      "failed caching remote ICS payload"
    );
  }
  let events =
    parse_ics_events(&ics_text, &args)
      .map_err(err_to_string)?;
  apply_external_calendar_events(
    &state, &args, events
  )
  .map_err(err_to_string)
}

#[tauri::command]
#[instrument(skip(state, args), fields(request_id = ?request_id, calendar_id = %args.source.id, name = %args.source.name))]
pub async fn external_calendar_import_ics(
  state: State<'_, AppState>,
  args: ExternalCalendarImportArg,
  request_id: Option<String>
) -> Result<
  ExternalCalendarSyncResult,
  String
> {
  info!(request_id = ?request_id, calendar_id = %args.source.id, "external_calendar_import_ics command invoked");
  if args
    .ics_text
    .trim()
    .is_empty()
  {
    return Err(
      "ICS file is empty".to_string()
    );
  }

  if let Err(error) =
    cache_ics_payload(
      &args.source,
      &args.ics_text,
      "file_import",
    )
  {
    warn!(
      calendar_id = %args.source.id,
      error = %error,
      "failed caching imported ICS payload"
    );
  }

  let events = parse_ics_events(
    &args.ics_text,
    &args.source
  )
  .map_err(err_to_string)?;
  apply_external_calendar_events(
    &state,
    &args.source,
    events,
  )
  .map_err(err_to_string)
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id))]
pub async fn external_calendar_cache_list(
  request_id: Option<String>
) -> Result<
  Vec<ExternalCalendarCacheEntry>,
  String
> {
  info!(
    request_id = ?request_id,
    "external_calendar_cache_list command invoked"
  );
  list_cached_ics_entries()
    .map_err(err_to_string)
}

#[tauri::command]
#[instrument(skip(state, args), fields(request_id = ?request_id, calendar_id = %args.source.id, cache_id = %args.cache_id))]
pub async fn external_calendar_import_cached(
  state: State<'_, AppState>,
  args: ExternalCalendarImportCachedArg,
  request_id: Option<String>
) -> Result<
  ExternalCalendarSyncResult,
  String
> {
  info!(
    request_id = ?request_id,
    calendar_id = %args.source.id,
    cache_id = %args.cache_id,
    "external_calendar_import_cached command invoked"
  );
  let cache = load_cache_record(
    &args.cache_id
  )
  .map_err(err_to_string)?;
  let ics_text =
    std::fs::read_to_string(
      &cache.ics_path
    )
    .map_err(anyhow::Error::new)
    .with_context(|| {
      format!(
        "failed to read cached ICS \
         payload {}",
        cache.ics_path
      )
    })
    .map_err(err_to_string)?;

  if let Err(error) =
    cache_ics_payload(
      &args.source,
      &ics_text,
      "cache_reimport",
    )
  {
    warn!(
      calendar_id = %args.source.id,
      cache_id = %args.cache_id,
      error = %error,
      "failed caching reimported ICS payload"
    );
  }

  let events = parse_ics_events(
    &ics_text,
    &args.source,
  )
  .map_err(err_to_string)?;
  apply_external_calendar_events(
    &state,
    &args.source,
    events,
  )
  .map_err(err_to_string)
}

fn apply_external_calendar_events(
  state: &AppState,
  args: &ExternalCalendarSourceArg,
  events: Vec<ExternalCalendarEvent>
) -> anyhow::Result<
  ExternalCalendarSyncResult
> {
  let remote_events = events.len();

  let mut created = 0_usize;
  let mut updated = 0_usize;
  let mut deleted = 0_usize;

  let all_tasks = state.list(TasksListArgs {
      query: None,
      status: None,
      project: None,
      tag: None,
  })?;
  let mut existing_by_uid =
    BTreeMap::<String, TaskDto>::new();
  for task in all_tasks {
    if !task_has_tag_value(
      &task.tags,
      CAL_SOURCE_TAG_KEY,
      &normalize_tag_value(&args.id),
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
        CAL_EVENT_TAG_KEY,
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
            &event.tags,
          );
        let update = TaskUpdateArgs {
          uuid:  existing.uuid,
          patch: TaskPatch {
            title: Some(
              event.title.clone(),
            ),
            description: Some(
              event.description
                .clone(),
            ),
            project: Some(Some(
              format!(
                "calendar/{}",
                args.name.trim()
              ),
            )),
            tags: Some(merged_tags),
            due: Some(Some(
              event.due_rfc3339.clone(),
            )),
            wait: Some(None),
            scheduled: Some(None),
            ..TaskPatch::default()
          },
        };

        state.update(update)?;
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
            event.due_rfc3339.clone(),
          ),
          wait:        None,
          scheduled:   None,
        };

        state.add(create)?;
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
      state.delete(task.uuid)?;
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
    calendar_id: args.id.clone(),
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
  let candidate_locations =
    normalized_calendar_locations(trimmed);

  let client =
    reqwest::Client::builder()
      .timeout(Duration::from_secs(30))
      .build()
      .context(
        "failed building HTTP client \
         for calendar sync"
      )?;

  let mut last_error =
    None::<anyhow::Error>;
  for candidate in &candidate_locations {
    for request_profile in
      calendar_request_profiles()
    {
      let mut request = client
        .get(candidate.as_str())
        .header(
          reqwest::header::ACCEPT,
          request_profile.accept,
        )
        .header(
          reqwest::header::ACCEPT_LANGUAGE,
          "en-US,en;q=0.9",
        )
        .header(
          reqwest::header::CACHE_CONTROL,
          "no-cache",
        )
        .header(
          reqwest::header::PRAGMA,
          "no-cache",
        )
        .header(
          reqwest::header::USER_AGENT,
          request_profile.user_agent,
        );

      if let Some(referer) =
        request_profile.referer
      {
        request = request.header(
          reqwest::header::REFERER,
          referer,
        );
      }

      let response =
        match request.send().await {
          | Ok(response) => response,
          | Err(error) => {
            warn!(
              candidate = %candidate,
              profile = request_profile.name,
              error = %error,
              "failed requesting \
               external calendar \
               candidate URL"
            );
            last_error = Some(
              anyhow::Error::new(error)
                .context(format!(
                  "failed requesting \
                   calendar URL: \
                   {candidate}"
                )),
            );
            continue;
          }
        };

      let status = response.status();
      let body = match response.text().await {
        | Ok(body) => body,
        | Err(error) => {
          warn!(
            candidate = %candidate,
            profile = request_profile.name,
            error = %error,
            "failed reading calendar \
             response body for \
             candidate URL"
          );
          last_error = Some(
            anyhow::Error::new(error)
              .context(format!(
                "failed reading \
                 calendar response \
                 body for {}",
                candidate
              )),
          );
          continue;
        }
      };

      if status.is_success() {
        return Ok(body);
      }

      if status
        == reqwest::StatusCode::FORBIDDEN
      {
        if is_bls_bot_denied_body(
          &body
        ) {
          warn!(
            candidate = %candidate,
            profile = request_profile.name,
            "calendar source blocked \
             request as bot traffic"
          );
          last_error = Some(
            anyhow::anyhow!(
              "calendar source denied \
               automated access (HTTP \
               403). This endpoint \
               appears to enforce \
               anti-bot policy. Try a \
               lower sync rate, \
               manual sync, or an \
               alternate ICS host."
            ),
          );
        } else {
          warn!(
            candidate = %candidate,
            profile = request_profile.name,
            "calendar source returned \
             forbidden without known \
             anti-bot signature"
          );
          last_error = Some(
            anyhow::anyhow!(
              "calendar URL returned \
               HTTP 403 for {}",
              candidate
            ),
          );
        }
      } else {
        warn!(
          candidate = %candidate,
          profile = request_profile.name,
          status = %status,
          "external calendar \
           candidate URL returned \
           non-success status"
        );
        last_error = Some(
          anyhow::anyhow!(
            "calendar URL returned \
             HTTP {} for {}",
            status,
            candidate
          ),
        );
      }
    }
  }

  Err(last_error.unwrap_or_else(|| {
    anyhow::anyhow!(
      "unable to fetch calendar from \
       any candidate URL"
    )
  }))
}

fn normalized_calendar_locations(
  location: &str
) -> Vec<String> {
  let trimmed = location.trim();
  let lower =
    trimmed.to_ascii_lowercase();

  if lower.starts_with("webcal://") {
    let remainder = &trimmed[9..];
    let https =
      format!("https://{remainder}");
    let http =
      format!("http://{remainder}");
    info!(
      raw = %trimmed,
      normalized = %https,
      fallback = %http,
      "rewrote webcal calendar URL \
       candidates"
    );
    return vec![https, http];
  }

  if lower.starts_with("webcals://") {
    let remainder = &trimmed[10..];
    let normalized =
      format!("https://{remainder}");
    info!(
      raw = %trimmed,
      normalized = %normalized,
      "rewrote webcals calendar URL to https"
    );
    return vec![normalized];
  }

  vec![trimmed.to_string()]
}

#[derive(Clone, Copy)]
struct CalendarRequestProfile {
  name:       &'static str,
  user_agent: &'static str,
  accept:     &'static str,
  referer:    Option<&'static str>
}

fn calendar_request_profiles()
-> [CalendarRequestProfile; 2] {
  [
    CalendarRequestProfile {
      name: "thunderbird",
      user_agent:
        "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Thunderbird/128.0",
      accept:
        "text/calendar, text/plain, */*;q=0.8",
      referer:
        Some("https://www.bls.gov/schedule/news_release/")
    },
    CalendarRequestProfile {
      name: "browser",
      user_agent:
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36",
      accept:
        "text/calendar, text/plain, */*;q=0.8",
      referer:
        Some("https://www.bls.gov/schedule/news_release/")
    },
  ]
}

fn is_bls_bot_denied_body(
  body: &str
) -> bool {
  let body_lower =
    body.to_ascii_lowercase();
  body_lower.contains("access denied")
    && body_lower.contains(
      "automated retrieval programs"
    )
    && body_lower.contains(
      "bls usage policy"
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
  )
    && !source.imported_ics_file
  {
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

fn resolve_gui_data_dir()
-> std::path::PathBuf {
  if let Ok(path) =
    std::env::var("RIVET_GUI_DATA")
  {
    return std::path::PathBuf::from(
      path
    );
  }

  if let Ok(cwd) =
    std::env::current_dir()
  {
    return cwd.join(".rivet_gui_data");
  }

  std::path::PathBuf::from(
    ".rivet_gui_data"
  )
}

fn external_calendar_cache_dir()
-> std::path::PathBuf {
  resolve_gui_data_dir().join(
    "calendar_ics_cache"
  )
}

fn sanitize_cache_fragment(
  value: &str
) -> String {
  let sanitized = value
    .chars()
    .map(|ch| {
      if ch.is_ascii_alphanumeric()
        || ch == '-'
        || ch == '_'
      {
        ch
      } else {
        '_'
      }
    })
    .collect::<String>();
  let trimmed =
    sanitized.trim_matches('_');
  if trimmed.is_empty() {
    "calendar".to_string()
  } else {
    trimmed.to_string()
  }
}

fn cache_ics_payload(
  source: &ExternalCalendarSourceArg,
  ics_text: &str,
  kind: &str
) -> anyhow::Result<String> {
  let cache_dir =
    external_calendar_cache_dir();
  std::fs::create_dir_all(
    &cache_dir
  )
  .map_err(anyhow::Error::new)
  .with_context(|| {
    format!(
      "failed creating calendar cache directory {}",
      cache_dir.display()
    )
  })?;

  let now = Utc::now();
  let cache_id = format!(
    "{}-{}-{}",
    sanitize_cache_fragment(
      &source.id
    ),
    now.format(
      "%Y%m%dT%H%M%S"
    ),
    uuid::Uuid::new_v4()
  );

  let ics_path = cache_dir.join(
    format!("{cache_id}.ics")
  );
  std::fs::write(
    &ics_path, ics_text
  )
  .map_err(anyhow::Error::new)
  .with_context(|| {
    format!(
      "failed writing cached ICS file {}",
      ics_path.display()
    )
  })?;

  let record = ExternalCalendarCacheRecord {
    cache_id: cache_id.clone(),
    source_id: source.id.clone(),
    name: source.name.clone(),
    location: source.location.clone(),
    color: source.color.clone(),
    cached_at: now.to_rfc3339(),
    kind: kind.to_string(),
    ics_path: ics_path.display().to_string()
  };
  let metadata_path = cache_dir.join(
    format!("{cache_id}.json")
  );
  let metadata_raw =
    serde_json::to_string_pretty(
      &record
    )
    .map_err(anyhow::Error::new)
    .context(
      "failed encoding ICS cache metadata"
    )?;
  std::fs::write(
    &metadata_path,
    metadata_raw
  )
  .map_err(anyhow::Error::new)
  .with_context(|| {
    format!(
      "failed writing ICS cache metadata {}",
      metadata_path.display()
    )
  })?;

  Ok(cache_id)
}

fn load_cache_record(
  cache_id: &str
) -> anyhow::Result<
  ExternalCalendarCacheRecord
> {
  let normalized =
    sanitize_cache_fragment(cache_id);
  let metadata_path =
    external_calendar_cache_dir().join(
      format!("{normalized}.json")
    );
  let metadata_raw =
    std::fs::read_to_string(
      &metadata_path
    )
    .map_err(anyhow::Error::new)
    .with_context(|| {
      format!(
        "failed reading cache metadata {}",
        metadata_path.display()
      )
    })?;
  let record =
    serde_json::from_str::<
      ExternalCalendarCacheRecord
    >(&metadata_raw)
    .map_err(anyhow::Error::new)
    .with_context(|| {
      format!(
        "failed parsing cache metadata {}",
        metadata_path.display()
      )
    })?;
  Ok(record)
}

fn list_cached_ics_entries()
-> anyhow::Result<
  Vec<ExternalCalendarCacheEntry>
> {
  let cache_dir =
    external_calendar_cache_dir();
  if !cache_dir.is_dir() {
    return Ok(Vec::new());
  }

  let mut entries = Vec::new();
  for dir_entry in std::fs::read_dir(
    &cache_dir
  )
  .map_err(anyhow::Error::new)
  .with_context(|| {
    format!(
      "failed reading cache directory {}",
      cache_dir.display()
    )
  })? {
    let dir_entry =
      match dir_entry {
        | Ok(entry) => entry,
        | Err(error) => {
          warn!(
            error = %error,
            "failed iterating ICS cache directory entry"
          );
          continue;
        }
      };
    let path = dir_entry.path();
    let is_json = path
      .extension()
      .and_then(|value| {
        value.to_str()
      })
      .is_some_and(|value| {
        value.eq_ignore_ascii_case(
          "json"
        )
      });
    if !is_json {
      continue;
    }

    let raw = match std::fs::read_to_string(
      &path
    ) {
      | Ok(raw) => raw,
      | Err(error) => {
        warn!(
          path = %path.display(),
          error = %error,
          "failed reading ICS cache metadata file"
        );
        continue;
      }
    };
    let record =
      match serde_json::from_str::<
        ExternalCalendarCacheRecord
      >(&raw) {
        | Ok(record) => record,
        | Err(error) => {
          warn!(
            path = %path.display(),
            error = %error,
            "failed parsing ICS cache metadata"
          );
          continue;
        }
      };
    if !std::path::Path::new(
      &record.ics_path
    )
    .is_file()
    {
      warn!(
        cache_id = %record.cache_id,
        ics_path = %record.ics_path,
        "skipping ICS cache entry with missing ICS payload file"
      );
      continue;
    }
    entries.push(
      ExternalCalendarCacheEntry {
        cache_id: record.cache_id,
        name: record.name,
        location: record.location,
        color: record.color,
        cached_at: record.cached_at,
        kind: record.kind
      }
    );
  }

  entries.sort_by(|left, right| {
    right
      .cached_at
      .cmp(&left.cached_at)
  });
  Ok(entries)
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
