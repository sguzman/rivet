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

