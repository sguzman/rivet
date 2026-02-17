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

