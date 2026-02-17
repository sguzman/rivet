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

