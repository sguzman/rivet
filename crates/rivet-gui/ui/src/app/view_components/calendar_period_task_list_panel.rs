#[derive(Properties, PartialEq)]
struct CalendarPeriodTaskListPanelProps {
  tasks:               Vec<CalendarDueTask>,
  timezone:            Tz,
  tag_colors:          BTreeMap<String, String>,
  external_calendars:  Vec<ExternalCalendarSource>
}

#[function_component(CalendarPeriodTaskListPanel)]
fn calendar_period_task_list_panel(
  props: &CalendarPeriodTaskListPanelProps
) -> Html {
  let selected_calendar =
    use_state(|| {
      "__all__".to_string()
    });

  let calendar_name_map = props
    .external_calendars
    .iter()
    .map(|source| {
      (
        source.id.clone(),
        source.name.clone(),
      )
    })
    .collect::<BTreeMap<_, _>>();

  let calendar_filter_options = {
    let mut ids =
      BTreeSet::<String>::new();
    for entry in &props.tasks {
      if let Some(calendar_id) =
        first_tag_value(
          &entry.task.tags,
          CAL_SOURCE_TAG_KEY,
        )
      {
        ids.insert(
          calendar_id.to_string()
        );
      }
    }

    ids.into_iter()
      .map(|calendar_id| {
        let label = calendar_name_map
          .get(&calendar_id)
          .cloned()
          .unwrap_or_else(|| {
            format!(
              "Calendar {}",
              &calendar_id
            )
          });
        (calendar_id, label)
      })
      .collect::<Vec<_>>()
  };

  let filtered_tasks = props
    .tasks
    .iter()
    .filter(|entry| {
      let selected =
        selected_calendar.as_str();
      if selected == "__all__" {
        return true;
      }
      if selected == "__none__" {
        return first_tag_value(
          &entry.task.tags,
          CAL_SOURCE_TAG_KEY,
        )
        .is_none();
      }
      first_tag_value(
        &entry.task.tags,
        CAL_SOURCE_TAG_KEY,
      )
      .is_some_and(|calendar_id| {
        calendar_id == selected
      })
    })
    .cloned()
    .collect::<Vec<_>>();

  let on_calendar_filter_change = {
    let selected_calendar =
      selected_calendar.clone();
    Callback::from(
      move |e: web_sys::Event| {
        let select: web_sys::HtmlSelectElement =
          e.target_unchecked_into();
        selected_calendar
          .set(select.value());
      },
    )
  };

  html! {
      <div class="panel">
          <div class="header">{ "Tasks In Current Period" }</div>
          <div class="details calendar-task-list">
              <div class="field calendar-period-filter">
                  <label>{ "Calendar" }</label>
                  <select
                      class="tag-select"
                      value={(*selected_calendar).clone()}
                      onchange={on_calendar_filter_change}
                  >
                      <option value="__all__">{ "All calendars and tasks" }</option>
                      <option value="__none__">{ "No calendar source" }</option>
                      {
                          for calendar_filter_options.iter().map(|(calendar_id, label)| html! {
                              <option value={calendar_id.clone()}>{ label.clone() }</option>
                          })
                      }
                  </select>
              </div>
              {
                  if filtered_tasks.is_empty() {
                      html! {
                          <div class="calendar-empty">
                              { "No tasks due in this calendar period for the selected calendar filter." }
                          </div>
                      }
                  } else {
                      html! {
                          <>
                              {
                                  for filtered_tasks.iter().cloned().map(|entry| html! {
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
