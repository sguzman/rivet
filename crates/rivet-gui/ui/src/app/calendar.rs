fn calendar_true() -> bool {
  true
}

fn calendar_default_week_start()
-> String {
  "monday".to_string()
}

fn calendar_default_red_dot_limit()
-> usize {
  5_000
}

fn calendar_default_task_list_limit()
-> usize {
  200
}

fn calendar_default_task_list_window_days()
-> i64 {
  365
}

fn calendar_default_day_view_hour_end()
-> u32 {
  23
}

fn load_calendar_config()
-> CalendarConfig {
  match toml::from_str::<CalendarConfig>(
    CALENDAR_CONFIG_TOML
  ) {
    | Ok(mut config) => {
      sanitize_calendar_config(
        &mut config
      );
      tracing::info!(
        version = config.version,
        timezone = ?config.timezone,
        week_start = %config.policies.week_start,
        "loaded calendar config"
      );
      config
    }
    | Err(error) => {
      tracing::error!(%error, "failed parsing calendar config; using defaults");
      CalendarConfig::default()
    }
  }
}

fn sanitize_calendar_config(
  config: &mut CalendarConfig
) {
  if config
    .policies
    .week_start
    .trim()
    .is_empty()
  {
    config.policies.week_start =
      calendar_default_week_start();
  }

  if config.policies.red_dot_limit == 0
  {
    config.policies.red_dot_limit =
      calendar_default_red_dot_limit();
  }

  if config.policies.task_list_limit
    == 0
  {
    config.policies.task_list_limit =
      calendar_default_task_list_limit(
      );
  }

  if config
    .policies
    .task_list_window_days
    <= 0
  {
    config
      .policies
      .task_list_window_days =
      calendar_default_task_list_window_days(
      );
  }

  if config.day_view.hour_start > 23 {
    config.day_view.hour_start = 23;
  }
  if config.day_view.hour_end > 23 {
    config.day_view.hour_end = 23;
  }
  if config.day_view.hour_end
    < config.day_view.hour_start
  {
    config.day_view.hour_end =
      config.day_view.hour_start;
  }
}

fn load_calendar_view_mode()
-> CalendarViewMode {
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
          CALENDAR_VIEW_STORAGE_KEY
        )
        .ok()
        .flatten()
    });

  stored
    .as_deref()
    .and_then(
      CalendarViewMode::from_key
    )
    .unwrap_or(CalendarViewMode::Month)
}

fn save_calendar_view_mode(
  view: CalendarViewMode
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
    let _ = storage.set_item(
      CALENDAR_VIEW_STORAGE_KEY,
      view.as_key()
    );
  }
}

fn resolve_calendar_timezone(
  config: &CalendarConfig
) -> Tz {
  if let Some(raw) =
    config.timezone.as_ref()
    && let Some(tz) =
      parse_calendar_timezone(
        raw,
        "calendar.toml"
      )
  {
    return tz;
  }

  if let Ok(time_config) =
    toml::from_str::<ProjectTimeConfig>(
      PROJECT_TIME_CONFIG_TOML
    )
  {
    let timezone = time_config
      .timezone
      .or_else(|| {
        time_config.time.and_then(
          |section| section.timezone
        )
      });
    if let Some(raw) = timezone
      && let Some(tz) =
        parse_calendar_timezone(
          &raw,
          "rivet-time.toml"
        )
    {
      return tz;
    }
  } else {
    tracing::warn!(
      "failed to parse embedded \
       rivet-time.toml; falling back \
       to default timezone"
    );
  }

  parse_calendar_timezone(
    DEFAULT_CALENDAR_TIMEZONE,
    "calendar-default"
  )
  .unwrap_or(chrono_tz::UTC)
}

fn parse_calendar_timezone(
  raw: &str,
  source: &str
) -> Option<Tz> {
  let trimmed = raw.trim();
  if trimmed.is_empty() {
    return None;
  }

  match trimmed.parse::<Tz>() {
    | Ok(tz) => Some(tz),
    | Err(error) => {
      tracing::error!(
        source,
        timezone = %trimmed,
        error = %error,
        "invalid timezone id"
      );
      None
    }
  }
}

fn today_in_timezone(
  timezone: Tz
) -> NaiveDate {
  Utc::now()
    .with_timezone(&timezone)
    .date_naive()
}

fn calendar_week_start_day(
  raw: &str
) -> Weekday {
  if raw
    .trim()
    .eq_ignore_ascii_case("sunday")
  {
    Weekday::Sun
  } else {
    Weekday::Mon
  }
}

fn shift_calendar_focus(
  current: NaiveDate,
  view: CalendarViewMode,
  step: i64,
  _week_start: Weekday
) -> NaiveDate {
  match view {
    | CalendarViewMode::Year => {
      shift_years(current, step as i32)
    }
    | CalendarViewMode::Quarter => {
      shift_months(
        current,
        (step * 3) as i32
      )
    }
    | CalendarViewMode::Month => {
      shift_months(current, step as i32)
    }
    | CalendarViewMode::Week => {
      add_days(current, step * 7)
    }
    | CalendarViewMode::Day => {
      add_days(current, step)
    }
  }
}

fn shift_years(
  date: NaiveDate,
  years: i32
) -> NaiveDate {
  let year =
    date.year().saturating_add(years);
  let month = date.month();
  let day = date
    .day()
    .min(days_in_month(year, month));
  NaiveDate::from_ymd_opt(
    year, month, day
  )
  .unwrap_or(date)
}

fn shift_months(
  date: NaiveDate,
  months: i32
) -> NaiveDate {
  let mut year = date.year();
  let mut month =
    date.month() as i32 + months;

  while month < 1 {
    month += 12;
    year = year.saturating_sub(1);
  }
  while month > 12 {
    month -= 12;
    year = year.saturating_add(1);
  }

  let month = month as u32;
  let day = date
    .day()
    .min(days_in_month(year, month));
  NaiveDate::from_ymd_opt(
    year, month, day
  )
  .unwrap_or(date)
}

fn first_day_of_month(
  year: i32,
  month: u32
) -> NaiveDate {
  NaiveDate::from_ymd_opt(
    year, month, 1
  )
  .unwrap_or(NaiveDate::MIN)
}

fn last_day_of_month(
  year: i32,
  month: u32
) -> NaiveDate {
  let (next_year, next_month) =
    if month >= 12 {
      (year.saturating_add(1), 1_u32)
    } else {
      (year, month + 1)
    };
  add_days(
    first_day_of_month(
      next_year, next_month
    ),
    -1
  )
}

fn days_in_month(
  year: i32,
  month: u32
) -> u32 {
  last_day_of_month(year, month).day()
}

fn add_days(
  date: NaiveDate,
  days: i64
) -> NaiveDate {
  date
    .checked_add_signed(Duration::days(
      days
    ))
    .unwrap_or(date)
}

fn start_of_week(
  day: NaiveDate,
  week_start: Weekday
) -> NaiveDate {
  let day_idx = day
    .weekday()
    .num_days_from_monday()
    as i64;
  let start_idx = week_start
    .num_days_from_monday()
    as i64;
  let diff =
    (7 + day_idx - start_idx) % 7;
  add_days(day, -diff)
}

fn collect_calendar_due_tasks(
  tasks: &[TaskDto],
  timezone: Tz,
  config: &CalendarConfig,
  board_colors: &BTreeMap<
    String,
    String
  >,
  calendar_colors: &BTreeMap<
    String,
    String
  >
) -> Vec<CalendarDueTask> {
  let mut entries = tasks
    .iter()
    .filter(|task| {
      calendar_status_visible(
        &task.status,
        &config.visibility
      )
    })
    .filter_map(|task| {
      let due_raw =
        task.due.as_ref()?;
      let due_utc =
        parse_taskwarrior_utc(
          due_raw.as_str()
        )?;
      let marker = marker_for_task(
        task,
        board_colors,
        calendar_colors
      );
      Some(CalendarDueTask {
        task: task.clone(),
        due_local: due_utc
          .with_timezone(&timezone),
        due_utc,
        marker
      })
    })
    .collect::<Vec<_>>();

  entries
    .sort_by_key(|entry| entry.due_utc);

  tracing::debug!(
    total_tasks = tasks.len(),
    due_tasks = entries.len(),
    timezone = %timezone,
    "calendar tasks collected"
  );
  entries
}

fn marker_for_task(
  task: &TaskDto,
  board_colors: &BTreeMap<
    String,
    String
  >,
  calendar_colors: &BTreeMap<
    String,
    String
  >
) -> CalendarTaskMarker {
  if let Some(calendar_id) =
    first_tag_value(
      &task.tags,
      CAL_SOURCE_TAG_KEY
    )
  {
    let color = first_tag_value(
      &task.tags,
      CAL_COLOR_TAG_KEY
    )
    .map(normalize_marker_color)
    .or_else(|| {
      calendar_colors
        .get(calendar_id)
        .cloned()
    })
    .unwrap_or_else(|| {
      "#d64545".to_string()
    });
    return CalendarTaskMarker {
      shape:
        CalendarMarkerShape::Circle,
      color
    };
  }

  if let Some(board_id) =
    first_tag_value(
      &task.tags,
      BOARD_TAG_KEY
    )
  {
    let color = board_colors
      .get(board_id)
      .cloned()
      .unwrap_or_else(|| {
        default_board_color()
      });
    return CalendarTaskMarker {
      shape:
        CalendarMarkerShape::Triangle,
      color
    };
  }

  CalendarTaskMarker {
    shape: CalendarMarkerShape::Square,
    color: CALENDAR_UNAFFILIATED_COLOR
      .to_string()
  }
}

fn calendar_status_visible(
  status: &TaskStatus,
  visibility: &CalendarVisibility
) -> bool {
  match status {
    | TaskStatus::Pending => {
      visibility.pending
    }
    | TaskStatus::Waiting => {
      visibility.waiting
    }
    | TaskStatus::Completed => {
      visibility.completed
    }
    | TaskStatus::Deleted => {
      visibility.deleted
    }
  }
}

fn parse_taskwarrior_utc(
  raw: &str
) -> Option<DateTime<Utc>> {
  NaiveDateTime::parse_from_str(
    raw,
    "%Y%m%dT%H%M%SZ"
  )
  .ok()
  .map(|naive| {
    DateTime::<Utc>::from_naive_utc_and_offset(
      naive, Utc
    )
  })
}

fn calendar_date_window(
  view: CalendarViewMode,
  focus: NaiveDate,
  week_start: Weekday,
  _config: &CalendarConfig
) -> (NaiveDate, NaiveDate) {
  match view {
    | CalendarViewMode::Year => {
      (
        first_day_of_month(
          focus.year(),
          1
        ),
        last_day_of_month(
          focus.year(),
          12
        )
      )
    }
    | CalendarViewMode::Quarter => {
      let quarter_start_month =
        ((focus.month() - 1) / 3) * 3
          + 1;
      let start = first_day_of_month(
        focus.year(),
        quarter_start_month
      );
      let end = last_day_of_month(
        focus.year(),
        quarter_start_month + 2
      );
      (start, end)
    }
    | CalendarViewMode::Month => {
      (
        first_day_of_month(
          focus.year(),
          focus.month()
        ),
        last_day_of_month(
          focus.year(),
          focus.month()
        )
      )
    }
    | CalendarViewMode::Week => {
      let start = start_of_week(
        focus, week_start
      );
      (start, add_days(start, 6))
    }
    | CalendarViewMode::Day => {
      (focus, focus)
    }
  }
}

fn summarize_calendar_period(
  due_tasks: &[CalendarDueTask],
  view: CalendarViewMode,
  focus: NaiveDate,
  week_start: Weekday,
  config: &CalendarConfig
) -> CalendarStats {
  let (start, end) =
    calendar_date_window(
      view, focus, week_start, config
    );
  let mut stats =
    CalendarStats::default();

  for entry in due_tasks {
    let day =
      entry.due_local.date_naive();
    if day < start || day > end {
      continue;
    }
    stats.push(&entry.task.status);
  }

  stats
}

fn collect_calendar_period_tasks(
  due_tasks: &[CalendarDueTask],
  view: CalendarViewMode,
  focus: NaiveDate,
  week_start: Weekday,
  config: &CalendarConfig
) -> Vec<CalendarDueTask> {
  let (start, end) =
    calendar_date_window(
      view, focus, week_start, config
    );

  due_tasks
    .iter()
    .filter(|entry| {
      let day =
        entry.due_local.date_naive();
      day >= start && day <= end
    })
    .cloned()
    .collect()
}

fn calendar_title_for_view(
  view: CalendarViewMode,
  focus: NaiveDate,
  week_start: Weekday
) -> String {
  match view {
    | CalendarViewMode::Year => {
      format!(
        "Year View {}",
        focus.year()
      )
    }
    | CalendarViewMode::Quarter => {
      let quarter =
        ((focus.month() - 1) / 3) + 1;
      let quarter_start_month =
        ((focus.month() - 1) / 3) * 3
          + 1;
      let start = first_day_of_month(
        focus.year(),
        quarter_start_month
      );
      let end = first_day_of_month(
        focus.year(),
        quarter_start_month + 2
      );
      format!(
        "Quarter View Q{} {} ({}-{})",
        quarter,
        focus.year(),
        start.format("%b"),
        end.format("%b")
      )
    }
    | CalendarViewMode::Month => {
      format!(
        "Month View {}",
        focus.format("%B %Y")
      )
    }
    | CalendarViewMode::Week => {
      let start = start_of_week(
        focus, week_start
      );
      let end = add_days(start, 6);
      format!(
        "Week View {} - {}",
        start.format("%Y-%m-%d"),
        end.format("%Y-%m-%d")
      )
    }
    | CalendarViewMode::Day => {
      format!(
        "Day View {}",
        focus.format("%A, %Y-%m-%d")
      )
    }
  }
}

fn render_calendar_view(
  view: CalendarViewMode,
  focus: NaiveDate,
  week_start: Weekday,
  due_tasks: &[CalendarDueTask],
  config: &CalendarConfig,
  tag_colors: &BTreeMap<String, String>,
  on_navigate: Callback<(
    NaiveDate,
    CalendarViewMode
  )>
) -> Html {
  match view {
    | CalendarViewMode::Year => {
      render_calendar_year_view(
        focus,
        due_tasks,
        config,
        on_navigate
      )
    }
    | CalendarViewMode::Quarter => {
      render_calendar_quarter_view(
        focus,
        due_tasks,
        config,
        on_navigate
      )
    }
    | CalendarViewMode::Month => {
      render_calendar_month_view(
        focus,
        week_start,
        due_tasks,
        config,
        on_navigate
      )
    }
    | CalendarViewMode::Week => {
      render_calendar_week_view(
        focus,
        week_start,
        due_tasks,
        config,
        on_navigate
      )
    }
    | CalendarViewMode::Day => {
      render_calendar_day_view(
        focus, due_tasks, config,
        tag_colors
      )
    }
  }
}

fn render_calendar_year_view(
  focus: NaiveDate,
  due_tasks: &[CalendarDueTask],
  config: &CalendarConfig,
  on_navigate: Callback<(
    NaiveDate,
    CalendarViewMode
  )>
) -> Html {
  let year = focus.year();

  html! {
      <div class="calendar-grid calendar-year-grid">
          {
              for (1_u32..=12_u32).map(|month| {
                  let month_start = first_day_of_month(year, month);
                  let markers = due_tasks
                      .iter()
                      .filter(|entry| {
                          entry.due_local.year() == year
                              && entry.due_local.month() == month
                      })
                      .map(|entry| entry.marker.clone())
                      .collect::<Vec<_>>();
                  let count = markers.len();
                  let on_navigate = on_navigate.clone();

                  html! {
                      <button
                          type="button"
                          class="calendar-period-card"
                          onclick={Callback::from(move |_| on_navigate.emit((month_start, CalendarViewMode::Month)))}
                      >
                          <div class="calendar-period-title">{ month_start.format("%B").to_string() }</div>
                          <div class="badge">{ format!("{count} tasks") }</div>
                          { render_calendar_markers(&markers, config.policies.red_dot_limit) }
                      </button>
                  }
              })
          }
      </div>
  }
}

fn render_calendar_quarter_view(
  focus: NaiveDate,
  due_tasks: &[CalendarDueTask],
  config: &CalendarConfig,
  on_navigate: Callback<(
    NaiveDate,
    CalendarViewMode
  )>
) -> Html {
  let quarter_start_month =
    ((focus.month() - 1) / 3) * 3 + 1;
  let months = [
    quarter_start_month,
    quarter_start_month + 1,
    quarter_start_month + 2
  ];

  html! {
      <div class="calendar-grid calendar-quarter-grid">
          {
              for months.into_iter().map(|month| {
                  let month_start = first_day_of_month(focus.year(), month);
                  let markers = due_tasks
                      .iter()
                      .filter(|entry| {
                          entry.due_local.year() == focus.year()
                              && entry.due_local.month() == month
                      })
                      .map(|entry| entry.marker.clone())
                      .collect::<Vec<_>>();
                  let count = markers.len();
                  let on_navigate = on_navigate.clone();
                  html! {
                      <button
                          type="button"
                          class="calendar-period-card"
                          onclick={Callback::from(move |_| on_navigate.emit((month_start, CalendarViewMode::Month)))}
                      >
                          <div class="calendar-period-title">{ month_start.format("%B").to_string() }</div>
                          <div class="badge">{ format!("{count} tasks") }</div>
                          { render_calendar_markers(&markers, config.policies.red_dot_limit) }
                      </button>
                  }
              })
          }
      </div>
  }
}

fn render_calendar_month_view(
  focus: NaiveDate,
  week_start: Weekday,
  due_tasks: &[CalendarDueTask],
  config: &CalendarConfig,
  on_navigate: Callback<(
    NaiveDate,
    CalendarViewMode
  )>
) -> Html {
  let first = first_day_of_month(
    focus.year(),
    focus.month()
  );
  let grid_start =
    start_of_week(first, week_start);
  let labels =
    weekday_labels(week_start);
  let month_last = last_day_of_month(
    focus.year(),
    focus.month()
  );
  let mut week_starts = Vec::new();
  for row in 0_i64..6_i64 {
    let week_start_day =
      add_days(grid_start, row * 7);
    let week_end_day =
      add_days(week_start_day, 6);
    if week_end_day < first
      || week_start_day > month_last
    {
      continue;
    }
    week_starts.push(week_start_day);
  }

  html! {
      <>
          <div class="calendar-weekday-row">
              {
                  for labels.into_iter().map(|label| html! {
                      <div class="calendar-weekday">{ label }</div>
                  })
              }
          </div>
          <div class="calendar-grid calendar-month-grid">
              {
                  for (0_i64..42_i64).map(|offset| {
                      let day = add_days(grid_start, offset);
                      let markers = due_tasks
                          .iter()
                          .filter(|entry| entry.due_local.date_naive() == day)
                          .map(|entry| entry.marker.clone())
                          .collect::<Vec<_>>();
                      let count = markers.len();
                      let outside = day.month() != focus.month();
                      let on_navigate = on_navigate.clone();
                      html! {
                          <button
                              type="button"
                              class={classes!("calendar-day-cell", outside.then_some("outside"), (count > 0).then_some("has-tasks"))}
                              onclick={Callback::from(move |_| on_navigate.emit((day, CalendarViewMode::Day)))}
                          >
                              <div class="calendar-day-label">{ day.day() }</div>
                              { render_calendar_markers(&markers, config.policies.red_dot_limit) }
                          </button>
                      }
                  })
              }
          </div>
          <div class="calendar-week-shortcuts">
              {
                  for week_starts.into_iter().map(|week_start_day| {
                      let week_end_day =
                        add_days(
                          week_start_day,
                          6
                        );
                      let label = format!(
                          "{} - {}",
                          week_start_day.format("%b %d"),
                          week_end_day.format("%b %d")
                      );
                      let on_navigate =
                        on_navigate.clone();
                      html! {
                          <button
                              type="button"
                              class="btn calendar-week-shortcut-btn"
                              onclick={Callback::from(move |_| on_navigate.emit((week_start_day, CalendarViewMode::Week)))}
                          >
                              { label }
                          </button>
                      }
                  })
              }
          </div>
      </>
  }
}

fn render_calendar_week_view(
  focus: NaiveDate,
  week_start: Weekday,
  due_tasks: &[CalendarDueTask],
  config: &CalendarConfig,
  on_navigate: Callback<(
    NaiveDate,
    CalendarViewMode
  )>
) -> Html {
  let start =
    start_of_week(focus, week_start);

  html! {
      <div class="calendar-grid calendar-week-grid">
          {
              for (0_i64..7_i64).map(|offset| {
                  let day = add_days(start, offset);
                  let markers = due_tasks
                      .iter()
                      .filter(|entry| entry.due_local.date_naive() == day)
                      .map(|entry| entry.marker.clone())
                      .collect::<Vec<_>>();
                  let day_tasks = due_tasks
                      .iter()
                      .filter(|entry| entry.due_local.date_naive() == day)
                      .cloned()
                      .collect::<Vec<_>>();
                  let count = day_tasks.len();
                  let on_navigate = on_navigate.clone();

                  html! {
                      <button
                          type="button"
                          class={classes!("calendar-week-day-card", (count > 0).then_some("has-tasks"))}
                          onclick={Callback::from(move |_| on_navigate.emit((day, CalendarViewMode::Day)))}
                      >
                          <div class="calendar-week-day-head">
                              <span>{ day.format("%a %d").to_string() }</span>
                              <span class="badge">{ count }</span>
                          </div>
                          { render_calendar_markers(&markers, config.policies.red_dot_limit) }
                          <div class="calendar-week-day-list">
                              {
                                  for day_tasks.iter().take(5).map(|entry| html! {
                                      <div class="calendar-week-task">{ &entry.task.title }</div>
                                  })
                              }
                              {
                                  if count > 5 {
                                      html! { <div class="calendar-week-task muted">{ format!("+{} more", count - 5) }</div> }
                                  } else {
                                      html! {}
                                  }
                              }
                          </div>
                      </button>
                  }
              })
          }
      </div>
  }
}

fn render_calendar_day_view(
  focus: NaiveDate,
  due_tasks: &[CalendarDueTask],
  config: &CalendarConfig,
  tag_colors: &BTreeMap<String, String>
) -> Html {
  let mut tasks = due_tasks
    .iter()
    .filter(|entry| {
      entry.due_local.date_naive()
        == focus
    })
    .cloned()
    .collect::<Vec<_>>();
  tasks
    .sort_by_key(|entry| entry.due_utc);

  let hour_start =
    config.day_view.hour_start;
  let hour_end =
    config.day_view.hour_end;

  html! {
      <div class="calendar-day-view">
          <div class="calendar-day-hours">
              {
                  for (hour_start..=hour_end).map(|hour| {
                      let markers = tasks
                          .iter()
                          .filter(|entry| entry.due_local.hour() == hour)
                          .map(|entry| entry.marker.clone())
                          .collect::<Vec<_>>();
                      html! {
                          <div class="calendar-hour-row">
                              <span class="calendar-hour-label">{ format!("{hour:02}:00") }</span>
                              { render_calendar_markers(&markers, config.policies.red_dot_limit) }
                          </div>
                      }
                  })
              }
          </div>
          <div class="calendar-day-task-list">
              {
                  if tasks.is_empty() {
                      html! { <div class="calendar-empty">{ "No tasks due on this day." }</div> }
                  } else {
                      html! {
                          <>
                              {
                                  for tasks.iter().map(|entry| html! {
                                      <div class="calendar-task-item">
                                          <div class="calendar-task-title">{ &entry.task.title }</div>
                                          <div class="task-subtitle">{ format_calendar_due_datetime(entry, entry.due_local.timezone()) }</div>
                                          <div class="calendar-task-meta">
                                              {
                                                  for entry.task.tags.iter().take(4).map(|tag| html! {
                                                      <span class="badge tag-badge" style={tag_badge_style(tag, tag_colors)}>{ format!("#{tag}") }</span>
                                                  })
                                              }
                                          </div>
                                      </div>
                                  })
                              }
                          </>
                      }
                  }
              }
          </div>
      </div>
  }
}

fn weekday_labels(
  week_start: Weekday
) -> Vec<&'static str> {
  match week_start {
    | Weekday::Sun => {
      vec![
        "Sun", "Mon", "Tue", "Wed",
        "Thu", "Fri", "Sat",
      ]
    }
    | _ => {
      vec![
        "Mon", "Tue", "Wed", "Thu",
        "Fri", "Sat", "Sun",
      ]
    }
  }
}

fn render_calendar_markers(
  markers: &[CalendarTaskMarker],
  limit: usize
) -> Html {
  if markers.is_empty() {
    return html! {};
  }

  let capped = markers.len().min(limit);
  let overflow = markers
    .len()
    .saturating_sub(capped);

  html! {
      <div class="calendar-markers">
          {
              for markers.iter().take(capped).map(|marker| {
                  let style = format!("--marker-color:{};", marker.color);
                  html! {
                      <span class={classes!("calendar-marker", marker.shape.as_class())} style={style}></span>
                  }
              })
          }
          {
              if overflow > 0 {
                  html! { <span class="badge">{ format!("+{overflow}") }</span> }
              } else {
                  html! {}
              }
          }
      </div>
  }
}

fn format_calendar_due_datetime(
  entry: &CalendarDueTask,
  timezone: Tz
) -> String {
  format!(
    "{} ({timezone})",
    entry
      .due_local
      .format("%Y-%m-%d %H:%M")
  )
}

#[cfg(test)]
mod tests {
  use chrono::DateTime;
  use rivet_gui_shared::TaskStatus;

  use super::*;

  fn sample_due_task(
    date: NaiveDate
  ) -> CalendarDueTask {
    let due_naive = date
      .and_hms_opt(12, 0, 0)
      .expect("valid noon time");
    let due_utc =
      DateTime::<Utc>::from_naive_utc_and_offset(
        due_naive, Utc
      );
    let due_local =
      due_utc.with_timezone(&chrono_tz::UTC);

    CalendarDueTask {
      task:      TaskDto {
        uuid:        Uuid::new_v4(),
        id:          None,
        title:       "sample".to_string(),
        description: String::new(),
        status:      TaskStatus::Pending,
        project:     None,
        tags:        vec![],
        priority:    None,
        due:         Some(
          due_utc
            .format("%Y%m%dT%H%M%SZ")
            .to_string()
        ),
        wait:        None,
        scheduled:   None,
        created:     None,
        modified:    None
      },
      due_utc,
      due_local,
      marker:    CalendarTaskMarker {
        shape: CalendarMarkerShape::Square,
        color: "#7f8691".to_string()
      }
    }
  }

  #[test]
  fn month_period_filters_tasks_by_month(
  ) {
    let config = CalendarConfig::default();
    let focus =
      NaiveDate::from_ymd_opt(2026, 2, 15)
        .expect("valid date");

    let tasks = vec![
      sample_due_task(
        NaiveDate::from_ymd_opt(
          2026, 2, 1
        )
        .expect("valid date"),
      ),
      sample_due_task(
        NaiveDate::from_ymd_opt(
          2026, 2, 28
        )
        .expect("valid date"),
      ),
      sample_due_task(
        NaiveDate::from_ymd_opt(
          2026, 3, 1
        )
        .expect("valid date"),
      ),
    ];

    let filtered =
      collect_calendar_period_tasks(
        &tasks,
        CalendarViewMode::Month,
        focus,
        Weekday::Mon,
        &config,
      );

    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|entry| {
      entry.due_local.month() == 2
    }));
  }

  #[test]
  fn week_period_filters_tasks_by_week() {
    let config = CalendarConfig::default();
    let focus =
      NaiveDate::from_ymd_opt(2026, 2, 18)
        .expect("valid date");

    let tasks = vec![
      sample_due_task(
        NaiveDate::from_ymd_opt(
          2026, 2, 16
        )
        .expect("valid date"),
      ),
      sample_due_task(
        NaiveDate::from_ymd_opt(
          2026, 2, 22
        )
        .expect("valid date"),
      ),
      sample_due_task(
        NaiveDate::from_ymd_opt(
          2026, 2, 23
        )
        .expect("valid date"),
      ),
    ];

    let filtered =
      collect_calendar_period_tasks(
        &tasks,
        CalendarViewMode::Week,
        focus,
        Weekday::Mon,
        &config,
      );

    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|entry| {
      let day = entry.due_local.date_naive();
      day >= NaiveDate::from_ymd_opt(
        2026, 2, 16
      )
      .expect("valid date")
        && day
          <= NaiveDate::from_ymd_opt(
            2026, 2, 22
          )
          .expect("valid date")
    }));
  }
}
