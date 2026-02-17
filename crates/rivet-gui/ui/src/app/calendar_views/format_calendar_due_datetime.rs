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
