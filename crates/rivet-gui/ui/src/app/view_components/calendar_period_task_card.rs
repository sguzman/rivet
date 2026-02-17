#[derive(Properties, PartialEq)]
struct CalendarPeriodTaskCardProps {
  entry: CalendarDueTask,
  timezone: Tz,
  tag_colors:
    BTreeMap<String, String>
}

#[function_component(CalendarPeriodTaskCard)]
fn calendar_period_task_card(
  props: &CalendarPeriodTaskCardProps
) -> Html {
  let due_label =
    format_calendar_due_datetime(
      &props.entry,
      props.timezone
    );
  html! {
      <div class="calendar-task-item">
          <div class="calendar-task-title">{ &props.entry.task.title }</div>
          <div class="task-subtitle">{ due_label }</div>
          <div class="calendar-task-meta">
              {
                  if let Some(project) = props.entry.task.project.clone() {
                      html! { <span class="badge">{ format!("project:{project}") }</span> }
                  } else {
                      html! {}
                  }
              }
              {
                  for props.entry.task.tags.iter().take(3).map(|tag| html! {
                      <span class="badge tag-badge" style={tag_badge_style(tag, &props.tag_colors)}>{ format!("#{tag}") }</span>
                  })
              }
          </div>
      </div>
  }
}
