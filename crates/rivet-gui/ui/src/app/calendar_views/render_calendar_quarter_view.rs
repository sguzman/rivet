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

