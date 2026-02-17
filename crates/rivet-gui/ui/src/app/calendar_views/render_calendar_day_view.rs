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

