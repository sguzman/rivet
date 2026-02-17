#[derive(Properties, PartialEq)]
struct CalendarPeriodTaskListPanelProps {
  tasks:      Vec<CalendarDueTask>,
  timezone:   Tz,
  tag_colors: BTreeMap<String, String>
}

#[function_component(CalendarPeriodTaskListPanel)]
fn calendar_period_task_list_panel(
  props: &CalendarPeriodTaskListPanelProps
) -> Html {
  html! {
      <div class="panel">
          <div class="header">{ "Tasks In Current Period" }</div>
          <div class="details calendar-task-list">
              {
                  if props.tasks.is_empty() {
                      html! {
                          <div class="calendar-empty">
                              { "No tasks due in this calendar period." }
                          </div>
                      }
                  } else {
                      html! {
                          <>
                              {
                                  for props.tasks.iter().cloned().map(|entry| html! {
                                      <CalendarPeriodTaskCard
                                          entry={entry}
                                          timezone={props.timezone}
                                          tag_colors={props.tag_colors.clone()}
                                      />
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
