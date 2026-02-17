#[derive(Properties, PartialEq)]
struct ExternalCalendarsPanelProps {
  sources:     Vec<ExternalCalendarSource>,
  busy:        bool,
  last_sync:   Option<String>,
  on_add:      Callback<MouseEvent>,
  on_sync_all: Callback<MouseEvent>,
  on_sync_one: Callback<String>,
  on_edit:     Callback<ExternalCalendarSource>,
  on_delete:   Callback<String>
}

#[function_component(ExternalCalendarsPanel)]
fn external_calendars_panel(
  props: &ExternalCalendarsPanelProps
) -> Html {
  html! {
      <>
          <div class="calendar-external-header">{ "External Calendars" }</div>
          <div class="actions">
              <button class="btn" onclick={props.on_add.clone()}>{ "Add Source" }</button>
              <button class="btn" onclick={props.on_sync_all.clone()} disabled={props.busy}>
                  { if props.busy { "Syncing..." } else { "Sync Enabled" } }
              </button>
          </div>
          {
              if let Some(last_sync) = props.last_sync.clone() {
                  html! { <div class="field-help">{ last_sync }</div> }
              } else {
                  html! {}
              }
          }
          <ExternalCalendarSourceList
              sources={props.sources.clone()}
              busy={props.busy}
              on_sync={props.on_sync_one.clone()}
              on_edit={props.on_edit.clone()}
              on_delete={props.on_delete.clone()}
          />
      </>
  }
}
