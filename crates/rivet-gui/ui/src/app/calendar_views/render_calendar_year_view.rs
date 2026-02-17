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

