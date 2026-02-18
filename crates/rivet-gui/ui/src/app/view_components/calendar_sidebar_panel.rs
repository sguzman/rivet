#[derive(Properties, PartialEq)]
struct CalendarSidebarPanelProps {
  view:               CalendarViewMode,
  on_set_view:
    Callback<CalendarViewMode>,
  on_prev:            Callback<MouseEvent>,
  on_today:           Callback<MouseEvent>,
  on_next:            Callback<MouseEvent>,
  timezone:           Tz,
  focus_date:         NaiveDate,
  due_count:          usize,
  external_sources:   Vec<ExternalCalendarSource>,
  external_busy:      bool,
  external_last_sync: Option<String>,
  on_add_source:
    Callback<MouseEvent>,
  on_sync_all_sources:
    Callback<MouseEvent>,
  on_import_file:
    Callback<web_sys::Event>,
  on_sync_source:
    Callback<String>,
  on_edit_source:
    Callback<ExternalCalendarSource>,
  on_delete_source:
    Callback<String>
}

#[function_component(CalendarSidebarPanel)]
fn calendar_sidebar_panel(
  props: &CalendarSidebarPanelProps
) -> Html {
  html! {
      <div class="panel calendar-sidebar">
          <div class="header">{ "Calendar Views" }</div>
          <div class="details">
              <CalendarViewSwitch
                  current_view={props.view}
                  on_set_view={props.on_set_view.clone()}
              />
              <CalendarNavActions
                  on_prev={props.on_prev.clone()}
                  on_today={props.on_today.clone()}
                  on_next={props.on_next.clone()}
              />
              <CalendarMetaSummary
                  timezone={props.timezone}
                  focus_date={props.focus_date}
                  due_count={props.due_count}
              />
              <CalendarMarkerLegend />
              <ExternalCalendarsPanel
                  sources={props.external_sources.clone()}
                  busy={props.external_busy}
                  last_sync={props.external_last_sync.clone()}
                  on_add={props.on_add_source.clone()}
                  on_sync_all={props.on_sync_all_sources.clone()}
                  on_import_file={props.on_import_file.clone()}
                  on_sync_one={props.on_sync_source.clone()}
                  on_edit={props.on_edit_source.clone()}
                  on_delete={props.on_delete_source.clone()}
              />
          </div>
      </div>
  }
}
