#[derive(Properties, PartialEq)]
struct ExternalCalendarSourceCardProps {
  source: ExternalCalendarSource,
  busy:   bool,
  on_sync:
    Callback<String>,
  on_edit:
    Callback<ExternalCalendarSource>,
  on_delete:
    Callback<String>
}

#[function_component(ExternalCalendarSourceCard)]
fn external_calendar_source_card(
  props: &ExternalCalendarSourceCardProps
) -> Html {
  let source_id = props.source.id.clone();
  let source_id_for_sync = source_id.clone();
  let source_id_for_delete = source_id.clone();
  let source_for_edit = props.source.clone();
  let color_style =
    format!("background:{};", props.source.color);

  html! {
      <div class="calendar-source-item">
          <div class="calendar-source-top">
              <span class="calendar-source-color" style={color_style}></span>
              <span class="calendar-source-name">{ &props.source.name }</span>
              {
                  if props.source.enabled {
                      html! { <span class="badge">{ "enabled" }</span> }
                  } else {
                      html! { <span class="badge">{ "disabled" }</span> }
                  }
              }
          </div>
          <div class="task-subtitle">{ &props.source.location }</div>
          <div class="calendar-source-meta">
              <span class="badge">{ format!("refresh:{}m", props.source.refresh_minutes) }</span>
              {
                  if props.source.show_reminders {
                      html! { <span class="badge">{ "reminders:on" }</span> }
                  } else {
                      html! {}
                  }
              }
              {
                  if props.source.offline_support {
                      html! { <span class="badge">{ "offline:on" }</span> }
                  } else {
                      html! {}
                  }
              }
          </div>
          <div class="actions">
              <button class="btn" onclick={{
                  let on_sync = props.on_sync.clone();
                  Callback::from(move |_| on_sync.emit(source_id_for_sync.clone()))
              }} disabled={props.busy}>
                  { "Sync" }
              </button>
              <button class="btn" onclick={{
                  let on_edit = props.on_edit.clone();
                  Callback::from(move |_| on_edit.emit(source_for_edit.clone()))
              }}>
                  { "Edit" }
              </button>
              <button class="btn danger" onclick={{
                  let on_delete = props.on_delete.clone();
                  Callback::from(move |_| on_delete.emit(source_id_for_delete.clone()))
              }}>
                  { "Delete" }
              </button>
          </div>
      </div>
  }
}
