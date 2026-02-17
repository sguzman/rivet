#[derive(Properties, PartialEq)]
struct CalendarCanvasPanelProps {
  title:              String,
  view:               CalendarViewMode,
  focus_date:         NaiveDate,
  week_start:         Weekday,
  due_tasks:          Vec<CalendarDueTask>,
  config:             CalendarConfig,
  tag_colors:
    BTreeMap<String, String>,
  on_navigate:
    Callback<(NaiveDate, CalendarViewMode)>
}

#[function_component(CalendarCanvasPanel)]
fn calendar_canvas_panel(
  props: &CalendarCanvasPanelProps
) -> Html {
  html! {
      <div class="panel calendar-panel">
          <div class="header">{ props.title.clone() }</div>
          <div class="details calendar-content">
              {
                  render_calendar_view(
                      props.view,
                      props.focus_date,
                      props.week_start,
                      &props.due_tasks,
                      &props.config,
                      &props.tag_colors,
                      props.on_navigate.clone(),
                  )
              }
          </div>
      </div>
  }
}
