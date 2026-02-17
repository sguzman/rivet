#[derive(Properties, PartialEq)]
struct ExternalCalendarSourceListProps {
  sources: Vec<ExternalCalendarSource>,
  busy:    bool,
  on_sync:
    Callback<String>,
  on_edit:
    Callback<ExternalCalendarSource>,
  on_delete:
    Callback<String>
}

#[function_component(ExternalCalendarSourceList)]
fn external_calendar_source_list(
  props: &ExternalCalendarSourceListProps
) -> Html {
  html! {
      <div class="calendar-source-list">
          {
              if props.sources.is_empty() {
                  html! { <div class="calendar-empty">{ "No external calendar sources configured." }</div> }
              } else {
                  html! {
                      <>
                          {
                              for props.sources.iter().cloned().map(|source| html! {
                                  <ExternalCalendarSourceCard
                                      source={source}
                                      busy={props.busy}
                                      on_sync={props.on_sync.clone()}
                                      on_edit={props.on_edit.clone()}
                                      on_delete={props.on_delete.clone()}
                                  />
                              })
                          }
                      </>
                  }
              }
          }
      </div>
  }
}
