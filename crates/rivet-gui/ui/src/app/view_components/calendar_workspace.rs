#[derive(Properties, PartialEq)]
struct CalendarWorkspaceProps {
  calendar_view:       CalendarViewMode,
  on_calendar_set_view:
    Callback<CalendarViewMode>,
  on_calendar_prev:    Callback<MouseEvent>,
  on_calendar_today:   Callback<MouseEvent>,
  on_calendar_next:    Callback<MouseEvent>,
  calendar_timezone:   Tz,
  calendar_focus_date: NaiveDate,
  calendar_due_tasks:  Vec<CalendarDueTask>,
  external_calendars:  Vec<ExternalCalendarSource>,
  external_busy:       bool,
  external_last_sync:  Option<String>,
  on_open_add_external_calendar:
    Callback<MouseEvent>,
  on_sync_all_external_calendars:
    Callback<MouseEvent>,
  on_import_external_calendar_file:
    Callback<web_sys::Event>,
  on_sync_external_calendar:
    Callback<String>,
  on_open_edit_external_calendar:
    Callback<ExternalCalendarSource>,
  on_delete_external_calendar:
    Callback<String>,
  calendar_title:      String,
  calendar_week_start: Weekday,
  calendar_config:     CalendarConfig,
  tag_colors:
    BTreeMap<String, String>,
  on_calendar_navigate:
    Callback<(NaiveDate, CalendarViewMode)>,
  calendar_period_stats:
    CalendarStats,
  calendar_period_tasks:
    Vec<CalendarDueTask>
}

#[function_component(CalendarWorkspace)]
fn calendar_workspace(
  props: &CalendarWorkspaceProps
) -> Html {
  html! {
      <>
          <CalendarSidebarPanel
              view={props.calendar_view}
              on_set_view={props.on_calendar_set_view.clone()}
              on_prev={props.on_calendar_prev.clone()}
              on_today={props.on_calendar_today.clone()}
              on_next={props.on_calendar_next.clone()}
              timezone={props.calendar_timezone}
              focus_date={props.calendar_focus_date}
              due_count={props.calendar_due_tasks.len()}
              external_sources={props.external_calendars.clone()}
              external_busy={props.external_busy}
              external_last_sync={props.external_last_sync.clone()}
              on_add_source={props.on_open_add_external_calendar.clone()}
              on_sync_all_sources={props.on_sync_all_external_calendars.clone()}
              on_import_file={props.on_import_external_calendar_file.clone()}
              on_sync_source={props.on_sync_external_calendar.clone()}
              on_edit_source={props.on_open_edit_external_calendar.clone()}
              on_delete_source={props.on_delete_external_calendar.clone()}
          />

          <CalendarCanvasPanel
              title={props.calendar_title.clone()}
              view={props.calendar_view}
              focus_date={props.calendar_focus_date}
              week_start={props.calendar_week_start}
              due_tasks={props.calendar_due_tasks.clone()}
              config={props.calendar_config.clone()}
              tag_colors={props.tag_colors.clone()}
              on_navigate={props.on_calendar_navigate.clone()}
          />

          <div class="right-stack">
              <CalendarStatsPanel
                  stats={props.calendar_period_stats.clone()}
              />
              <CalendarPeriodTaskListPanel
                  tasks={props.calendar_period_tasks.clone()}
                  timezone={props.calendar_timezone}
                  tag_colors={props.tag_colors.clone()}
              />
          </div>
      </>
  }
}
