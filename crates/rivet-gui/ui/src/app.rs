use std::collections::{
  BTreeMap,
  BTreeSet
};

use chrono::{
  DateTime,
  Datelike,
  Duration,
  NaiveDate,
  NaiveDateTime,
  Timelike,
  Utc,
  Weekday
};
use chrono_tz::Tz;
use gloo::console::log;
use gloo::timers::callback::Interval;
use gloo::timers::future::TimeoutFuture;
use rivet_gui_shared::{
  TaskCreate,
  TaskDto,
  TaskIdArg,
  TaskPatch,
  TaskStatus,
  TaskUpdateArgs,
  TasksListArgs
};
use serde::{
  Deserialize,
  Serialize
};
use uuid::Uuid;
use yew::{
  Callback,
  Html,
  TargetCast,
  classes,
  function_component,
  html,
  use_effect_with,
  use_state
};

use crate::api::invoke_tauri;
use crate::components::{
  Details,
  FacetPanel,
  KanbanBoard,
  Sidebar,
  TaskList
};

#[derive(Clone, PartialEq)]
struct ModalState {
  mode:                 ModalMode,
  draft_title:          String,
  draft_desc:           String,
  draft_project:        String,
  draft_board_id:       Option<String>,
  lock_board_selection: bool,
  draft_custom_tag:     String,
  draft_tags:           Vec<String>,
  picker_key:           Option<String>,
  picker_value:         Option<String>,
  draft_due:            String,
  recurrence_pattern:   String,
  recurrence_time:      String,
  recurrence_days:      Vec<String>,
  recurrence_months:    Vec<String>,
  recurrence_month_day: String,
  error:                Option<String>
}

#[derive(Clone, PartialEq)]
enum ModalMode {
  Add,
  Edit(Uuid)
}

#[derive(
  Clone, PartialEq, Deserialize,
)]
struct TagSchema {
  #[serde(default)]
  version: u32,
  #[serde(default)]
  keys:    Vec<TagKey>
}

#[derive(
  Clone, PartialEq, Deserialize,
)]
struct TagKey {
  id:                  String,
  label:               Option<String>,
  selection:           Option<String>,
  color:               Option<String>,
  #[serde(default)]
  allow_custom_values: bool,
  #[serde(default)]
  values:              Vec<String>
}

#[derive(
  Clone,
  PartialEq,
  Eq,
  Serialize,
  Deserialize,
)]
struct KanbanBoardDef {
  id:    String,
  name:  String,
  #[serde(
    default = "default_board_color"
  )]
  color: String
}

#[derive(
  Clone,
  PartialEq,
  Eq,
  Serialize,
  Deserialize,
)]
struct ExternalCalendarSource {
  id:              String,
  name:            String,
  color:           String,
  location:        String,
  refresh_minutes: u32,
  enabled:         bool,
  read_only:       bool,
  show_reminders:  bool,
  offline_support: bool
}

#[derive(
  Clone, PartialEq, Deserialize,
)]
struct ExternalCalendarSyncResult {
  calendar_id:     String,
  created:         usize,
  updated:         usize,
  deleted:         usize,
  remote_events:   usize,
  refresh_minutes: u32
}

#[derive(Clone, PartialEq)]
struct ExternalCalendarModalState {
  mode:   ExternalCalendarModalMode,
  source: ExternalCalendarSource,
  error:  Option<String>
}

#[derive(
  Clone, Copy, PartialEq, Eq,
)]
enum ExternalCalendarModalMode {
  Add,
  Edit
}

impl TagSchema {
  fn key(
    &self,
    id: &str
  ) -> Option<&TagKey> {
    self
      .keys
      .iter()
      .find(|key| key.id == id)
  }

  fn default_picker(
    &self
  ) -> (Option<String>, Option<String>)
  {
    let Some(key) = self.keys.first()
    else {
      return (None, None);
    };
    let value =
      key.values.first().cloned();
    (Some(key.id.clone()), value)
  }
}

impl Default for TagSchema {
  fn default() -> Self {
    Self {
      version: 1,
      keys:    vec![
        TagKey {
          id:                  "area"
            .to_string(),
          label:               Some(
            "Area".to_string()
          ),
          selection:           Some(
            "single".to_string()
          ),
          color:               Some(
            "#4B7BEC".to_string()
          ),
          allow_custom_values: false,
          values:              vec![
            "software".to_string(),
            "research".to_string(),
            "learning".to_string(),
            "library".to_string(),
            "admin".to_string(),
            "family".to_string(),
            "farm".to_string(),
            "home".to_string(),
            "health".to_string(),
          ]
        },
        TagKey {
          id:                  "stage"
            .to_string(),
          label:               Some(
            "Stage".to_string()
          ),
          selection:           Some(
            "single".to_string()
          ),
          color:               Some(
            "#8854D0".to_string()
          ),
          allow_custom_values: false,
          values:              vec![
            "inbox".to_string(),
            "idea".to_string(),
            "planned".to_string(),
            "active".to_string(),
            "waiting".to_string(),
            "paused".to_string(),
            "done".to_string(),
            "archived".to_string(),
          ]
        },
        TagKey {
          id:                  "kanban"
            .to_string(),
          label:               Some(
            "Kanban Lane".to_string()
          ),
          selection:           Some(
            "single".to_string()
          ),
          color:               Some(
            "#4A90E2".to_string()
          ),
          allow_custom_values: false,
          values:              vec![
            "todo".to_string(),
            "working".to_string(),
            "finished".to_string(),
          ]
        },
        TagKey {
          id:                  "board"
            .to_string(),
          label:               Some(
            "Kanban Board".to_string()
          ),
          selection:           Some(
            "single".to_string()
          ),
          color:               Some(
            "#3B7A57".to_string()
          ),
          allow_custom_values: true,
          values:              vec![]
        },
      ]
    }
  }
}

#[derive(
  Clone, Copy, PartialEq, Eq,
)]
enum ThemeMode {
  Day,
  Night
}

impl ThemeMode {
  fn as_class(self) -> &'static str {
    match self {
      | Self::Day => "theme-day",
      | Self::Night => "theme-night"
    }
  }

  fn next(self) -> Self {
    match self {
      | Self::Day => Self::Night,
      | Self::Night => Self::Day
    }
  }

  fn storage_value(
    self
  ) -> &'static str {
    match self {
      | Self::Day => "day",
      | Self::Night => "night"
    }
  }

  fn toggle_label(
    self
  ) -> &'static str {
    match self {
      | Self::Day => "Night",
      | Self::Night => "Day"
    }
  }
}

#[derive(
  Clone, Copy, PartialEq, Eq,
)]
enum CalendarViewMode {
  Year,
  Quarter,
  Month,
  Week,
  Day,
  List
}

impl CalendarViewMode {
  fn all() -> [Self; 5] {
    [
      Self::Year,
      Self::Quarter,
      Self::Month,
      Self::Week,
      Self::Day
    ]
  }

  fn as_key(self) -> &'static str {
    match self {
      | Self::Year => "year",
      | Self::Quarter => "quarter",
      | Self::Month => "month",
      | Self::Week => "week",
      | Self::Day => "day",
      | Self::List => "list"
    }
  }

  fn label(self) -> &'static str {
    match self {
      | Self::Year => "Year",
      | Self::Quarter => "Quarter",
      | Self::Month => "Month",
      | Self::Week => "Week",
      | Self::Day => "Day",
      | Self::List => "Task List"
    }
  }

  fn from_key(
    key: &str
  ) -> Option<Self> {
    match key {
      | "year" => Some(Self::Year),
      | "quarter" => {
        Some(Self::Quarter)
      }
      | "month" => Some(Self::Month),
      | "week" => Some(Self::Week),
      | "day" => Some(Self::Day),
      | "list" => Some(Self::List),
      | _ => None
    }
  }
}

#[derive(
  Clone, PartialEq, Deserialize,
)]
struct CalendarConfig {
  #[serde(default)]
  version:    u32,
  timezone:   Option<String>,
  #[serde(default)]
  policies:   CalendarPolicies,
  #[serde(default)]
  visibility: CalendarVisibility,
  #[serde(default)]
  day_view:   CalendarDayView
}

#[derive(
  Clone, PartialEq, Deserialize,
)]
struct CalendarPolicies {
  #[serde(
    default = "calendar_default_week_start"
  )]
  week_start:            String,
  #[serde(
    default = "calendar_default_red_dot_limit"
  )]
  red_dot_limit:         usize,
  #[serde(
    default = "calendar_default_task_list_limit"
  )]
  task_list_limit:       usize,
  #[serde(
    default = "calendar_default_task_list_window_days"
  )]
  task_list_window_days: i64
}

#[derive(
  Clone, PartialEq, Deserialize,
)]
struct CalendarVisibility {
  #[serde(default = "calendar_true")]
  pending:   bool,
  #[serde(default = "calendar_true")]
  waiting:   bool,
  #[serde(default = "calendar_true")]
  completed: bool,
  #[serde(default = "calendar_true")]
  deleted:   bool
}

#[derive(
  Clone, PartialEq, Deserialize,
)]
struct CalendarDayView {
  #[serde(default)]
  hour_start: u32,
  #[serde(
    default = "calendar_default_day_view_hour_end"
  )]
  hour_end:   u32
}

#[derive(Deserialize)]
struct ProjectTimeConfig {
  timezone: Option<String>,
  time:     Option<ProjectTimeSection>
}

#[derive(Deserialize)]
struct ProjectTimeSection {
  timezone: Option<String>
}

#[derive(Clone)]
struct CalendarDueTask {
  task:      TaskDto,
  due_utc:   DateTime<Utc>,
  due_local: DateTime<Tz>,
  marker:    CalendarTaskMarker
}

#[derive(Clone, Copy)]
enum CalendarMarkerShape {
  Triangle,
  Circle,
  Square
}

impl CalendarMarkerShape {
  fn as_class(self) -> &'static str {
    match self {
      | Self::Triangle => "triangle",
      | Self::Circle => "circle",
      | Self::Square => "square"
    }
  }
}

#[derive(Clone)]
struct CalendarTaskMarker {
  shape: CalendarMarkerShape,
  color: String
}

#[derive(Default)]
struct CalendarStats {
  total:     usize,
  pending:   usize,
  waiting:   usize,
  completed: usize,
  deleted:   usize
}

impl CalendarStats {
  fn push(
    &mut self,
    status: &TaskStatus
  ) {
    self.total =
      self.total.saturating_add(1);
    match status {
      | TaskStatus::Pending => {
        self.pending = self
          .pending
          .saturating_add(1);
      }
      | TaskStatus::Waiting => {
        self.waiting = self
          .waiting
          .saturating_add(1);
      }
      | TaskStatus::Completed => {
        self.completed = self
          .completed
          .saturating_add(1);
      }
      | TaskStatus::Deleted => {
        self.deleted = self
          .deleted
          .saturating_add(1);
      }
    }
  }
}

impl Default for CalendarConfig {
  fn default() -> Self {
    Self {
      version:    1,
      timezone:   Some(
        DEFAULT_CALENDAR_TIMEZONE
          .to_string()
      ),
      policies:
        CalendarPolicies::default(),
      visibility:
        CalendarVisibility::default(),
      day_view:
        CalendarDayView::default()
    }
  }
}

impl Default for CalendarPolicies {
  fn default() -> Self {
    Self {
      week_start:
        calendar_default_week_start(),
      red_dot_limit:
        calendar_default_red_dot_limit(),
      task_list_limit:
        calendar_default_task_list_limit(),
      task_list_window_days:
        calendar_default_task_list_window_days(
        )
    }
  }
}

impl Default for CalendarVisibility {
  fn default() -> Self {
    Self {
      pending:   true,
      waiting:   true,
      completed: true,
      deleted:   true
    }
  }
}

impl Default for CalendarDayView {
  fn default() -> Self {
    Self {
      hour_start: 0,
      hour_end:
        calendar_default_day_view_hour_end(
        )
    }
  }
}

const THEME_STORAGE_KEY: &str =
  "rivet.theme";
const WORKSPACE_TAB_STORAGE_KEY: &str =
  "rivet.workspace_tab";
const CALENDAR_VIEW_STORAGE_KEY: &str =
  "rivet.calendar.view";
const EXTERNAL_CALENDARS_STORAGE_KEY:
  &str = "rivet.external_calendars";
const KANBAN_BOARDS_STORAGE_KEY: &str =
  "rivet.kanban.boards";
const KANBAN_ACTIVE_BOARD_STORAGE_KEY:
  &str = "rivet.kanban.active_board";
const TAG_SCHEMA_TOML: &str =
  include_str!("../assets/tags.toml");
const CALENDAR_CONFIG_TOML: &str = include_str!(
  "../assets/calendar.toml"
);
const PROJECT_TIME_CONFIG_TOML: &str = include_str!(
  "../../../../rivet-time.toml"
);
const DEFAULT_CALENDAR_TIMEZONE: &str =
  "America/Mexico_City";
const KANBAN_TAG_KEY: &str = "kanban";
const BOARD_TAG_KEY: &str = "board";
const RECUR_TAG_KEY: &str = "recur";
const RECUR_TIME_TAG_KEY: &str =
  "recur_time";
const RECUR_DAYS_TAG_KEY: &str =
  "recur_days";
const RECUR_MONTHS_TAG_KEY: &str =
  "recur_months";
const RECUR_MONTH_DAY_TAG_KEY: &str =
  "recur_day";
const CAL_SOURCE_TAG_KEY: &str =
  "cal_source";
const CAL_COLOR_TAG_KEY: &str =
  "cal_color";
const CALENDAR_UNAFFILIATED_COLOR:
  &str = "#7f8691";

const WEEKDAY_KEYS: [&str; 7] = [
  "mon", "tue", "wed", "thu", "fri",
  "sat", "sun"
];

const MONTH_KEYS: [&str; 12] = [
  "jan", "feb", "mar", "apr", "may",
  "jun", "jul", "aug", "sep", "oct",
  "nov", "dec"
];

#[function_component(App)]
pub fn app() -> Html {
  let theme =
    use_state(load_theme_mode);
  let active_tab =
    use_state(load_workspace_tab);
  let tag_schema =
    use_state(load_tag_schema);
  let calendar_config =
    use_state(load_calendar_config);
  let calendar_view =
    use_state(load_calendar_view_mode);
  let calendar_focus_date = {
    let config_snapshot =
      (*calendar_config).clone();
    use_state(move || {
      today_in_timezone(
        resolve_calendar_timezone(
          &config_snapshot
        )
      )
    })
  };
  let active_view =
    use_state(|| "all".to_string());
  let kanban_boards =
    use_state(load_kanban_boards);
  let active_kanban_board = {
    let boards_snapshot =
      (*kanban_boards).clone();
    use_state(move || {
      load_active_kanban_board(
        &boards_snapshot
      )
    })
  };
  let dragging_kanban_task =
    use_state(|| None::<Uuid>);
  let drag_over_kanban_lane =
    use_state(|| None::<String>);
  let kanban_rename_open =
    use_state(|| false);
  let kanban_rename_input =
    use_state(String::new);
  let kanban_create_open =
    use_state(|| false);
  let kanban_create_input =
    use_state(String::new);
  let kanban_compact_cards =
    use_state(|| false);
  let external_calendars =
    use_state(load_external_calendars);
  let external_calendar_modal =
    use_state(|| {
      None::<ExternalCalendarModalState>
    });
  let external_calendar_busy =
    use_state(|| false);
  let external_calendar_last_sync =
    use_state(|| None::<String>);
  let search = use_state(String::new);
  let refresh_tick =
    use_state(|| 0_u64);

  let tasks =
    use_state(Vec::<TaskDto>::new);
  let facet_tasks =
    use_state(Vec::<TaskDto>::new);
  let selected =
    use_state(|| None::<Uuid>);
  let bulk_selected =
    use_state(BTreeSet::<Uuid>::new);
  let active_project =
    use_state(|| None::<String>);
  let active_tag =
    use_state(|| None::<String>);
  let all_filter_project =
    use_state(|| None::<String>);
  let all_filter_tag =
    use_state(|| None::<String>);
  let all_filter_completion =
    use_state(|| "all".to_string());
  let all_filter_priority =
    use_state(|| "all".to_string());
  let all_filter_due =
    use_state(|| "all".to_string());
  let modal_state =
    use_state(|| None::<ModalState>);
  let modal_busy = use_state(|| false);
  let modal_submit_seq =
    use_state(|| 0_u64);

  {
    use_effect_with((), move |_| {
      ui_debug(
        "app.mounted",
        "frontend mounted and hooks \
         initialized"
      );
      || ()
    });
  }

  {
    let active_tab = active_tab.clone();
    use_effect_with(
      (*active_tab).clone(),
      move |tab| {
        save_workspace_tab(tab);
        tracing::debug!(
          tab = %tab,
          "persisted workspace tab"
        );
        || ()
      }
    );
  }

  {
    let calendar_view =
      calendar_view.clone();
    use_effect_with(
      *calendar_view,
      move |view| {
        save_calendar_view_mode(*view);
        tracing::debug!(
          view = %view.as_key(),
          "persisted calendar view mode"
        );
        || ()
      }
    );
  }

  {
    let external_calendars =
      external_calendars.clone();
    use_effect_with(
      (*external_calendars).clone(),
      move |calendars| {
        save_external_calendars(
          calendars
        );
        tracing::debug!(
          calendar_sources =
            calendars.len(),
          "persisted external \
           calendar sources"
        );
        || ()
      }
    );
  }

  {
    let external_calendars =
      external_calendars.clone();
    let refresh_tick =
      refresh_tick.clone();
    let external_calendar_last_sync =
      external_calendar_last_sync
        .clone();
    use_effect_with(
      (*external_calendars).clone(),
      move |sources| {
        let mut intervals = Vec::new();

        for source in sources
          .iter()
          .cloned()
          .filter(|source| {
            source.enabled
              && source.refresh_minutes
                > 0
          })
        {
          let refresh_tick =
            refresh_tick.clone();
          let external_calendar_last_sync =
            external_calendar_last_sync
              .clone();
          let period_ms = source
            .refresh_minutes
            .saturating_mul(60_000);

          intervals.push(Interval::new(
            period_ms,
            move || {
              let source =
                source.clone();
              let refresh_tick =
                refresh_tick.clone();
              let external_calendar_last_sync =
                external_calendar_last_sync
                  .clone();
              wasm_bindgen_futures::spawn_local(async move {
                match invoke_tauri::<
                  ExternalCalendarSyncResult,
                  _
                >(
                  "external_calendar_sync",
                  &source
                )
                .await
                {
                  | Ok(result) => {
                    tracing::info!(
                      calendar_id = %result.calendar_id,
                      created = result.created,
                      updated = result.updated,
                      deleted = result.deleted,
                      "external calendar auto sync succeeded"
                    );
                    external_calendar_last_sync
                      .set(Some(format!(
                        "Synced {}: +{} / ~{} / -{}",
                        source.name,
                        result.created,
                        result.updated,
                        result.deleted
                      )));
                    refresh_tick.set(
                      (*refresh_tick)
                        .saturating_add(1),
                    );
                  }
                  | Err(err) => {
                    tracing::error!(
                      calendar = %source.name,
                      error = %err,
                      "external calendar auto sync failed"
                    );
                    external_calendar_last_sync
                      .set(Some(format!(
                        "Sync failed for {}: {}",
                        source.name, err
                      )));
                  }
                }
              });
            },
          ));
        }

        move || drop(intervals)
      }
    );
  }

  {
    let kanban_boards =
      kanban_boards.clone();
    use_effect_with(
      (*kanban_boards).clone(),
      move |boards| {
        save_kanban_boards(boards);
        tracing::debug!(
          board_count = boards.len(),
          "persisted kanban boards"
        );
        || ()
      }
    );
  }

  {
    let active_kanban_board =
      active_kanban_board.clone();
    use_effect_with(
      (*active_kanban_board).clone(),
      move |active| {
        save_active_kanban_board(
          active.as_deref()
        );
        tracing::debug!(
          active_board = ?active,
          "persisted active kanban \
           board"
        );
        || ()
      }
    );
  }

  {
    let kanban_boards =
      kanban_boards.clone();
    let active_kanban_board =
      active_kanban_board.clone();
    use_effect_with(
      (
        (*kanban_boards).clone(),
        (*active_kanban_board).clone()
      ),
      move |(boards, active)| {
        let contains_active = active
          .as_ref()
          .is_some_and(|id| {
            boards.iter().any(|board| {
              &board.id == id
            })
          });

        if !contains_active {
          let next =
            boards.first().map(
              |board| board.id.clone()
            );
          if next
            != *active_kanban_board
          {
            tracing::info!(
              active_board = ?active,
              next_board = ?next,
              "repairing active kanban \
               board selection"
            );
            active_kanban_board
              .set(next);
          }
        }

        || ()
      }
    );
  }

  {
    let active_tab = active_tab.clone();
    let active_view =
      active_view.clone();
    let refresh_tick =
      refresh_tick.clone();
    let tasks = tasks.clone();

    use_effect_with(
      (
        (*active_tab).clone(),
        (*active_view).clone(),
        *refresh_tick
      ),
      move |(tab, view, tick)| {
        let tasks = tasks.clone();
        let tab = tab.clone();
        let view = view.clone();
        let tick = *tick;

        wasm_bindgen_futures::spawn_local(async move {
                    tracing::info!(tab = %tab, view = %view, tick, "refreshing task list");

                    let status = if tab == "kanban"
                        || tab == "calendar"
                        || view == "all"
                    {
                        None
                    } else {
                        Some(TaskStatus::Pending)
                    };

                    let args = TasksListArgs {
                        query: None,
                        status,
                        project: None,
                        tag: None,
                    };

                    match invoke_tauri::<Vec<TaskDto>, _>("tasks_list", &args).await {
                        Ok(list) => tasks.set(list),
                        Err(err) => tracing::error!(error = %err, "tasks_list failed"),
                    }
                });

        || ()
      }
    );
  }

  {
    let refresh_tick =
      refresh_tick.clone();
    let facet_tasks =
      facet_tasks.clone();

    use_effect_with(
      *refresh_tick,
      move |_| {
        let facet_tasks =
          facet_tasks.clone();

        wasm_bindgen_futures::spawn_local(
          async move {
            let args = TasksListArgs {
              query: None,
              status: None,
              project: None,
              tag: None
            };

            match invoke_tauri::<Vec<TaskDto>, _>(
              "tasks_list",
              &args
            )
            .await
            {
              | Ok(list) => {
                tracing::debug!(
                  total = list.len(),
                  "refreshed facet task \
                   snapshot"
                );
                facet_tasks.set(list);
              }
              | Err(err) => tracing::error!(error = %err, "facet tasks refresh failed")
            }
          }
        );

        || ()
      }
    );
  }

  let task_visible_tasks = {
    let query = (*search).clone();
    filter_visible_tasks(
      &tasks,
      &active_view,
      &query,
      active_project.as_deref(),
      active_tag.as_deref(),
      all_filter_completion.as_str(),
      all_filter_project.as_deref(),
      all_filter_tag.as_deref(),
      all_filter_priority.as_str(),
      all_filter_due.as_str()
    )
  };
  let tag_colors =
    build_tag_color_map(&tag_schema);
  let kanban_board_color_map =
    build_kanban_board_color_map(
      &kanban_boards
    );
  let external_calendar_color_map =
    build_external_calendar_color_map(
      &external_calendars
    );
  let kanban_columns =
    kanban_columns_from_schema(
      &tag_schema
    );
  let default_kanban_lane =
    kanban_columns
      .first()
      .cloned()
      .unwrap_or_else(|| {
        "todo".to_string()
      });

  let kanban_visible_tasks = {
    let base = filter_visible_tasks(
      &tasks,
      "kanban",
      "",
      None,
      None,
      all_filter_completion.as_str(),
      all_filter_project.as_deref(),
      all_filter_tag.as_deref(),
      all_filter_priority.as_str(),
      all_filter_due.as_str()
    );

    if let Some(board_id) =
      (*active_kanban_board).clone()
    {
      base
        .into_iter()
        .filter(|task| {
          task_has_tag_value(
            &task.tags,
            BOARD_TAG_KEY,
            &board_id
          )
        })
        .collect()
    } else {
      Vec::new()
    }
  };

  let selected_task = (*selected)
    .and_then(|id| {
      task_visible_tasks
        .iter()
        .find(|task| task.uuid == id)
        .cloned()
    });

  let project_facets =
    build_project_facets(&facet_tasks);
  let tag_facets =
    build_tag_facets(&facet_tasks);
  let calendar_timezone =
    resolve_calendar_timezone(
      &calendar_config
    );
  let calendar_week_start =
    calendar_week_start_day(
      &calendar_config
        .policies
        .week_start
    );
  let calendar_due_tasks =
    collect_calendar_due_tasks(
      &facet_tasks,
      calendar_timezone,
      &calendar_config,
      &kanban_board_color_map,
      &external_calendar_color_map
    );
  let calendar_period_stats =
    summarize_calendar_period(
      &calendar_due_tasks,
      *calendar_view,
      *calendar_focus_date,
      calendar_week_start,
      &calendar_config
    );
  let calendar_period_tasks =
    collect_calendar_period_tasks(
      &calendar_due_tasks,
      *calendar_view,
      *calendar_focus_date,
      calendar_week_start,
      &calendar_config
    );
  let calendar_title =
    calendar_title_for_view(
      *calendar_view,
      *calendar_focus_date,
      calendar_week_start
    );

  let on_nav = {
    let active_view =
      active_view.clone();
    let search = search.clone();
    let selected = selected.clone();
    let bulk_selected =
      bulk_selected.clone();
    let active_project =
      active_project.clone();
    let active_tag = active_tag.clone();
    let all_filter_project =
      all_filter_project.clone();
    let all_filter_tag =
      all_filter_tag.clone();
    let all_filter_completion =
      all_filter_completion.clone();
    let all_filter_priority =
      all_filter_priority.clone();
    let all_filter_due =
      all_filter_due.clone();
    Callback::from(
      move |view: String| {
        if view != "all" {
          search.set(String::new());
        }
        active_view.set(view);
        selected.set(None);
        bulk_selected
          .set(BTreeSet::new());
        active_project.set(None);
        active_tag.set(None);
        all_filter_project.set(None);
        all_filter_tag.set(None);
        all_filter_completion
          .set("all".to_string());
        all_filter_priority
          .set("all".to_string());
        all_filter_due
          .set("all".to_string());
      }
    )
  };

  let on_select_tasks_tab = {
    let active_tab = active_tab.clone();
    let selected = selected.clone();
    let bulk_selected =
      bulk_selected.clone();
    let dragging_kanban_task =
      dragging_kanban_task.clone();
    let drag_over_kanban_lane =
      drag_over_kanban_lane.clone();
    Callback::from(move |_| {
      active_tab
        .set("tasks".to_string());
      selected.set(None);
      bulk_selected
        .set(BTreeSet::new());
      dragging_kanban_task.set(None);
      drag_over_kanban_lane.set(None);
    })
  };

  let on_select_kanban_tab = {
    let active_tab = active_tab.clone();
    let selected = selected.clone();
    let bulk_selected =
      bulk_selected.clone();
    Callback::from(move |_| {
      active_tab
        .set("kanban".to_string());
      selected.set(None);
      bulk_selected
        .set(BTreeSet::new());
    })
  };

  let on_select_calendar_tab = {
    let active_tab = active_tab.clone();
    let selected = selected.clone();
    let bulk_selected =
      bulk_selected.clone();
    let dragging_kanban_task =
      dragging_kanban_task.clone();
    let drag_over_kanban_lane =
      drag_over_kanban_lane.clone();
    Callback::from(move |_| {
      active_tab
        .set("calendar".to_string());
      selected.set(None);
      bulk_selected
        .set(BTreeSet::new());
      dragging_kanban_task.set(None);
      drag_over_kanban_lane.set(None);
    })
  };

  let on_select = {
    let selected = selected.clone();
    Callback::from(move |id: Uuid| {
      selected.set(Some(id))
    })
  };

  let on_toggle_select = {
    let bulk_selected =
      bulk_selected.clone();
    Callback::from(move |id: Uuid| {
      let mut next =
        (*bulk_selected).clone();
      if next.contains(&id) {
        next.remove(&id);
      } else {
        next.insert(id);
      }
      bulk_selected.set(next);
    })
  };

  let on_choose_project = {
    let active_project =
      active_project.clone();
    let selected = selected.clone();
    let bulk_selected =
      bulk_selected.clone();
    Callback::from(
      move |project: Option<String>| {
        active_project.set(project);
        selected.set(None);
        bulk_selected
          .set(BTreeSet::new());
      }
    )
  };

  let on_choose_tag = {
    let active_tag = active_tag.clone();
    let selected = selected.clone();
    let bulk_selected =
      bulk_selected.clone();
    Callback::from(
      move |tag: Option<String>| {
        active_tag.set(tag);
        selected.set(None);
        bulk_selected
          .set(BTreeSet::new());
      }
    )
  };

  let on_all_completion_change = {
    let all_filter_completion =
      all_filter_completion.clone();
    Callback::from(
      move |e: web_sys::Event| {
        if let Some(input) =
          e.target_dyn_into::<
            web_sys::HtmlSelectElement
          >()
        {
          all_filter_completion
            .set(input.value());
        } else {
          tracing::warn!(
            "all completion filter event \
             had non-select target"
          );
        }
      }
    )
  };

  let on_all_project_change = {
    let all_filter_project =
      all_filter_project.clone();
    Callback::from(
      move |e: web_sys::Event| {
        if let Some(input) =
          e.target_dyn_into::<
            web_sys::HtmlSelectElement
          >()
        {
          let value = input.value();
          if value.is_empty() {
            all_filter_project
              .set(None);
          } else {
            all_filter_project
              .set(Some(value));
          }
        } else {
          tracing::warn!(
            "all project filter event had \
             non-select target"
          );
        }
      }
    )
  };

  let on_all_tag_change = {
    let all_filter_tag =
      all_filter_tag.clone();
    Callback::from(
      move |e: web_sys::Event| {
        if let Some(input) =
          e.target_dyn_into::<
            web_sys::HtmlSelectElement
          >()
        {
          let value = input.value();
          if value.is_empty() {
            all_filter_tag.set(None);
          } else {
            all_filter_tag.set(Some(value));
          }
        } else {
          tracing::warn!(
            "all tag filter event had \
             non-select target"
          );
        }
      }
    )
  };

  let on_all_filters_clear = {
    let all_filter_project =
      all_filter_project.clone();
    let all_filter_tag =
      all_filter_tag.clone();
    let all_filter_completion =
      all_filter_completion.clone();
    let all_filter_priority =
      all_filter_priority.clone();
    let all_filter_due =
      all_filter_due.clone();
    Callback::from(move |_| {
      all_filter_project.set(None);
      all_filter_tag.set(None);
      all_filter_completion
        .set("all".to_string());
      all_filter_priority
        .set("all".to_string());
      all_filter_due
        .set("all".to_string());
    })
  };

  let on_all_priority_change = {
    let all_filter_priority =
      all_filter_priority.clone();
    Callback::from(
      move |e: web_sys::Event| {
        if let Some(input) =
          e.target_dyn_into::<
            web_sys::HtmlSelectElement
          >()
        {
          all_filter_priority
            .set(input.value());
        } else {
          tracing::warn!(
            "all priority filter event \
             had non-select target"
          );
        }
      }
    )
  };

  let on_all_due_change = {
    let all_filter_due =
      all_filter_due.clone();
    Callback::from(
      move |e: web_sys::Event| {
        if let Some(input) =
          e.target_dyn_into::<
            web_sys::HtmlSelectElement
          >()
        {
          all_filter_due.set(input.value());
        } else {
          tracing::warn!(
            "all due filter event had \
             non-select target"
          );
        }
      }
    )
  };

  let on_calendar_set_view = {
    let calendar_view =
      calendar_view.clone();
    Callback::from(
      move |view: CalendarViewMode| {
        tracing::info!(
          view = %view.as_key(),
          "calendar view changed"
        );
        calendar_view.set(view);
      }
    )
  };

  let on_calendar_prev = {
    let calendar_focus_date =
      calendar_focus_date.clone();
    let calendar_view =
      calendar_view.clone();
    let week_start =
      calendar_week_start;
    Callback::from(move |_| {
      let next = shift_calendar_focus(
        *calendar_focus_date,
        *calendar_view,
        -1,
        week_start
      );
      tracing::debug!(
        from = %calendar_focus_date.format("%Y-%m-%d"),
        to = %next.format("%Y-%m-%d"),
        view = %(*calendar_view).as_key(),
        "calendar moved backward"
      );
      calendar_focus_date.set(next);
    })
  };

  let on_calendar_next = {
    let calendar_focus_date =
      calendar_focus_date.clone();
    let calendar_view =
      calendar_view.clone();
    let week_start =
      calendar_week_start;
    Callback::from(move |_| {
      let next = shift_calendar_focus(
        *calendar_focus_date,
        *calendar_view,
        1,
        week_start
      );
      tracing::debug!(
        from = %calendar_focus_date.format("%Y-%m-%d"),
        to = %next.format("%Y-%m-%d"),
        view = %(*calendar_view).as_key(),
        "calendar moved forward"
      );
      calendar_focus_date.set(next);
    })
  };

  let on_calendar_today = {
    let calendar_focus_date =
      calendar_focus_date.clone();
    let calendar_timezone =
      calendar_timezone;
    Callback::from(move |_| {
      let today = today_in_timezone(
        calendar_timezone
      );
      tracing::info!(
        today = %today.format("%Y-%m-%d"),
        timezone = %calendar_timezone,
        "calendar focus reset to today"
      );
      calendar_focus_date.set(today);
    })
  };

  let on_calendar_navigate = {
    let calendar_focus_date =
      calendar_focus_date.clone();
    let calendar_view =
      calendar_view.clone();
    Callback::from(
      move |(day, view): (
        NaiveDate,
        CalendarViewMode
      )| {
        calendar_focus_date.set(day);
        calendar_view.set(view);
      }
    )
  };

  let on_open_add_external_calendar = {
    let external_calendar_modal =
      external_calendar_modal.clone();
    Callback::from(move |_| {
      external_calendar_modal.set(Some(
          ExternalCalendarModalState {
            mode:
              ExternalCalendarModalMode::Add,
            source:
              new_external_calendar_source(),
            error: None,
          },
        ));
    })
  };

  let on_open_edit_external_calendar = {
    let external_calendar_modal =
      external_calendar_modal.clone();
    Callback::from(
        move |source: ExternalCalendarSource| {
          external_calendar_modal.set(Some(
            ExternalCalendarModalState {
              mode:
                ExternalCalendarModalMode::Edit,
              source,
              error: None,
            },
          ));
        },
      )
  };

  let on_close_external_calendar_modal = {
    let external_calendar_modal =
      external_calendar_modal.clone();
    Callback::from(move |_| {
      external_calendar_modal.set(None);
    })
  };

  let on_submit_external_calendar = {
    let external_calendars =
      external_calendars.clone();
    let external_calendar_modal =
      external_calendar_modal.clone();
    Callback::from(
        move |modal_state: ExternalCalendarModalState| {
          let mut source =
            modal_state.source.clone();

          if source
            .name
            .trim()
            .is_empty()
          {
            let mut next = modal_state;
            next.error = Some(
              "Calendar name is required."
                .to_string(),
            );
            external_calendar_modal
              .set(Some(next));
            return;
          }

          if source
            .location
            .trim()
            .is_empty()
          {
            let mut next = modal_state;
            next.error = Some(
              "Calendar URL is required."
                .to_string(),
            );
            external_calendar_modal
              .set(Some(next));
            return;
          }

          source.name = source
            .name
            .trim()
            .to_string();
          source.location = source
            .location
            .trim()
            .to_string();
          if source
            .refresh_minutes
            == 0
          {
            source.refresh_minutes = 30;
          }
          if source
            .color
            .trim()
            .is_empty()
          {
            source.color =
              "#d64545".to_string();
          }

          let mut next_sources =
            (*external_calendars).clone();
          match modal_state.mode {
            | ExternalCalendarModalMode::Add => {
              next_sources.push(source);
            }
            | ExternalCalendarModalMode::Edit => {
              if let Some(existing) =
                next_sources.iter_mut().find(
                  |existing| {
                    existing.id
                      == source.id
                  },
                )
              {
                *existing = source;
              }
            }
          }

          external_calendars
            .set(next_sources);
          external_calendar_modal
            .set(None);
        },
      )
  };

  let on_delete_external_calendar = {
    let external_calendars =
      external_calendars.clone();
    Callback::from(
      move |calendar_id: String| {
        let confirmed =
          web_sys::window()
            .and_then(|window| {
              window
                .confirm_with_message(
                  "Delete this \
                   external calendar \
                   source?"
                )
                .ok()
            })
            .unwrap_or(false);
        if !confirmed {
          return;
        }

        let mut next_sources =
          (*external_calendars).clone();
        next_sources.retain(|source| {
          source.id != calendar_id
        });
        external_calendars
          .set(next_sources);
      }
    )
  };

  let on_sync_external_calendar = {
    let external_calendars =
      external_calendars.clone();
    let external_calendar_busy =
      external_calendar_busy.clone();
    let external_calendar_last_sync =
      external_calendar_last_sync
        .clone();
    let refresh_tick =
      refresh_tick.clone();
    Callback::from(
      move |calendar_id: String| {
        if *external_calendar_busy {
          return;
        }
        let Some(source) =
          external_calendars
            .iter()
            .find(|source| {
              source.id == calendar_id
            })
            .cloned()
        else {
          return;
        };

        external_calendar_busy
          .set(true);
        let external_calendar_busy =
          external_calendar_busy
            .clone();
        let external_calendar_last_sync =
          external_calendar_last_sync
            .clone();
        let refresh_tick =
          refresh_tick.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match invoke_tauri::<
              ExternalCalendarSyncResult,
              _
            >("external_calendar_sync", &source)
            .await
            {
              | Ok(result) => {
                external_calendar_last_sync
                  .set(Some(format!(
                    "Synced {}: +{} / ~{} / -{}",
                    source.name,
                    result.created,
                    result.updated,
                    result.deleted
                  )));
                refresh_tick.set(
                  (*refresh_tick)
                    .saturating_add(1),
                );
              }
              | Err(err) => {
                external_calendar_last_sync
                  .set(Some(format!(
                    "Sync failed for {}: {}",
                    source.name, err
                  )));
              }
            }
            external_calendar_busy
              .set(false);
          });
      }
    )
  };

  let on_sync_all_external_calendars = {
    let external_calendars =
      external_calendars.clone();
    let external_calendar_busy =
      external_calendar_busy.clone();
    let external_calendar_last_sync =
      external_calendar_last_sync
        .clone();
    let refresh_tick =
      refresh_tick.clone();
    Callback::from(move |_| {
      if *external_calendar_busy {
        return;
      }

      let sources: Vec<
        ExternalCalendarSource
      > = external_calendars
        .iter()
        .filter(|source| source.enabled)
        .cloned()
        .collect();
      if sources.is_empty() {
        external_calendar_last_sync
          .set(Some(
            "No enabled calendars to \
             sync."
              .to_string()
          ));
        return;
      }

      external_calendar_busy.set(true);
      let external_calendar_busy =
        external_calendar_busy.clone();
      let external_calendar_last_sync =
        external_calendar_last_sync
          .clone();
      let refresh_tick =
        refresh_tick.clone();
      wasm_bindgen_futures::spawn_local(
        async move {
          let mut lines = Vec::new();
          for source in sources {
            match invoke_tauri::<
              ExternalCalendarSyncResult,
              _
            >("external_calendar_sync", &source)
            .await
            {
              | Ok(result) => {
                lines.push(format!(
                  "{}: +{} / ~{} / -{}",
                  source.name,
                  result.created,
                  result.updated,
                  result.deleted
                ));
              }
              | Err(err) => {
                lines.push(format!(
                  "{}: failed ({})",
                  source.name, err
                ));
              }
            }
          }

          external_calendar_last_sync
            .set(Some(
              lines.join(" | ")
            ));
          refresh_tick.set(
            (*refresh_tick)
              .saturating_add(1)
          );
          external_calendar_busy
            .set(false);
        }
      );
    })
  };

  let on_select_kanban_board = {
    let active_kanban_board =
      active_kanban_board.clone();
    let selected = selected.clone();
    Callback::from(
      move |board_id: String| {
        tracing::info!(
          board_id = %board_id,
          "selected kanban board"
        );
        active_kanban_board
          .set(Some(board_id));
        selected.set(None);
      }
    )
  };

  let on_create_kanban_board = {
    let kanban_create_open =
      kanban_create_open.clone();
    let kanban_create_input =
      kanban_create_input.clone();
    Callback::from(move |_| {
      kanban_create_input
        .set(String::new());
      kanban_create_open.set(true);
    })
  };

  let on_close_create_kanban_board = {
    let kanban_create_open =
      kanban_create_open.clone();
    Callback::from(move |_| {
      kanban_create_open.set(false);
    })
  };

  let on_create_kanban_input = {
    let kanban_create_input =
      kanban_create_input.clone();
    Callback::from(
      move |e: web_sys::InputEvent| {
        let input: web_sys::HtmlInputElement =
          e.target_unchecked_into();
        kanban_create_input
          .set(input.value());
      }
    )
  };

  let on_submit_create_kanban_board = {
    let kanban_boards =
      kanban_boards.clone();
    let active_kanban_board =
      active_kanban_board.clone();
    let kanban_create_open =
      kanban_create_open.clone();
    let kanban_create_input =
      kanban_create_input.clone();
    Callback::from(move |_| {
      let name = (*kanban_create_input)
        .trim()
        .to_string();

      if name.is_empty() {
        tracing::warn!(
          "ignored empty kanban board \
           name"
        );
        return;
      }

      let mut next =
        (*kanban_boards).clone();
      let unique_name =
        make_unique_board_name(
          &next, &name
        );
      let board_id =
        Uuid::new_v4().to_string();
      tracing::info!(
        board_id = %board_id,
        name = %unique_name,
        "creating kanban board"
      );
      let color =
        next_board_color(&next);
      next.push(KanbanBoardDef {
        id: board_id.clone(),
        name: unique_name,
        color
      });

      kanban_boards.set(next);
      active_kanban_board
        .set(Some(board_id));
      kanban_create_open.set(false);
    })
  };

  let on_toggle_kanban_card_density = {
    let kanban_compact_cards =
      kanban_compact_cards.clone();
    Callback::from(move |_| {
      kanban_compact_cards
        .set(!*kanban_compact_cards);
    })
  };

  let on_open_rename_kanban_board = {
    let kanban_boards =
      kanban_boards.clone();
    let active_kanban_board =
      active_kanban_board.clone();
    let kanban_rename_input =
      kanban_rename_input.clone();
    let kanban_rename_open =
      kanban_rename_open.clone();
    Callback::from(move |_| {
      let Some(board_id) =
        (*active_kanban_board).clone()
      else {
        tracing::warn!(
          "rename board clicked with \
           no active board"
        );
        return;
      };

      let Some(current) =
        (*kanban_boards)
          .iter()
          .find(|board| {
            board.id == board_id
          })
          .cloned()
      else {
        tracing::warn!(
          %board_id,
          "active board not found \
           for rename modal"
        );
        return;
      };

      kanban_rename_input
        .set(current.name);
      kanban_rename_open.set(true);
    })
  };

  let on_close_rename_kanban_board = {
    let kanban_rename_open =
      kanban_rename_open.clone();
    Callback::from(move |_| {
      kanban_rename_open.set(false);
    })
  };

  let on_rename_kanban_input = {
    let kanban_rename_input =
      kanban_rename_input.clone();
    Callback::from(
      move |e: web_sys::InputEvent| {
        let input: web_sys::HtmlInputElement =
          e.target_unchecked_into();
        kanban_rename_input
          .set(input.value());
      }
    )
  };

  let on_submit_rename_kanban_board = {
    let kanban_boards =
      kanban_boards.clone();
    let active_kanban_board =
      active_kanban_board.clone();
    let kanban_rename_open =
      kanban_rename_open.clone();
    let kanban_rename_input =
      kanban_rename_input.clone();
    Callback::from(move |_| {
      let Some(board_id) =
        (*active_kanban_board).clone()
      else {
        tracing::warn!(
          "rename board clicked with \
           no active board"
        );
        return;
      };

      let name = (*kanban_rename_input)
        .trim()
        .to_string();

      if name.is_empty() {
        tracing::warn!(
          %board_id,
          "ignored empty rename \
           request"
        );
        return;
      }

      let mut next =
        (*kanban_boards).clone();
      let unique_name =
        make_unique_board_name_except(
          &next, &name, &board_id
        );
      for board in &mut next {
        if board.id == board_id {
          board.name =
            unique_name.clone();
        }
      }

      tracing::info!(
        %board_id,
        name = %unique_name,
        "renamed kanban board"
      );
      kanban_boards.set(next);
      kanban_rename_open.set(false);
    })
  };

  let on_delete_kanban_board = {
    let kanban_boards =
      kanban_boards.clone();
    let active_kanban_board =
      active_kanban_board.clone();
    let facet_tasks =
      facet_tasks.clone();
    let refresh_tick =
      refresh_tick.clone();
    Callback::from(move |_| {
      let Some(board_id) =
        (*active_kanban_board).clone()
      else {
        tracing::warn!(
          "delete board clicked with \
           no active board"
        );
        return;
      };

      let Some(board) =
        (*kanban_boards)
          .iter()
          .find(|entry| {
            entry.id == board_id
          })
          .cloned()
      else {
        tracing::warn!(
          %board_id,
          "active board not found \
           for deletion"
        );
        return;
      };

      let confirmed = web_sys::window()
        .and_then(|window| {
          window
            .confirm_with_message(
              &format!(
                "Delete board \
                 '{}'?\nThis removes \
                 board assignment \
                 from pending tasks \
                 using this board.",
                board.name
              )
            )
            .ok()
        })
        .unwrap_or(false);

      if !confirmed {
        tracing::info!(
          %board_id,
          "board deletion canceled"
        );
        return;
      }

      let mut next_boards =
        (*kanban_boards).clone();
      next_boards.retain(|entry| {
        entry.id != board_id
      });

      let next_active = next_boards
        .first()
        .map(|entry| entry.id.clone());
      tracing::warn!(
        %board_id,
        next_active = ?next_active,
        "deleted kanban board"
      );
      kanban_boards.set(next_boards);
      active_kanban_board
        .set(next_active);

      let tasks_to_clean: Vec<TaskDto> =
        (*facet_tasks)
          .iter()
          .filter(|task| {
            matches!(
              task.status,
              TaskStatus::Pending
                | TaskStatus::Waiting
            ) && task_has_tag_value(
              &task.tags,
              BOARD_TAG_KEY,
              &board_id
            )
          })
          .cloned()
          .collect();

      let refresh_tick =
        refresh_tick.clone();
      wasm_bindgen_futures::spawn_local(
        async move {
          for task in tasks_to_clean {
            let mut next_tags =
              task.tags.clone();
            remove_board_tag_for_id(
              &mut next_tags,
              &board_id
            );

            let update =
              TaskUpdateArgs {
                uuid:  task.uuid,
                patch: TaskPatch {
                  tags: Some(next_tags),
                  ..TaskPatch::default()
                }
              };

            if let Err(err) = invoke_tauri::<TaskDto, _>("task_update", &update).await {
                        tracing::error!(error = %err, task = %task.uuid, board_id = %board_id, "failed clearing deleted board tag");
                    }
          }

          refresh_tick.set(
            (*refresh_tick)
              .saturating_add(1)
          );
        }
      );
    })
  };

  let on_add_click = {
    let modal_state =
      modal_state.clone();
    let modal_busy = modal_busy.clone();
    let modal_submit_seq =
      modal_submit_seq.clone();
    let tag_schema = tag_schema.clone();
    let active_tab = active_tab.clone();
    let active_kanban_board =
      active_kanban_board.clone();
    Callback::from(move |_| {
      let (picker_key, picker_value) =
        tag_schema.default_picker();
      let draft_board_id =
        if *active_tab == "kanban" {
          (*active_kanban_board).clone()
        } else {
          None
        };
      let lock_board_selection =
        *active_tab == "kanban"
          && draft_board_id.is_some();
      modal_busy.set(false);
      modal_submit_seq.set(
        (*modal_submit_seq)
          .wrapping_add(1)
      );
      modal_state.set(Some(
        ModalState {
          mode: ModalMode::Add,
          draft_title: String::new(),
          draft_desc: String::new(),
          draft_project: String::new(),
          draft_board_id,
          lock_board_selection,
          draft_custom_tag: String::new(
          ),
          draft_tags: vec![],
          picker_key,
          picker_value,
          draft_due: String::new(),
          recurrence_pattern: "none"
            .to_string(),
          recurrence_time: String::new(
          ),
          recurrence_days: vec![],
          recurrence_months: vec![],
          recurrence_month_day:
            String::new(),
          error: None
        }
      ));
      ui_debug(
        "action.add_modal.open",
        "clicked Add Task"
      );
    })
  };

  let on_toggle_theme = {
    let theme = theme.clone();
    Callback::from(move |_| {
      let next = (*theme).next();
      save_theme_mode(next);
      theme.set(next);
    })
  };

  let on_window_minimize =
    Callback::from(move |_| {
      wasm_bindgen_futures::spawn_local(
        async move {
          if let Err(err) =
            invoke_tauri::<(), _>(
              "window_minimize",
              &()
            )
            .await
          {
            tracing::error!(
              error = %err,
              "window minimize failed"
            );
          }
        }
      );
    });

  let on_window_toggle_maximize =
    Callback::from(move |_| {
      wasm_bindgen_futures::spawn_local(
        async move {
          if let Err(err) =
            invoke_tauri::<(), _>(
              "window_toggle_maximize",
              &()
            )
            .await
          {
            tracing::error!(
              error = %err,
              "window toggle maximize failed"
            );
          }
        }
      );
    });

  let on_window_close =
    Callback::from(move |_| {
      wasm_bindgen_futures::spawn_local(
        async move {
          if let Err(err) =
            invoke_tauri::<(), _>(
              "window_close",
              &()
            )
            .await
          {
            tracing::error!(
              error = %err,
              "window close failed"
            );
          }
        }
      );
    });

  let on_done = {
    let refresh_tick =
      refresh_tick.clone();
    let selected = selected.clone();
    let bulk_selected =
      bulk_selected.clone();
    Callback::from(move |uuid: Uuid| {
      let refresh_tick =
        refresh_tick.clone();
      let selected = selected.clone();
      let bulk_selected =
        bulk_selected.clone();

      wasm_bindgen_futures::spawn_local(
        async move {
          let arg = TaskIdArg {
            uuid
          };
          match invoke_tauri::<TaskDto, _>("task_done", &arg).await {
                    Ok(_) => {
                        selected.set(None);
                        bulk_selected.set(BTreeSet::new());
                        refresh_tick.set((*refresh_tick).saturating_add(1));
                    }
                    Err(err) => tracing::error!(error = %err, "task_done failed"),
                }
        }
      );
    })
  };

  let on_delete = {
    let refresh_tick =
      refresh_tick.clone();
    let selected = selected.clone();
    let bulk_selected =
      bulk_selected.clone();
    Callback::from(move |uuid: Uuid| {
      let refresh_tick =
        refresh_tick.clone();
      let selected = selected.clone();
      let bulk_selected =
        bulk_selected.clone();

      wasm_bindgen_futures::spawn_local(
        async move {
          let arg = TaskIdArg {
            uuid
          };
          match invoke_tauri::<(), _>(
            "task_delete",
            &arg
          )
          .await
          {
            | Ok(()) => {
              selected.set(None);
              bulk_selected
                .set(BTreeSet::new());
              refresh_tick.set(
                (*refresh_tick)
                  .saturating_add(1)
              );
            }
            | Err(err) => {
              tracing::error!(error = %err, "task_delete failed")
            }
          }
        }
      );
    })
  };

  let on_kanban_move = {
    let tasks = tasks.clone();
    let kanban_columns =
      kanban_columns.clone();
    let default_kanban_lane =
      default_kanban_lane.clone();
    let refresh_tick =
      refresh_tick.clone();
    let dragging_kanban_task =
      dragging_kanban_task.clone();
    let drag_over_kanban_lane =
      drag_over_kanban_lane.clone();
    Callback::from(
      move |(uuid, lane): (
        Uuid,
        String
      )| {
        dragging_kanban_task.set(None);
        drag_over_kanban_lane.set(None);

        let target_lane =
          if kanban_columns.iter().any(
            |column| column == &lane
          ) {
            lane
          } else {
            tracing::warn!(
              lane = %lane,
              fallback = %default_kanban_lane,
              "unknown kanban lane during \
               move; falling back to \
               default lane"
            );
            default_kanban_lane.clone()
          };

        let Some(task) = (*tasks)
          .iter()
          .find(|task| {
            task.uuid == uuid
          })
          .cloned()
        else {
          tracing::warn!(
            %uuid,
            "kanban move ignored because \
             task is not in current \
             snapshot"
          );
          return;
        };

        if !matches!(
          task.status,
          TaskStatus::Pending
            | TaskStatus::Waiting
        ) {
          tracing::warn!(
            %uuid,
            status = ?task.status,
            "kanban move ignored for \
             non-pending task"
          );
          return;
        }

        let mut next_tags =
          task.tags.clone();
        remove_tags_for_key(
          &mut next_tags,
          KANBAN_TAG_KEY
        );
        push_tag_unique(
          &mut next_tags,
          format!(
            "{KANBAN_TAG_KEY}:\
             {target_lane}"
          )
        );

        tracing::info!(
          %uuid,
          lane = %target_lane,
          tag_count = next_tags.len(),
          "moving task in kanban by \
           rewriting lane tag"
        );

        let update = TaskUpdateArgs {
          uuid,
          patch: TaskPatch {
            tags: Some(next_tags),
            ..TaskPatch::default()
          }
        };

        let refresh_tick =
          refresh_tick.clone();
        wasm_bindgen_futures::spawn_local(
          async move {
            match invoke_tauri::<TaskDto, _>(
              "task_update",
              &update
            )
            .await
            {
              | Ok(_) => {
                refresh_tick.set(
                  (*refresh_tick)
                    .saturating_add(1)
                );
              }
              | Err(err) => tracing::error!(error = %err, %uuid, "kanban move task_update failed")
            }
          }
        );
      }
    )
  };

  let on_kanban_drag_start = {
    let dragging_kanban_task =
      dragging_kanban_task.clone();
    Callback::from(move |uuid: Uuid| {
      tracing::debug!(
        %uuid,
        "kanban drag start"
      );
      dragging_kanban_task
        .set(Some(uuid));
    })
  };

  let on_kanban_drag_end = {
    let dragging_kanban_task =
      dragging_kanban_task.clone();
    let drag_over_kanban_lane =
      drag_over_kanban_lane.clone();
    Callback::from(move |_| {
      tracing::debug!(
        "kanban drag end"
      );
      dragging_kanban_task.set(None);
      drag_over_kanban_lane.set(None);
    })
  };

  let on_kanban_drag_over_lane = {
    let drag_over_kanban_lane =
      drag_over_kanban_lane.clone();
    Callback::from(
      move |lane: String| {
        if (*drag_over_kanban_lane)
          .as_deref()
          != Some(lane.as_str())
        {
          tracing::debug!(
            lane = %lane,
            "kanban drag over lane"
          );
          drag_over_kanban_lane
            .set(Some(lane));
        }
      }
    )
  };

  let on_bulk_done = {
    let bulk_selected =
      bulk_selected.clone();
    let refresh_tick =
      refresh_tick.clone();
    let selected = selected.clone();
    Callback::from(move |_| {
      let ids: Vec<Uuid> =
        (*bulk_selected)
          .iter()
          .copied()
          .collect();
      if ids.is_empty() {
        return;
      }

      let bulk_selected =
        bulk_selected.clone();
      let refresh_tick =
        refresh_tick.clone();
      let selected = selected.clone();

      wasm_bindgen_futures::spawn_local(
        async move {
          for uuid in ids {
            let arg = TaskIdArg {
              uuid
            };
            if let Err(err) = invoke_tauri::<TaskDto, _>("task_done", &arg).await {
                        tracing::error!(error = %err, %uuid, "bulk task_done failed");
                    }
          }

          selected.set(None);
          bulk_selected
            .set(BTreeSet::new());
          refresh_tick.set(
            (*refresh_tick)
              .saturating_add(1)
          );
        }
      );
    })
  };

  let on_bulk_delete = {
    let bulk_selected =
      bulk_selected.clone();
    let refresh_tick =
      refresh_tick.clone();
    let selected = selected.clone();
    Callback::from(move |_| {
      let ids: Vec<Uuid> =
        (*bulk_selected)
          .iter()
          .copied()
          .collect();
      if ids.is_empty() {
        return;
      }

      let bulk_selected =
        bulk_selected.clone();
      let refresh_tick =
        refresh_tick.clone();
      let selected = selected.clone();

      wasm_bindgen_futures::spawn_local(
        async move {
          for uuid in ids {
            let arg = TaskIdArg {
              uuid
            };
            if let Err(err) =
              invoke_tauri::<(), _>(
                "task_delete",
                &arg
              )
              .await
            {
              tracing::error!(error = %err, %uuid, "bulk task_delete failed");
            }
          }

          selected.set(None);
          bulk_selected
            .set(BTreeSet::new());
          refresh_tick.set(
            (*refresh_tick)
              .saturating_add(1)
          );
        }
      );
    })
  };

  let on_edit = {
    let modal_state =
      modal_state.clone();
    let modal_busy = modal_busy.clone();
    let modal_submit_seq =
      modal_submit_seq.clone();
    let tag_schema = tag_schema.clone();
    let kanban_boards =
      kanban_boards.clone();
    Callback::from(
      move |task: TaskDto| {
        let (picker_key, picker_value) =
          tag_schema.default_picker();
        let draft_board_id =
          board_id_from_task_tags(
            &kanban_boards,
            &task.tags
          );
        let (
          recurrence_pattern,
          recurrence_time,
          recurrence_days,
          recurrence_months,
          recurrence_month_day
        ) = recurrence_from_tags(
          &task.tags
        );
        let filtered_tags = task
          .tags
          .into_iter()
          .filter(|tag| {
            !tag.starts_with(&format!(
              "{BOARD_TAG_KEY}:"
            )) && !is_recurrence_tag(
              tag
            )
          })
          .collect();
        modal_busy.set(false);
        modal_submit_seq.set(
          (*modal_submit_seq)
            .wrapping_add(1)
        );
        modal_state.set(Some(
          ModalState {
            mode: ModalMode::Edit(
              task.uuid
            ),
            draft_title: task.title,
            draft_desc: task
              .description,
            draft_project: task
              .project
              .unwrap_or_default(),
            draft_board_id,
            lock_board_selection: false,
            draft_custom_tag:
              String::new(),
            draft_tags: filtered_tags,
            picker_key,
            picker_value,
            draft_due: task
              .due
              .unwrap_or_default(),
            recurrence_pattern,
            recurrence_time,
            recurrence_days,
            recurrence_months,
            recurrence_month_day,
            error: None
          }
        ));
      }
    )
  };

  let close_modal = {
    let modal_state =
      modal_state.clone();
    let modal_busy = modal_busy.clone();
    let modal_submit_seq =
      modal_submit_seq.clone();
    Callback::from(move |_| {
      modal_busy.set(false);
      modal_submit_seq.set(
        (*modal_submit_seq)
          .wrapping_add(1)
      );
      modal_state.set(None);
      ui_debug(
        "action.modal.cancel",
        "Cancel clicked, closing modal"
      );
    })
  };

  let on_modal_close_click = {
    let close_modal =
      close_modal.clone();
    Callback::from(move |_| {
      close_modal.emit(())
    })
  };

  let on_modal_submit = {
    let modal_state =
      modal_state.clone();
    let refresh_tick =
      refresh_tick.clone();
    let modal_busy = modal_busy.clone();
    let modal_submit_seq =
      modal_submit_seq.clone();
    let kanban_boards =
      kanban_boards.clone();
    let default_kanban_lane =
      default_kanban_lane.clone();
    Callback::from(
      move |state: ModalState| {
        if *modal_busy {
          ui_debug(
            "action.modal.submit.skip",
            "ignored duplicate while \
             busy"
          );
          return;
        }
        modal_busy.set(true);
        let submit_seq =
          (*modal_submit_seq)
            .wrapping_add(1);
        modal_submit_seq
          .set(submit_seq);
        ui_debug(
          "action.modal.submit",
          &format!(
            "mode={}, title_len={}, \
             desc_len={}",
            match state.mode {
              | ModalMode::Add => "add",
              | ModalMode::Edit(_) =>
                "edit",
            },
            state.draft_title.len(),
            state.draft_desc.len()
          )
        );
        let modal_state =
          modal_state.clone();
        let refresh_tick =
          refresh_tick.clone();
        let modal_busy =
          modal_busy.clone();
        let modal_submit_seq =
          modal_submit_seq.clone();
        let kanban_boards =
          kanban_boards.clone();
        let default_kanban_lane =
          default_kanban_lane.clone();

        {
          let modal_state =
            modal_state.clone();
          let modal_busy =
            modal_busy.clone();
          let modal_submit_seq =
            modal_submit_seq.clone();
          let timeout_state =
            state.clone();
          wasm_bindgen_futures::spawn_local(async move {
                    TimeoutFuture::new(8_000).await;
                    if *modal_busy && *modal_submit_seq == submit_seq {
                        let mut next = timeout_state;
                        next.error = Some(
                            "Save timed out waiting for backend response. Check Tauri IPC/capability configuration."
                                .to_string(),
                        );
                        modal_state.set(Some(next));
                        modal_busy.set(false);
                        ui_debug("action.modal.submit.timeout", "save invoke timed out");
                    }
                });
        }

        wasm_bindgen_futures::spawn_local(async move {
                if state.draft_title.trim().is_empty() {
                    let mut next = state.clone();
                    next.error = Some("Title is required.".to_string());
                    modal_state.set(Some(next));
                    modal_busy.set(false);
                    return;
                }

                let board_tag = state
                    .draft_board_id
                    .clone()
                    .and_then(|board_id| {
                        kanban_boards
                            .iter()
                            .find(|board| board.id == board_id)
                            .map(|board| format!("{BOARD_TAG_KEY}:{}", board.id))
                    });

                match state.mode {
                    ModalMode::Add => {
                        let create = TaskCreate {
                            title: state.draft_title.trim().to_string(),
                            description: state.draft_desc.trim().to_string(),
                            project: optional_text(&state.draft_project),
                            tags: collect_tags_for_submit(
                                &state,
                                board_tag.clone(),
                                true,
                                &default_kanban_lane,
                            ),
                            priority: None,
                            due: optional_text(&state.draft_due),
                            wait: None,
                            scheduled: None,
                        };

                        ui_debug("invoke.task_add.begin", "calling tauri command task_add");
                        if let Err(err) = invoke_tauri::<TaskDto, _>("task_add", &create).await {
                            tracing::error!(error = %err, "task_add failed");
                            ui_debug("invoke.task_add.error", &err);
                            let mut next = state.clone();
                            next.error = Some(format!("Save failed: {err}"));
                            modal_state.set(Some(next));
                            modal_busy.set(false);
                            modal_submit_seq.set(submit_seq.wrapping_add(1));
                            return;
                        }
                        ui_debug("invoke.task_add.ok", "task_add succeeded");
                    }
                    ModalMode::Edit(uuid) => {
                        let update = TaskUpdateArgs {
                            uuid,
                            patch: TaskPatch {
                                title: Some(state.draft_title.trim().to_string()),
                                description: Some(state.draft_desc.trim().to_string()),
                                project: Some(optional_text(&state.draft_project)),
                                tags: Some(collect_tags_for_submit(
                                    &state,
                                    board_tag,
                                    false,
                                    &default_kanban_lane,
                                )),
                                due: Some(optional_text(&state.draft_due)),
                                ..TaskPatch::default()
                            },
                        };

                        ui_debug(
                            "invoke.task_update.begin",
                            &format!("calling tauri command task_update uuid={uuid}"),
                        );
                        if let Err(err) = invoke_tauri::<TaskDto, _>("task_update", &update).await {
                            tracing::error!(error = %err, "task_update failed");
                            ui_debug("invoke.task_update.error", &err);
                            let mut next = state.clone();
                            next.error = Some(format!("Save failed: {err}"));
                            modal_state.set(Some(next));
                            modal_busy.set(false);
                            modal_submit_seq.set(submit_seq.wrapping_add(1));
                            return;
                        }
                        ui_debug("invoke.task_update.ok", "task_update succeeded");
                    }
                }

                ui_debug("action.modal.close", "save complete, closing modal");
                modal_state.set(None);
                refresh_tick.set((*refresh_tick).saturating_add(1));
                modal_busy.set(false);
                modal_submit_seq.set(submit_seq.wrapping_add(1));
            });
      }
    )
  };

  let bulk_count =
    (*bulk_selected).len();
  let is_any_modal_open = (*modal_state)
    .is_some()
    || *kanban_create_open
    || *kanban_rename_open
    || (*external_calendar_modal)
      .is_some();
  let active_kanban_board_name =
    (*active_kanban_board)
      .as_ref()
      .and_then(|board_id| {
        kanban_boards
          .iter()
          .find(|board| {
            &board.id == board_id
          })
          .map(|board| {
            board.name.clone()
          })
      });

  html! {
      <div class={classes!("app", (*theme).as_class(), is_any_modal_open.then_some("modal-open"))}>
          <div class="window-chrome" data-tauri-drag-region="true">
              <div class="window-brand">
                  <img class="window-mascot" src="assets/icons/mascot-square.png" alt="Rivet mascot" />
                  <span>{ "Rivet" }</span>
              </div>
              <div class="window-controls" data-tauri-drag-region="false">
                  <button class="window-btn" type="button" onclick={on_window_minimize} title="Minimize">{ "_" }</button>
                  <button class="window-btn" type="button" onclick={on_window_toggle_maximize} title="Maximize/Restore">{ "[ ]" }</button>
                  <button class="window-btn danger" type="button" onclick={on_window_close} title="Close">{ "X" }</button>
              </div>
          </div>
          <div class="workspace-tabs">
              <div class="workspace-tab-list">
                  <button
                      class={if *active_tab == "tasks" { "workspace-tab active" } else { "workspace-tab" }}
                      onclick={on_select_tasks_tab}
                  >
                      { "Tasks" }
                  </button>
                  <button
                      class={if *active_tab == "kanban" { "workspace-tab active" } else { "workspace-tab" }}
                      onclick={on_select_kanban_tab}
                  >
                      { "Kanban" }
                  </button>
                  <button
                      class={if *active_tab == "calendar" { "workspace-tab active" } else { "workspace-tab" }}
                      onclick={on_select_calendar_tab}
                  >
                      { "Calendar" }
                  </button>
              </div>
              <div class="workspace-actions">
                  {
                      if bulk_count > 0 {
                          html! {
                              <>
                                  <button class="btn ok" onclick={on_bulk_done.clone()}>{ format!("Done {bulk_count}") }</button>
                                  <button class="btn danger" onclick={on_bulk_delete.clone()}>{ format!("Delete {bulk_count}") }</button>
                              </>
                          }
                      } else {
                          html! {}
                      }
                  }
                  <button class="btn" onclick={on_add_click}>{ "Add Task" }</button>
                  <button class="btn" onclick={on_toggle_theme}>{ (*theme).toggle_label() }</button>
              </div>
          </div>

          <div class="main">
              {
                  if *active_tab == "calendar" {
                      html! {
                          <>
                              <div class="panel calendar-sidebar">
                                  <div class="header">{ "Calendar Views" }</div>
                                  <div class="details">
                                      <div class="calendar-view-switch">
                                          {
                                              for CalendarViewMode::all().iter().copied().map(|view| {
                                                  let on_calendar_set_view = on_calendar_set_view.clone();
                                                  let is_active = *calendar_view == view;
                                                  html! {
                                                      <button
                                                          class={classes!("calendar-view-btn", is_active.then_some("active"))}
                                                          onclick={Callback::from(move |_| on_calendar_set_view.emit(view))}
                                                      >
                                                          { view.label() }
                                                      </button>
                                                  }
                                              })
                                          }
                                      </div>
                                      <div class="actions calendar-nav-actions">
                                          <button class="btn" onclick={on_calendar_prev.clone()}>{ "Prev" }</button>
                                          <button class="btn" onclick={on_calendar_today.clone()}>{ "Today" }</button>
                                          <button class="btn" onclick={on_calendar_next.clone()}>{ "Next" }</button>
                                      </div>
                                      <div class="kv">
                                          <strong>{ "timezone" }</strong>
                                          <div>{ calendar_timezone.to_string() }</div>
                                      </div>
                                      <div class="kv">
                                          <strong>{ "focus date" }</strong>
                                          <div>{ calendar_focus_date.format("%Y-%m-%d").to_string() }</div>
                                      </div>
                                      <div class="kv">
                                          <strong>{ "due tasks" }</strong>
                                          <div>{ calendar_due_tasks.len() }</div>
                                      </div>
                                      <div class="calendar-dot-legend">
                                          <span class="calendar-marker triangle" style="--marker-color:var(--accent);"></span>
                                          <span>{ "Kanban board task" }</span>
                                      </div>
                                      <div class="calendar-dot-legend">
                                          <span class="calendar-marker circle" style="--marker-color:#d64545;"></span>
                                          <span>{ "External calendar task" }</span>
                                      </div>
                                      <div class="calendar-dot-legend">
                                          <span class="calendar-marker square" style="--marker-color:#7f8691;"></span>
                                          <span>{ "Unassigned task" }</span>
                                      </div>
                                      <div class="calendar-external-header">{ "External Calendars" }</div>
                                      <div class="actions">
                                          <button class="btn" onclick={on_open_add_external_calendar.clone()}>{ "Add Source" }</button>
                                          <button class="btn" onclick={on_sync_all_external_calendars.clone()} disabled={*external_calendar_busy}>
                                              { if *external_calendar_busy { "Syncing..." } else { "Sync Enabled" } }
                                          </button>
                                      </div>
                                      {
                                          if let Some(last_sync) = (*external_calendar_last_sync).clone() {
                                              html! { <div class="field-help">{ last_sync }</div> }
                                          } else {
                                              html! {}
                                          }
                                      }
                                      <div class="calendar-source-list">
                                          {
                                              if external_calendars.is_empty() {
                                                  html! { <div class="calendar-empty">{ "No external calendar sources configured." }</div> }
                                              } else {
                                                  html! {
                                                      <>
                                                          {
                                                              for external_calendars.iter().map(|source| {
                                                                  let source_id = source.id.clone();
                                                                  let source_id_for_sync = source_id.clone();
                                                                  let source_id_for_delete = source_id.clone();
                                                                  let source_for_edit = source.clone();
                                                                  let color_style = format!("background:{};", source.color);
                                                                  html! {
                                                                      <div class="calendar-source-item">
                                                                          <div class="calendar-source-top">
                                                                              <span class="calendar-source-color" style={color_style}></span>
                                                                              <span class="calendar-source-name">{ &source.name }</span>
                                                                              {
                                                                                  if source.enabled {
                                                                                      html! { <span class="badge">{ "enabled" }</span> }
                                                                                  } else {
                                                                                      html! { <span class="badge">{ "disabled" }</span> }
                                                                                  }
                                                                              }
                                                                          </div>
                                                                          <div class="task-subtitle">{ &source.location }</div>
                                                                          <div class="calendar-source-meta">
                                                                              <span class="badge">{ format!("refresh:{}m", source.refresh_minutes) }</span>
                                                                              {
                                                                                  if source.show_reminders {
                                                                                      html! { <span class="badge">{ "reminders:on" }</span> }
                                                                                  } else {
                                                                                      html! {}
                                                                                  }
                                                                              }
                                                                              {
                                                                                  if source.offline_support {
                                                                                      html! { <span class="badge">{ "offline:on" }</span> }
                                                                                  } else {
                                                                                      html! {}
                                                                                  }
                                                                              }
                                                                          </div>
                                                                          <div class="actions">
                                                                              <button class="btn" onclick={{
                                                                                  let on_sync_external_calendar = on_sync_external_calendar.clone();
                                                                                  Callback::from(move |_| on_sync_external_calendar.emit(source_id_for_sync.clone()))
                                                                              }} disabled={*external_calendar_busy}>
                                                                                  { "Sync" }
                                                                              </button>
                                                                              <button class="btn" onclick={{
                                                                                  let on_open_edit_external_calendar = on_open_edit_external_calendar.clone();
                                                                                  Callback::from(move |_| on_open_edit_external_calendar.emit(source_for_edit.clone()))
                                                                              }}>
                                                                                  { "Edit" }
                                                                              </button>
                                                                              <button class="btn danger" onclick={{
                                                                                  let on_delete_external_calendar = on_delete_external_calendar.clone();
                                                                                  Callback::from(move |_| on_delete_external_calendar.emit(source_id_for_delete.clone()))
                                                                              }}>
                                                                                  { "Delete" }
                                                                              </button>
                                                                          </div>
                                                                      </div>
                                                                  }
                                                              })
                                                          }
                                                      </>
                                                  }
                                              }
                                          }
                                      </div>
                                  </div>
                              </div>

                              <div class="panel calendar-panel">
                                  <div class="header">{ calendar_title.clone() }</div>
                                  <div class="details calendar-content">
                                      {
                                          render_calendar_view(
                                              *calendar_view,
                                              *calendar_focus_date,
                                              calendar_week_start,
                                              &calendar_due_tasks,
                                              &calendar_config,
                                              &tag_colors,
                                              on_calendar_navigate.clone(),
                                          )
                                      }
                                  </div>
                              </div>

                              <div class="right-stack">
                                  <div class="panel">
                                      <div class="header">{ "Calendar Stats" }</div>
                                      <div class="details">
                                          <div class="kv"><strong>{ "period tasks" }</strong><div>{ calendar_period_stats.total }</div></div>
                                          <div class="kv"><strong>{ "pending" }</strong><div>{ calendar_period_stats.pending }</div></div>
                                          <div class="kv"><strong>{ "waiting" }</strong><div>{ calendar_period_stats.waiting }</div></div>
                                          <div class="kv"><strong>{ "completed" }</strong><div>{ calendar_period_stats.completed }</div></div>
                                          <div class="kv"><strong>{ "deleted" }</strong><div>{ calendar_period_stats.deleted }</div></div>
                                      </div>
                                  </div>

                                  <div class="panel">
                                      <div class="header">{ "Tasks In Current Period" }</div>
                                      <div class="details calendar-task-list">
                                          {
                                              if calendar_period_tasks.is_empty() {
                                                  html! {
                                                      <div class="calendar-empty">
                                                          { "No tasks due in this calendar period." }
                                                      </div>
                                                  }
                                              } else {
                                                  html! {
                                                      <>
                                                          {
                                                              for calendar_period_tasks.iter().map(|entry| {
                                                                  let due_label = format_calendar_due_datetime(entry, calendar_timezone);
                                                                  html! {
                                                                      <div class="calendar-task-item">
                                                                          <div class="calendar-task-title">{ &entry.task.title }</div>
                                                                          <div class="task-subtitle">{ due_label }</div>
                                                                          <div class="calendar-task-meta">
                                                                              {
                                                                                  if let Some(project) = entry.task.project.clone() {
                                                                                      html! { <span class="badge">{ format!("project:{project}") }</span> }
                                                                                  } else {
                                                                                      html! {}
                                                                                  }
                                                                              }
                                                                              {
                                                                                  for entry.task.tags.iter().take(3).map(|tag| html! {
                                                                                      <span class="badge tag-badge" style={tag_badge_style(tag, &tag_colors)}>{ format!("#{tag}") }</span>
                                                                                  })
                                                                              }
                                                                          </div>
                                                                      </div>
                                                                  }
                                                              })
                                                          }
                                                      </>
                                                  }
                                              }
                                          }
                                      </div>
                                  </div>
                              </div>
                          </>
                      }
                  } else if *active_tab == "kanban" {
                      html! {
                          <>
                              <div class="panel board-sidebar">
                                  <div class="header">{ "Kanban Boards" }</div>
                                  <div class="details">
                                      <div class="actions">
                                          <button class="btn" onclick={on_create_kanban_board}>{ "New Board" }</button>
                                          <button class="btn" onclick={on_open_rename_kanban_board.clone()} disabled={(*active_kanban_board).is_none()}>{ "Rename" }</button>
                                          <button class="btn danger" onclick={on_delete_kanban_board.clone()} disabled={(*active_kanban_board).is_none()}>{ "Delete" }</button>
                                          <button class="btn" onclick={on_toggle_kanban_card_density.clone()}>
                                              { if *kanban_compact_cards { "Full Cards" } else { "Compact Cards" } }
                                          </button>
                                      </div>
                                      {
                                          if kanban_boards.is_empty() {
                                              html! { <div style="color:var(--muted);">{ "No boards yet. Create one to begin." }</div> }
                                          } else {
                                              html! {
                                                  <div class="board-list">
                                                      {
                                                          for kanban_boards.iter().map(|board| {
                                                              let board_id = board.id.clone();
                                                              let board_label = board.name.clone();
                                                              let board_color = board.color.clone();
                                                              let board_color_style = format!("background:{board_color};");
                                                              let is_active = (*active_kanban_board).as_deref() == Some(board_id.as_str());
                                                              let class = if is_active { "board-item active" } else { "board-item" };
                                                              html! {
                                                                  <div class={class} onclick={{
                                                                      let on_select_kanban_board = on_select_kanban_board.clone();
                                                                      Callback::from(move |_| on_select_kanban_board.emit(board_id.clone()))
                                                                  }}>
                                                                      <div class="board-item-line">
                                                                          <span class="board-color-dot" style={board_color_style}></span>
                                                                          <span>{ board_label }</span>
                                                                      </div>
                                                                  </div>
                                                              }
                                                          })
                                                      }
                                                  </div>
                                              }
                                          }
                                      }
                                  </div>
                              </div>

                              <KanbanBoard
                                  tasks={kanban_visible_tasks.clone()}
                                  columns={kanban_columns.clone()}
                                  board_name={active_kanban_board_name.clone()}
                                  tag_colors={tag_colors.clone()}
                                  compact_cards={*kanban_compact_cards}
                                  dragging_task={*dragging_kanban_task}
                                  drag_over_lane={(*drag_over_kanban_lane).clone()}
                                  on_move={on_kanban_move}
                                  on_drag_start={on_kanban_drag_start}
                                  on_drag_end={on_kanban_drag_end}
                                  on_drag_over_lane={on_kanban_drag_over_lane}
                                  on_edit={on_edit.clone()}
                                  on_done={on_done.clone()}
                                  on_delete={on_delete.clone()}
                              />

                              <div class="panel">
                                  <div class="header">{ "Kanban Filters" }</div>
                                  <div class="details">
                                      <div class="kv">
                                          <strong>{ "board" }</strong>
                                          <div>{ active_kanban_board_name.clone().unwrap_or_else(|| "None".to_string()) }</div>
                                      </div>
                                      <div class="kv">
                                          <strong>{ "cards shown" }</strong>
                                          <div>{ kanban_visible_tasks.len() }</div>
                                      </div>
                                      <div class="field">
                                          <label>{ "Completion" }</label>
                                          <select
                                              class="tag-select"
                                              value={(*all_filter_completion).clone()}
                                              onchange={on_all_completion_change}
                                          >
                                              <option value="all">{ "All" }</option>
                                              <option value="open">{ "Open (Pending + Waiting)" }</option>
                                              <option value="pending">{ "Pending" }</option>
                                              <option value="waiting">{ "Waiting" }</option>
                                              <option value="completed">{ "Completed" }</option>
                                              <option value="deleted">{ "Deleted" }</option>
                                          </select>
                                      </div>
                                      <div class="field">
                                          <label>{ "Project" }</label>
                                          <select
                                              class="tag-select"
                                              value={(*all_filter_project).clone().unwrap_or_default()}
                                              onchange={on_all_project_change}
                                          >
                                              <option value="">{ "All Projects" }</option>
                                              {
                                                  for project_facets.iter().map(|(project, count)| html! {
                                                      <option value={project.clone()}>{ format!("{project} ({count})") }</option>
                                                  })
                                              }
                                          </select>
                                      </div>
                                      <div class="field">
                                          <label>{ "Tag" }</label>
                                          <select
                                              class="tag-select"
                                              value={(*all_filter_tag).clone().unwrap_or_default()}
                                              onchange={on_all_tag_change}
                                          >
                                              <option value="">{ "All Tags" }</option>
                                              {
                                                  for tag_facets.iter().map(|(tag, count)| html! {
                                                      <option value={tag.clone()}>{ format!("{tag} ({count})") }</option>
                                                  })
                                              }
                                          </select>
                                      </div>
                                      <div class="field">
                                          <label>{ "Priority" }</label>
                                          <select
                                              class="tag-select"
                                              value={(*all_filter_priority).clone()}
                                              onchange={on_all_priority_change}
                                          >
                                              <option value="all">{ "All Priorities" }</option>
                                              <option value="low">{ "Low" }</option>
                                              <option value="medium">{ "Medium" }</option>
                                              <option value="high">{ "High" }</option>
                                              <option value="none">{ "None" }</option>
                                          </select>
                                      </div>
                                      <div class="field">
                                          <label>{ "Due" }</label>
                                          <select
                                              class="tag-select"
                                              value={(*all_filter_due).clone()}
                                              onchange={on_all_due_change}
                                          >
                                              <option value="all">{ "All" }</option>
                                              <option value="has_due">{ "Has Due Date" }</option>
                                              <option value="no_due">{ "No Due Date" }</option>
                                          </select>
                                      </div>
                                      <div class="actions">
                                          <button class="btn" onclick={on_all_filters_clear.clone()}>{ "Clear Filters" }</button>
                                      </div>
                                  </div>
                              </div>
                          </>
                      }
                  } else if *active_view == "settings" {
                      html! {
                          <>
                              <Sidebar active={(*active_view).clone()} on_nav={on_nav.clone()} />
                              <div class="panel list">
                                  <div class="header">{ "Settings" }</div>
                                  <div class="details">
                                      <div>{ "The desktop UI is a thin client over the core Rivet datastore." }</div>
                                      <div class="kv"><strong>{ "view" }</strong><div>{ "settings" }</div></div>
                                      <div class="kv"><strong>{ "status" }</strong><div>{ "core + tauri bridge active" }</div></div>
                                      <div class="kv"><strong>{ "workflow" }</strong><div>{ "Use context/report commands in CLI for advanced behavior." }</div></div>
                                  </div>
                              </div>
                              <div class="panel">
                                  <div class="header">{ "Current Data" }</div>
                                  <div class="details">
                                      <div class="kv"><strong>{ "tasks loaded" }</strong><div>{ tasks.len() }</div></div>
                                      <div class="kv"><strong>{ "selected" }</strong><div>{ bulk_count }</div></div>
                                  </div>
                              </div>
                          </>
                      }
                  } else {
                      html! {
                          <>
                              <Sidebar active={(*active_view).clone()} on_nav={on_nav.clone()} />
                              <TaskList
                                  tasks={task_visible_tasks.clone()}
                                  tag_colors={tag_colors.clone()}
                                  selected={*selected}
                                  selected_ids={(*bulk_selected).clone()}
                                  on_select={on_select}
                                  on_toggle_select={on_toggle_select}
                              />
                              {
                                  if *active_view == "projects" && selected_task.is_none() {
                                      html! {
                                          <FacetPanel
                                              title={"Projects".to_string()}
                                              selected={(*active_project).clone()}
                                              items={project_facets}
                                              on_select={on_choose_project}
                                          />
                                      }
                                  } else if *active_view == "all" {
                                      html! {
                                          <div class="right-stack">
                                              <div class="panel">
                                                  <div class="header">{ "Task Filters" }</div>
                                                  <div class="details">
                                                      <div class="field">
                                                          <label>{ "Search Tasks" }</label>
                                                          <input
                                                              value={(*search).clone()}
                                                              placeholder="Search tasks"
                                                              oninput={{
                                                                  let search = search.clone();
                                                                  Callback::from(move |e: web_sys::InputEvent| {
                                                                      let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                                      search.set(input.value());
                                                                  })
                                                              }}
                                                          />
                                                      </div>
                                                      <div class="field">
                                                          <label>{ "Completion" }</label>
                                                          <select
                                                              class="tag-select"
                                                              value={(*all_filter_completion).clone()}
                                                              onchange={on_all_completion_change}
                                                          >
                                                              <option value="all">{ "All" }</option>
                                                              <option value="open">{ "Open (Pending + Waiting)" }</option>
                                                              <option value="pending">{ "Pending" }</option>
                                                              <option value="waiting">{ "Waiting" }</option>
                                                              <option value="completed">{ "Completed" }</option>
                                                              <option value="deleted">{ "Deleted" }</option>
                                                          </select>
                                                      </div>
                                                      <div class="field">
                                                          <label>{ "Project" }</label>
                                                          <select
                                                              class="tag-select"
                                                              value={(*all_filter_project).clone().unwrap_or_default()}
                                                              onchange={on_all_project_change}
                                                          >
                                                              <option value="">{ "All Projects" }</option>
                                                              {
                                                                  for project_facets.iter().map(|(project, count)| html! {
                                                                      <option value={project.clone()}>{ format!("{project} ({count})") }</option>
                                                                  })
                                                              }
                                                          </select>
                                                      </div>
                                                      <div class="field">
                                                          <label>{ "Tag" }</label>
                                                          <select
                                                              class="tag-select"
                                                              value={(*all_filter_tag).clone().unwrap_or_default()}
                                                              onchange={on_all_tag_change}
                                                          >
                                                              <option value="">{ "All Tags" }</option>
                                                              {
                                                                  for tag_facets.iter().map(|(tag, count)| html! {
                                                                      <option value={tag.clone()}>{ format!("{tag} ({count})") }</option>
                                                                  })
                                                              }
                                                          </select>
                                                      </div>
                                                      <div class="field">
                                                          <label>{ "Priority" }</label>
                                                          <select
                                                              class="tag-select"
                                                              value={(*all_filter_priority).clone()}
                                                              onchange={on_all_priority_change}
                                                          >
                                                              <option value="all">{ "All Priorities" }</option>
                                                              <option value="low">{ "Low" }</option>
                                                              <option value="medium">{ "Medium" }</option>
                                                              <option value="high">{ "High" }</option>
                                                              <option value="none">{ "None" }</option>
                                                          </select>
                                                      </div>
                                                      <div class="field">
                                                          <label>{ "Due" }</label>
                                                          <select
                                                              class="tag-select"
                                                              value={(*all_filter_due).clone()}
                                                              onchange={on_all_due_change}
                                                          >
                                                              <option value="all">{ "All" }</option>
                                                              <option value="has_due">{ "Has Due Date" }</option>
                                                              <option value="no_due">{ "No Due Date" }</option>
                                                          </select>
                                                      </div>
                                                      <div class="actions">
                                                          <button class="btn" onclick={on_all_filters_clear.clone()}>{ "Clear Filters" }</button>
                                                      </div>
                                                  </div>
                                              </div>
                                              <Details
                                                  task={selected_task.clone()}
                                                  tag_colors={tag_colors.clone()}
                                                  on_done={on_done.clone()}
                                                  on_delete={on_delete.clone()}
                                                  on_edit={on_edit.clone()}
                                              />
                                          </div>
                                      }
                                  } else if *active_view == "tags" && selected_task.is_none() {
                                      html! {
                                          <FacetPanel
                                              title={"Tags".to_string()}
                                              selected={(*active_tag).clone()}
                                              items={tag_facets}
                                              on_select={on_choose_tag}
                                          />
                                      }
                                  } else {
                                      html! {
                                          <Details
                                              task={selected_task.clone()}
                                              tag_colors={tag_colors.clone()}
                                              on_done={on_done.clone()}
                                              on_delete={on_delete.clone()}
                                              on_edit={on_edit.clone()}
                                          />
                                      }
                                  }
                              }
                          </>
                      }
                  }
              }
          </div>

          {
              if let Some(state) = (*modal_state).clone() {
                  let submit_state = state.clone();
                  let is_busy = *modal_busy;
                  let board_options: Vec<(String, String)> = kanban_boards
                      .iter()
                      .map(|board| (board.id.clone(), board.name.clone()))
                      .collect();
                  let selected_board_name = state
                      .draft_board_id
                      .as_ref()
                      .and_then(|board_id| {
                          board_options
                              .iter()
                              .find(|(id, _name)| id == board_id)
                              .map(|(_id, name)| name.clone())
                      });
                  let picker_value_options = state
                      .picker_key
                      .as_deref()
                      .and_then(|id| tag_schema.key(id))
                      .map(|key| key.values.clone())
                      .unwrap_or_default();
                  let on_save_click = {
                      let on_modal_submit = on_modal_submit.clone();
                      let submit_state = submit_state.clone();
                      Callback::from(move |_| {
                          ui_debug("button.save.click", "save click fired");
                          on_modal_submit.emit(submit_state.clone());
                      })
                  };
                  let on_add_custom_tag = {
                      let modal_state = modal_state.clone();
                      Callback::from(move |_| {
                          if let Some(mut current) = (*modal_state).clone() {
                              let custom_tags = split_tags(&current.draft_custom_tag);
                              if custom_tags.is_empty() {
                                  return;
                              }

                              let mut added = 0_usize;
                              for tag in custom_tags {
                                  if push_tag_unique(&mut current.draft_tags, tag) {
                                      added += 1;
                                  }
                              }
                              tracing::debug!(added, "added custom tags");
                              current.draft_custom_tag.clear();
                              current.error = None;
                              modal_state.set(Some(current));
                          }
                      })
                  };
                  let on_picker_key_change = {
                      let modal_state = modal_state.clone();
                      let tag_schema = tag_schema.clone();
                      Callback::from(move |e: web_sys::Event| {
                          let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                          let key = select.value();
                          if let Some(mut current) = (*modal_state).clone() {
                              if key.trim().is_empty() {
                                  current.picker_key = None;
                                  current.picker_value = None;
                              } else {
                                  current.picker_key = Some(key.clone());
                                  current.picker_value = first_value_for_key(&tag_schema, &key);
                              }
                              current.error = None;
                              modal_state.set(Some(current));
                          }
                      })
                  };
                  let on_picker_value_change = {
                      let modal_state = modal_state.clone();
                      Callback::from(move |e: web_sys::Event| {
                          let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                          let value = select.value();
                          if let Some(mut current) = (*modal_state).clone() {
                              current.picker_value = if value.trim().is_empty() {
                                  None
                              } else {
                                  Some(value)
                              };
                              current.error = None;
                              modal_state.set(Some(current));
                          }
                      })
                  };
                  let on_add_picker_tag = {
                      let modal_state = modal_state.clone();
                      let tag_schema = tag_schema.clone();
                      Callback::from(move |_| {
                          if let Some(mut current) = (*modal_state).clone() {
                              let Some(key) = current.picker_key.clone() else {
                                  return;
                              };
                              let Some(value) = current.picker_value.clone() else {
                                  return;
                              };

                              let key = key.trim();
                              let value = value.trim();
                              if key.is_empty() || value.is_empty() {
                                  return;
                              }

                              let tag = format!("{key}:{value}");
                              if is_single_select_key(&tag_schema, key) {
                                  remove_tags_for_key(&mut current.draft_tags, key);
                              }
                              if push_tag_unique(&mut current.draft_tags, tag.clone()) {
                                  tracing::debug!(tag = %tag, "added picker tag");
                              } else {
                                  tracing::debug!(tag = %tag, "picker tag already present");
                              }

                              current.error = None;
                              modal_state.set(Some(current));
                          }
                      })
                  };
                  let on_board_change = {
                      let modal_state = modal_state.clone();
                      Callback::from(move |e: web_sys::Event| {
                          let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                          let value = select.value();
                          if let Some(mut current) = (*modal_state).clone() {
                              current.draft_board_id = if value.trim().is_empty() {
                                  None
                              } else {
                                  Some(value)
                              };
                              current.error = None;
                              modal_state.set(Some(current));
                          }
                      })
                  };
                  let on_recurrence_pattern_change = {
                      let modal_state = modal_state.clone();
                      Callback::from(move |e: web_sys::Event| {
                          let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                          let value = select.value();
                          if let Some(mut current) = (*modal_state).clone() {
                              current.recurrence_pattern = value;
                              current.error = None;
                              modal_state.set(Some(current));
                          }
                      })
                  };
                  let on_recurrence_time_change = {
                      let modal_state = modal_state.clone();
                      Callback::from(move |e: web_sys::InputEvent| {
                          let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                          if let Some(mut current) = (*modal_state).clone() {
                              current.recurrence_time = input.value();
                              current.error = None;
                              modal_state.set(Some(current));
                          }
                      })
                  };
                  let on_recurrence_month_day_change = {
                      let modal_state = modal_state.clone();
                      Callback::from(move |e: web_sys::InputEvent| {
                          let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                          if let Some(mut current) = (*modal_state).clone() {
                              current.recurrence_month_day = input.value();
                              current.error = None;
                              modal_state.set(Some(current));
                          }
                      })
                  };
                  html! {
                      <div class="modal-backdrop">
                          <div class="modal">
                              <div class="header">
                                  {
                                      match state.mode {
                                          ModalMode::Add => "Add Task",
                                          ModalMode::Edit(_) => "Edit Task",
                                      }
                                  }
                              </div>
                              <div class="content">
                                  {
                                      if let Some(err) = state.error.clone() {
                                          html! { <div class="form-error">{ err }</div> }
                                      } else {
                                          html! {}
                                      }
                                  }
                                  <div class="field">
                                      <label>{ "Title" }</label>
                                      <input
                                          value={state.draft_title.clone()}
                                          placeholder="Required task title"
                                          oninput={{
                                              let modal_state = modal_state.clone();
                                              Callback::from(move |e: web_sys::InputEvent| {
                                                  let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                  if let Some(mut current) = (*modal_state).clone() {
                                                      current.draft_title = input.value();
                                                      current.error = None;
                                                      modal_state.set(Some(current));
                                                  }
                                              })
                                          }}
                                      />
                                  </div>
                                  <div class="field">
                                      <label>{ "Description (optional)" }</label>
                                      <input
                                          value={state.draft_desc.clone()}
                                          placeholder="Optional details"
                                          oninput={{
                                              let modal_state = modal_state.clone();
                                              Callback::from(move |e: web_sys::InputEvent| {
                                                  let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                  if let Some(mut current) = (*modal_state).clone() {
                                                      current.draft_desc = input.value();
                                                      current.error = None;
                                                      modal_state.set(Some(current));
                                                  }
                                              })
                                          }}
                                      />
                                  </div>
                                  <div class="field">
                                      <label>{ "Project" }</label>
                                      <input
                                          value={state.draft_project.clone()}
                                          oninput={{
                                              let modal_state = modal_state.clone();
                                              Callback::from(move |e: web_sys::InputEvent| {
                                                  let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                  if let Some(mut current) = (*modal_state).clone() {
                                                      current.draft_project = input.value();
                                                      current.error = None;
                                                      modal_state.set(Some(current));
                                                  }
                                              })
                                          }}
                                      />
                                  </div>
                                  <div class="field">
                                      <label>
                                          {
                                              if state.lock_board_selection {
                                                  "Kanban Board (fixed by current board)"
                                              } else {
                                                  "Kanban Board (optional)"
                                              }
                                          }
                                      </label>
                                      <select
                                          class="tag-select"
                                          value={state.draft_board_id.clone().unwrap_or_default()}
                                          onchange={on_board_change}
                                          disabled={state.lock_board_selection}
                                      >
                                          <option value="">{ "No board (won't appear on Kanban)" }</option>
                                          {
                                              for board_options.iter().map(|(board_id, board_name)| html! {
                                                  <option value={board_id.clone()}>{ board_name.clone() }</option>
                                              })
                                          }
                                      </select>
                                      {
                                          if state.lock_board_selection {
                                              html! {
                                                  <div class="field-help">
                                                      {
                                                          selected_board_name
                                                              .map(|name| format!("This task will be added to board: {name}"))
                                                              .unwrap_or_else(|| "This task will be added to the active board.".to_string())
                                                      }
                                                  </div>
                                              }
                                          } else {
                                              html! {}
                                          }
                                      }
                                  </div>
                                  <div class="field">
                                      <label>{ "Custom Tag" }</label>
                                      <div class="field-inline">
                                          <input
                                              value={state.draft_custom_tag.clone()}
                                              placeholder="e.g. topic:corn or followup"
                                              oninput={{
                                                  let modal_state = modal_state.clone();
                                                  Callback::from(move |e: web_sys::InputEvent| {
                                                      let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                      if let Some(mut current) = (*modal_state).clone() {
                                                          current.draft_custom_tag = input.value();
                                                          current.error = None;
                                                          modal_state.set(Some(current));
                                                      }
                                                  })
                                              }}
                                          />
                                          <button
                                              type="button"
                                              class="btn"
                                              onclick={on_add_custom_tag}
                                          >
                                              { "Add" }
                                          </button>
                                      </div>
                                  </div>
                                  <div class="field">
                                      <label>{ "Pick Tag (key -> value)" }</label>
                                      <div class="tag-picker">
                                          <select
                                              class="tag-select"
                                              value={state.picker_key.clone().unwrap_or_default()}
                                              onchange={on_picker_key_change}
                                          >
                                              <option value="">{ "Select key" }</option>
                                              {
                                                  for tag_schema.keys.iter().filter(|key| key.id != BOARD_TAG_KEY).map(|key| {
                                                      let label = key.label.clone().unwrap_or_else(|| key.id.clone());
                                                      html! {
                                                          <option value={key.id.clone()}>
                                                              { format!("{label} ({})", key.id) }
                                                          </option>
                                                      }
                                                  })
                                              }
                                          </select>
                                          <select
                                              class="tag-select"
                                              value={state.picker_value.clone().unwrap_or_default()}
                                              onchange={on_picker_value_change}
                                              disabled={state.picker_key.is_none() || picker_value_options.is_empty()}
                                          >
                                              <option value="">{ "Select value" }</option>
                                              {
                                                  for picker_value_options.iter().map(|value| html! {
                                                      <option value={value.clone()}>{ value }</option>
                                                  })
                                              }
                                          </select>
                                          <button
                                              type="button"
                                              class="btn tag-plus"
                                              onclick={on_add_picker_tag}
                                              disabled={state.picker_key.is_none() || state.picker_value.is_none()}
                                              title="Add selected key:value tag"
                                          >
                                              { "+" }
                                          </button>
                                      </div>
                                  </div>
                                  <div class="field">
                                      <label>{ "Selected Tags" }</label>
                                      <div class="tag-list">
                                          {
                                              if state.draft_tags.is_empty() {
                                                  html! { <span class="tag-empty">{ "No tags selected yet." }</span> }
                                              } else {
                                                  html! {
                                                      <>
                                                          {
                                                              for state.draft_tags.iter().map(|tag| {
                                                                  let modal_state = modal_state.clone();
                                                                  let tag_to_remove = tag.clone();
                                                                  let chip_style = tag_chip_style(&tag_schema, tag, &tag_colors);
                                                                  html! {
                                                                      <span class="tag-chip" style={chip_style}>
                                                                          <span>{ tag }</span>
                                                                          <button
                                                                              type="button"
                                                                              class="tag-chip-remove"
                                                                              onclick={Callback::from(move |_| {
                                                                                  if let Some(mut current) = (*modal_state).clone() {
                                                                                      current.draft_tags.retain(|value| value != &tag_to_remove);
                                                                                      current.error = None;
                                                                                      modal_state.set(Some(current));
                                                                                  }
                                                                              })}
                                                                          >
                                                                              { "x" }
                                                                          </button>
                                                                      </span>
                                                                  }
                                                              })
                                                          }
                                                      </>
                                                  }
                                              }
                                          }
                                      </div>
                                  </div>
                                  <div class="field">
                                      <label>{ "Due" }</label>
                                      <input
                                          value={state.draft_due.clone()}
                                          placeholder="e.g. tomorrow, 2028, march, wed, 3:23pm, 2026-02-20"
                                          oninput={{
                                              let modal_state = modal_state.clone();
                                              Callback::from(move |e: web_sys::InputEvent| {
                                                  let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                  if let Some(mut current) = (*modal_state).clone() {
                                                      current.draft_due = input.value();
                                                      current.error = None;
                                                      modal_state.set(Some(current));
                                                  }
                                              })
                                          }}
                                      />
                                  </div>
                                  <div class="field">
                                      <label>{ "Recurrence" }</label>
                                      <select
                                          class="tag-select"
                                          value={state.recurrence_pattern.clone()}
                                          onchange={on_recurrence_pattern_change}
                                      >
                                          <option value="none">{ "None" }</option>
                                          <option value="daily">{ "Daily" }</option>
                                          <option value="weekly">{ "Weekly" }</option>
                                          <option value="months">{ "Months" }</option>
                                          <option value="monthly">{ "Monthly" }</option>
                                          <option value="yearly">{ "Yearly" }</option>
                                      </select>
                                  </div>
                                  {
                                      if state.recurrence_pattern != "none" {
                                          html! {
                                              <div class="field">
                                                  <label>{ "Recurring Time" }</label>
                                                  <input
                                                      value={state.recurrence_time.clone()}
                                                      placeholder="e.g. 03:23pm or 15:23"
                                                      oninput={on_recurrence_time_change}
                                                  />
                                              </div>
                                          }
                                      } else {
                                          html! {}
                                      }
                                  }
                                  {
                                      if state.recurrence_pattern == "weekly" {
                                          html! {
                                              <div class="field">
                                                  <label>{ "Weekly Days" }</label>
                                                  <div class="toggle-grid">
                                                      {
                                                          for WEEKDAY_KEYS.iter().map(|day| {
                                                              let day_key = (*day).to_string();
                                                              let day_label = day_key.to_ascii_uppercase();
                                                              let is_active = state.recurrence_days.iter().any(|entry| entry == &day_key);
                                                              let modal_state = modal_state.clone();
                                                              html! {
                                                                  <button
                                                                      type="button"
                                                                      class={classes!("toggle-btn", is_active.then_some("active"))}
                                                                      onclick={Callback::from(move |_| {
                                                                          if let Some(mut current) = (*modal_state).clone() {
                                                                              if current.recurrence_days.iter().any(|entry| entry == &day_key) {
                                                                                  current.recurrence_days.retain(|entry| entry != &day_key);
                                                                              } else {
                                                                                  current.recurrence_days.push(day_key.clone());
                                                                              }
                                                                              current.error = None;
                                                                              modal_state.set(Some(current));
                                                                          }
                                                                      })}
                                                                  >
                                                                      { day_label }
                                                                  </button>
                                                              }
                                                          })
                                                      }
                                                  </div>
                                              </div>
                                          }
                                      } else {
                                          html! {}
                                      }
                                  }
                                  {
                                      if state.recurrence_pattern == "monthly"
                                          || state.recurrence_pattern == "months"
                                          || state.recurrence_pattern == "yearly"
                                      {
                                          html! {
                                              <>
                                                  <div class="field">
                                                      <label>{ "Months" }</label>
                                                      <div class="toggle-grid months">
                                                          {
                                                              for MONTH_KEYS.iter().map(|month| {
                                                                  let month_key = (*month).to_string();
                                                                  let month_label = month_key.to_ascii_uppercase();
                                                                  let is_active = state.recurrence_months.iter().any(|entry| entry == &month_key);
                                                                  let modal_state = modal_state.clone();
                                                                  html! {
                                                                      <button
                                                                          type="button"
                                                                          class={classes!("toggle-btn", is_active.then_some("active"))}
                                                                          onclick={Callback::from(move |_| {
                                                                              if let Some(mut current) = (*modal_state).clone() {
                                                                                  if current.recurrence_months.iter().any(|entry| entry == &month_key) {
                                                                                      current.recurrence_months.retain(|entry| entry != &month_key);
                                                                                  } else {
                                                                                      current.recurrence_months.push(month_key.clone());
                                                                                  }
                                                                                  current.error = None;
                                                                                  modal_state.set(Some(current));
                                                                              }
                                                                          })}
                                                                      >
                                                                          { month_label }
                                                                      </button>
                                                                  }
                                                              })
                                                          }
                                                      </div>
                                                  </div>
                                                  <div class="field">
                                                      <label>{ "Month Day(s)" }</label>
                                                      <input
                                                          value={state.recurrence_month_day.clone()}
                                                          placeholder="e.g. 1 or 1,15,28"
                                                          oninput={on_recurrence_month_day_change}
                                                      />
                                                  </div>
                                              </>
                                          }
                                      } else {
                                          html! {}
                                      }
                                  }
                                  <div class="footer">
                                      <button
                                          id="modal-cancel-btn"
                                          type="button"
                                          class="btn"
                                          onclick={on_modal_close_click.clone()}
                                      >
                                          { "Cancel" }
                                      </button>
                                      <button
                                          id="modal-save-btn"
                                          type="button"
                                          class="btn"
                                          onclick={on_save_click}
                                          disabled={is_busy}
                                      >
                                          { if is_busy { "Saving..." } else { "Save" } }
                                      </button>
                                  </div>
                              </div>
                          </div>
                      </div>
                  }
              } else {
                  html! {}
              }
          }

          {
              if *kanban_create_open {
                  html! {
                      <div class="modal-backdrop" onclick={on_close_create_kanban_board.clone()}>
                          <div class="modal modal-sm" onclick={Callback::from(|e: yew::MouseEvent| e.stop_propagation())}>
                              <div class="header">{ "New Kanban Board" }</div>
                              <div class="content">
                                  <div class="field">
                                      <label>{ "Board Name" }</label>
                                      <input
                                          value={(*kanban_create_input).clone()}
                                          oninput={on_create_kanban_input}
                                          placeholder="Board name"
                                      />
                                  </div>
                                  <div class="footer">
                                      <button type="button" class="btn" onclick={on_close_create_kanban_board.clone()}>{ "Cancel" }</button>
                                      <button
                                          type="button"
                                          class="btn"
                                          onclick={on_submit_create_kanban_board}
                                          disabled={(*kanban_create_input).trim().is_empty()}
                                      >
                                          { "Create" }
                                      </button>
                                  </div>
                              </div>
                          </div>
                      </div>
                  }
              } else {
                  html! {}
              }
          }

          {
              if *kanban_rename_open {
                  html! {
                      <div class="modal-backdrop" onclick={on_close_rename_kanban_board.clone()}>
                          <div class="modal modal-sm" onclick={Callback::from(|e: yew::MouseEvent| e.stop_propagation())}>
                              <div class="header">{ "Rename Kanban Board" }</div>
                              <div class="content">
                                  <div class="field">
                                      <label>{ "Board Name" }</label>
                                      <input
                                          value={(*kanban_rename_input).clone()}
                                          oninput={on_rename_kanban_input}
                                      />
                                  </div>
                                  <div class="footer">
                                      <button type="button" class="btn" onclick={on_close_rename_kanban_board.clone()}>{ "Cancel" }</button>
                                      <button
                                          type="button"
                                          class="btn"
                                          onclick={on_submit_rename_kanban_board}
                                          disabled={(*kanban_rename_input).trim().is_empty()}
                                      >
                                          { "Save" }
                                      </button>
                                  </div>
                              </div>
                          </div>
                      </div>
                  }
              } else {
                  html! {}
              }
          }

          {
              if let Some(ext_modal) = (*external_calendar_modal).clone() {
                  let submit_state = ext_modal.clone();
                  let is_busy = *external_calendar_busy;
                  let on_save_click = {
                      let on_submit_external_calendar = on_submit_external_calendar.clone();
                      Callback::from(move |_| on_submit_external_calendar.emit(submit_state.clone()))
                  };
                  html! {
                      <div class="modal-backdrop" onclick={on_close_external_calendar_modal.clone()}>
                          <div class="modal modal-md" onclick={Callback::from(|e: yew::MouseEvent| e.stop_propagation())}>
                              <div class="header">
                                  {
                                      match ext_modal.mode {
                                          ExternalCalendarModalMode::Add => "Add External Calendar",
                                          ExternalCalendarModalMode::Edit => "Edit External Calendar",
                                      }
                                  }
                              </div>
                              <div class="content">
                                  {
                                      if let Some(err) = ext_modal.error.clone() {
                                          html! { <div class="form-error">{ err }</div> }
                                      } else {
                                          html! {}
                                      }
                                  }
                                  <div class="field field-inline-check">
                                      <label>{ "Enable This Calendar" }</label>
                                      <input
                                          type="checkbox"
                                          checked={ext_modal.source.enabled}
                                          onchange={{
                                              let external_calendar_modal = external_calendar_modal.clone();
                                              Callback::from(move |e: web_sys::Event| {
                                                  if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                      if let Some(mut current) = (*external_calendar_modal).clone() {
                                                          current.source.enabled = input.checked();
                                                          current.error = None;
                                                          external_calendar_modal.set(Some(current));
                                                      }
                                                  }
                                              })
                                          }}
                                      />
                                  </div>
                                  <div class="field">
                                      <label>{ "Calendar Name" }</label>
                                      <input
                                          value={ext_modal.source.name.clone()}
                                          oninput={{
                                              let external_calendar_modal = external_calendar_modal.clone();
                                              Callback::from(move |e: web_sys::InputEvent| {
                                                  let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                  if let Some(mut current) = (*external_calendar_modal).clone() {
                                                      current.source.name = input.value();
                                                      current.error = None;
                                                      external_calendar_modal.set(Some(current));
                                                  }
                                              })
                                          }}
                                      />
                                  </div>
                                  <div class="field">
                                      <label>{ "Color" }</label>
                                      <input
                                          type="color"
                                          value={ext_modal.source.color.clone()}
                                          oninput={{
                                              let external_calendar_modal = external_calendar_modal.clone();
                                              Callback::from(move |e: web_sys::InputEvent| {
                                                  let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                  if let Some(mut current) = (*external_calendar_modal).clone() {
                                                      current.source.color = input.value();
                                                      current.error = None;
                                                      external_calendar_modal.set(Some(current));
                                                  }
                                              })
                                          }}
                                      />
                                  </div>
                                  <div class="field">
                                      <label>{ "Location (ICS URL)" }</label>
                                      <input
                                          value={ext_modal.source.location.clone()}
                                          placeholder="https://example.com/calendar.ics"
                                          oninput={{
                                              let external_calendar_modal = external_calendar_modal.clone();
                                              Callback::from(move |e: web_sys::InputEvent| {
                                                  let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                  if let Some(mut current) = (*external_calendar_modal).clone() {
                                                      current.source.location = input.value();
                                                      current.error = None;
                                                      external_calendar_modal.set(Some(current));
                                                  }
                                              })
                                          }}
                                      />
                                  </div>
                                  <div class="field">
                                      <label>{ "Refresh Calendar" }</label>
                                      <select
                                          class="tag-select"
                                          value={ext_modal.source.refresh_minutes.to_string()}
                                          onchange={{
                                              let external_calendar_modal = external_calendar_modal.clone();
                                              Callback::from(move |e: web_sys::Event| {
                                                  let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                  if let Some(mut current) = (*external_calendar_modal).clone() {
                                                      let parsed = select.value().parse::<u32>().ok().unwrap_or(30);
                                                      current.source.refresh_minutes = parsed.max(1);
                                                      current.error = None;
                                                      external_calendar_modal.set(Some(current));
                                                  }
                                              })
                                          }}
                                      >
                                          <option value="5">{ "Every 5 minutes" }</option>
                                          <option value="15">{ "Every 15 minutes" }</option>
                                          <option value="30">{ "Every 30 minutes" }</option>
                                          <option value="60">{ "Every 60 minutes" }</option>
                                          <option value="360">{ "Every 6 hours" }</option>
                                          <option value="1440">{ "Every 24 hours" }</option>
                                      </select>
                                  </div>
                                  <div class="field field-inline-check">
                                      <label>{ "Read Only" }</label>
                                      <input
                                          type="checkbox"
                                          checked={ext_modal.source.read_only}
                                          onchange={{
                                              let external_calendar_modal = external_calendar_modal.clone();
                                              Callback::from(move |e: web_sys::Event| {
                                                  if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                      if let Some(mut current) = (*external_calendar_modal).clone() {
                                                          current.source.read_only = input.checked();
                                                          current.error = None;
                                                          external_calendar_modal.set(Some(current));
                                                      }
                                                  }
                                              })
                                          }}
                                      />
                                  </div>
                                  <div class="field field-inline-check">
                                      <label>{ "Show Reminders" }</label>
                                      <input
                                          type="checkbox"
                                          checked={ext_modal.source.show_reminders}
                                          onchange={{
                                              let external_calendar_modal = external_calendar_modal.clone();
                                              Callback::from(move |e: web_sys::Event| {
                                                  if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                      if let Some(mut current) = (*external_calendar_modal).clone() {
                                                          current.source.show_reminders = input.checked();
                                                          current.error = None;
                                                          external_calendar_modal.set(Some(current));
                                                      }
                                                  }
                                              })
                                          }}
                                      />
                                  </div>
                                  <div class="field field-inline-check">
                                      <label>{ "Offline Support" }</label>
                                      <input
                                          type="checkbox"
                                          checked={ext_modal.source.offline_support}
                                          onchange={{
                                              let external_calendar_modal = external_calendar_modal.clone();
                                              Callback::from(move |e: web_sys::Event| {
                                                  if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                      if let Some(mut current) = (*external_calendar_modal).clone() {
                                                          current.source.offline_support = input.checked();
                                                          current.error = None;
                                                          external_calendar_modal.set(Some(current));
                                                      }
                                                  }
                                              })
                                          }}
                                      />
                                  </div>
                                  <div class="footer">
                                      <button type="button" class="btn" onclick={on_close_external_calendar_modal.clone()}>{ "Cancel" }</button>
                                      <button type="button" class="btn" onclick={on_save_click} disabled={is_busy}>
                                          { if is_busy { "Saving..." } else { "Save" } }
                                      </button>
                                  </div>
                              </div>
                          </div>
                      </div>
                  }
              } else {
                  html! {}
              }
          }
      </div>
  }
}

fn load_theme_mode() -> ThemeMode {
  let stored = web_sys::window()
    .and_then(|window| {
      window
        .local_storage()
        .ok()
        .flatten()
    })
    .and_then(|storage| {
      storage
        .get_item(THEME_STORAGE_KEY)
        .ok()
        .flatten()
    });

  match stored.as_deref() {
    | Some("night") => ThemeMode::Night,
    | _ => ThemeMode::Day
  }
}

fn save_theme_mode(theme: ThemeMode) {
  if let Some(storage) =
    web_sys::window().and_then(
      |window| {
        window
          .local_storage()
          .ok()
          .flatten()
      }
    )
  {
    let _ = storage.set_item(
      THEME_STORAGE_KEY,
      theme.storage_value()
    );
  }
}

fn load_workspace_tab() -> String {
  let stored = web_sys::window()
    .and_then(|window| {
      window
        .local_storage()
        .ok()
        .flatten()
    })
    .and_then(|storage| {
      storage
        .get_item(
          WORKSPACE_TAB_STORAGE_KEY
        )
        .ok()
        .flatten()
    });

  match stored.as_deref() {
    | Some("kanban") => {
      "kanban".to_string()
    }
    | Some("calendar") => {
      "calendar".to_string()
    }
    | _ => "tasks".to_string()
  }
}

fn save_workspace_tab(tab: &str) {
  if let Some(storage) =
    web_sys::window().and_then(
      |window| {
        window
          .local_storage()
          .ok()
          .flatten()
      }
    )
  {
    let _ = storage.set_item(
      WORKSPACE_TAB_STORAGE_KEY,
      tab
    );
  }
}

fn load_external_calendars()
-> Vec<ExternalCalendarSource> {
  let stored = web_sys::window()
    .and_then(|window| {
      window
        .local_storage()
        .ok()
        .flatten()
    })
    .and_then(|storage| {
      storage
        .get_item(
          EXTERNAL_CALENDARS_STORAGE_KEY
        )
        .ok()
        .flatten()
    });

  if let Some(raw) = stored {
    match serde_json::from_str::<
      Vec<ExternalCalendarSource>
    >(&raw)
    {
      | Ok(mut sources) => {
        sources.retain(|source| {
          !source.id.trim().is_empty()
            && !source
              .name
              .trim()
              .is_empty()
            && !source
              .location
              .trim()
              .is_empty()
        });
        return sources;
      }
      | Err(error) => {
        tracing::error!(
          %error,
          "failed parsing external \
           calendars from local storage"
        );
      }
    }
  }

  Vec::new()
}

fn save_external_calendars(
  sources: &[ExternalCalendarSource]
) {
  if let Some(storage) =
    web_sys::window().and_then(
      |window| {
        window
          .local_storage()
          .ok()
          .flatten()
      }
    )
    && let Ok(json) =
      serde_json::to_string(sources)
  {
    let _ = storage.set_item(
      EXTERNAL_CALENDARS_STORAGE_KEY,
      &json
    );
  }
}

fn new_external_calendar_source()
-> ExternalCalendarSource {
  ExternalCalendarSource {
    id:              Uuid::new_v4()
      .to_string(),
    name:            String::new(),
    color:           "#d64545"
      .to_string(),
    location:        String::new(),
    refresh_minutes: 30,
    enabled:         true,
    read_only:       true,
    show_reminders:  true,
    offline_support: true
  }
}

fn load_kanban_boards()
-> Vec<KanbanBoardDef> {
  let stored = web_sys::window()
    .and_then(|window| {
      window
        .local_storage()
        .ok()
        .flatten()
    })
    .and_then(|storage| {
      storage
        .get_item(
          KANBAN_BOARDS_STORAGE_KEY
        )
        .ok()
        .flatten()
    });

  if let Some(raw) = stored {
    match serde_json::from_str::<
      Vec<KanbanBoardDef>
    >(&raw)
    {
      | Ok(mut boards) => {
        boards.retain(|board| {
          !board.id.trim().is_empty()
            && !board
              .name
              .trim()
              .is_empty()
        });
        assign_unique_board_colors(
          &mut boards
        );
        if !boards.is_empty() {
          return boards;
        }
      }
      | Err(err) => {
        tracing::error!(
          error = %err,
          "failed parsing kanban \
           boards from storage"
        );
      }
    }
  }

  vec![KanbanBoardDef {
    id:    Uuid::new_v4().to_string(),
    name:  "Main".to_string(),
    color: default_board_color()
  }]
}

fn save_kanban_boards(
  boards: &[KanbanBoardDef]
) {
  if let Some(storage) =
    web_sys::window().and_then(
      |window| {
        window
          .local_storage()
          .ok()
          .flatten()
      }
    )
    && let Ok(json) =
      serde_json::to_string(boards)
  {
    let _ = storage.set_item(
      KANBAN_BOARDS_STORAGE_KEY,
      &json
    );
  }
}

fn load_active_kanban_board(
  boards: &[KanbanBoardDef]
) -> Option<String> {
  let stored = web_sys::window()
    .and_then(|window| {
      window
        .local_storage()
        .ok()
        .flatten()
    })
    .and_then(|storage| {
      storage
        .get_item(
          KANBAN_ACTIVE_BOARD_STORAGE_KEY
        )
        .ok()
        .flatten()
    });

  if let Some(id) = stored
    && boards
      .iter()
      .any(|board| board.id == id)
  {
    return Some(id);
  }

  boards
    .first()
    .map(|board| board.id.clone())
}

fn save_active_kanban_board(
  board_id: Option<&str>
) {
  if let Some(storage) =
    web_sys::window().and_then(
      |window| {
        window
          .local_storage()
          .ok()
          .flatten()
      }
    )
  {
    match board_id {
      | Some(id) => {
        let _ = storage.set_item(
          KANBAN_ACTIVE_BOARD_STORAGE_KEY,
          id
        );
      }
      | None => {
        let _ = storage.remove_item(
          KANBAN_ACTIVE_BOARD_STORAGE_KEY
        );
      }
    }
  }
}

fn default_board_color() -> String {
  "hsl(212 74% 54%)".to_string()
}

fn board_color_candidate(
  seed: usize
) -> String {
  let hue =
    seed.saturating_mul(47) % 360;
  format!("hsl({hue} 74% 54%)")
}

fn next_board_color(
  boards: &[KanbanBoardDef]
) -> String {
  let mut used =
    BTreeSet::<String>::new();
  for board in boards {
    used.insert(
      board
        .color
        .trim()
        .to_ascii_lowercase()
    );
  }

  for offset in 0_usize..512_usize {
    let candidate =
      board_color_candidate(
        boards
          .len()
          .saturating_add(offset)
      );
    if !used.contains(
      &candidate.to_ascii_lowercase()
    ) {
      return candidate;
    }
  }

  default_board_color()
}

fn assign_unique_board_colors(
  boards: &mut [KanbanBoardDef]
) {
  let mut used =
    BTreeSet::<String>::new();
  for (index, board) in
    boards.iter_mut().enumerate()
  {
    let mut color =
      board.color.trim().to_string();
    if color.is_empty() {
      color =
        board_color_candidate(index);
    }

    let mut key =
      color.to_ascii_lowercase();
    if used.contains(&key) {
      for offset in 0_usize..512_usize {
        let candidate =
          board_color_candidate(
            index
              .saturating_add(offset)
          );
        let candidate_key = candidate
          .to_ascii_lowercase();
        if !used
          .contains(&candidate_key)
        {
          color = candidate;
          key = candidate_key;
          break;
        }
      }
    }

    board.color = color;
    used.insert(key);
  }
}

fn make_unique_board_name(
  boards: &[KanbanBoardDef],
  requested: &str
) -> String {
  make_unique_board_name_except(
    boards, requested, ""
  )
}

fn make_unique_board_name_except(
  boards: &[KanbanBoardDef],
  requested: &str,
  except_board_id: &str
) -> String {
  let base = requested.trim();
  if base.is_empty() {
    return "Board".to_string();
  }

  let mut candidate = base.to_string();
  let mut suffix = 2_u32;
  while boards.iter().any(|board| {
    board.id != except_board_id
      && board
        .name
        .eq_ignore_ascii_case(
          &candidate
        )
  }) {
    candidate =
      format!("{base} {suffix}");
    suffix = suffix.saturating_add(1);
  }

  candidate
}

fn board_id_from_task_tags(
  boards: &[KanbanBoardDef],
  tags: &[String]
) -> Option<String> {
  let board_id = first_tag_value(
    tags,
    BOARD_TAG_KEY
  )?
  .to_string();
  boards
    .iter()
    .find(|board| board.id == board_id)
    .map(|board| board.id.clone())
}

fn build_kanban_board_color_map(
  boards: &[KanbanBoardDef]
) -> BTreeMap<String, String> {
  boards
    .iter()
    .map(|board| {
      (
        board.id.clone(),
        normalize_marker_color(
          board.color.as_str()
        )
      )
    })
    .collect()
}

fn build_external_calendar_color_map(
  calendars: &[ExternalCalendarSource]
) -> BTreeMap<String, String> {
  calendars
    .iter()
    .map(|source| {
      (
        source.id.clone(),
        normalize_marker_color(
          source.color.as_str()
        )
      )
    })
    .collect()
}

fn normalize_marker_color(
  value: &str
) -> String {
  let trimmed = value.trim();
  if trimmed.is_empty() {
    return CALENDAR_UNAFFILIATED_COLOR
      .to_string();
  }

  if let Some(hex) =
    normalize_hex_color(trimmed)
  {
    return hex;
  }

  trimmed.to_string()
}

fn normalize_hex_color(
  value: &str
) -> Option<String> {
  let raw = value
    .trim()
    .trim_start_matches('#');
  if raw.len() == 3
    && raw
      .chars()
      .all(|ch| ch.is_ascii_hexdigit())
  {
    let mut expanded =
      String::with_capacity(7);
    expanded.push('#');
    for ch in raw.chars() {
      expanded.push(ch);
      expanded.push(ch);
    }
    return Some(
      expanded.to_ascii_lowercase()
    );
  }

  if raw.len() == 6
    && raw
      .chars()
      .all(|ch| ch.is_ascii_hexdigit())
  {
    return Some(format!(
      "#{}",
      raw.to_ascii_lowercase()
    ));
  }

  None
}

fn load_tag_schema() -> TagSchema {
  match toml::from_str::<TagSchema>(
    TAG_SCHEMA_TOML
  ) {
    | Ok(schema)
      if !schema.keys.is_empty() =>
    {
      tracing::info!(
        version = schema.version,
        key_count = schema.keys.len(),
        "loaded tag schema"
      );
      schema
    }
    | Ok(_) => {
      tracing::warn!(
        "tag schema was empty; using \
         fallback schema"
      );
      TagSchema::default()
    }
    | Err(error) => {
      tracing::error!(%error, "failed to parse tag schema; using fallback schema");
      TagSchema::default()
    }
  }
}

fn kanban_columns_from_schema(
  schema: &TagSchema
) -> Vec<String> {
  let values = schema
    .key(KANBAN_TAG_KEY)
    .map(|entry| {
      entry
        .values
        .iter()
        .map(|value| {
          value.trim().to_string()
        })
        .filter(|value| {
          !value.is_empty()
        })
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();

  if values.is_empty() {
    vec![
      "todo".to_string(),
      "working".to_string(),
      "finished".to_string(),
    ]
  } else {
    values
  }
}

fn optional_text(
  text: &str
) -> Option<String> {
  let trimmed = text.trim();
  if trimmed.is_empty() {
    None
  } else {
    Some(trimmed.to_string())
  }
}

fn split_tags(
  text: &str
) -> Vec<String> {
  text
    .split_whitespace()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(ToString::to_string)
    .collect()
}

fn recurrence_from_tags(
  tags: &[String]
) -> (
  String,
  String,
  Vec<String>,
  Vec<String>,
  String
) {
  let pattern = first_tag_value(
    tags,
    RECUR_TAG_KEY
  )
  .map(normalize_recurrence_pattern)
  .unwrap_or_else(|| {
    "none".to_string()
  });

  let time = first_tag_value(
    tags,
    RECUR_TIME_TAG_KEY
  )
  .unwrap_or_default()
  .to_string();

  let days = split_csv_values(
    first_tag_value(
      tags,
      RECUR_DAYS_TAG_KEY
    )
    .unwrap_or_default()
  )
  .into_iter()
  .map(|entry| {
    entry
      .to_ascii_lowercase()
      .trim()
      .to_string()
  })
  .filter(|entry| {
    WEEKDAY_KEYS
      .iter()
      .any(|key| key == entry)
  })
  .collect::<Vec<_>>();

  let months = split_csv_values(
    first_tag_value(
      tags,
      RECUR_MONTHS_TAG_KEY
    )
    .unwrap_or_default()
  )
  .into_iter()
  .map(|entry| {
    entry
      .to_ascii_lowercase()
      .trim()
      .to_string()
  })
  .filter(|entry| {
    MONTH_KEYS
      .iter()
      .any(|key| key == entry)
  })
  .collect::<Vec<_>>();

  let month_day = first_tag_value(
    tags,
    RECUR_MONTH_DAY_TAG_KEY
  )
  .unwrap_or_default()
  .to_string();

  (
    pattern, time, days, months,
    month_day
  )
}

fn normalize_recurrence_pattern(
  value: &str
) -> String {
  match value
    .trim()
    .to_ascii_lowercase()
    .as_str()
  {
    | "daily" => "daily".to_string(),
    | "weekly" => "weekly".to_string(),
    | "months" => "months".to_string(),
    | "monthly" => {
      "monthly".to_string()
    }
    | "yearly" => "yearly".to_string(),
    | _ => "none".to_string()
  }
}

fn split_csv_values(
  value: &str
) -> Vec<String> {
  value
    .split(',')
    .map(str::trim)
    .filter(|entry| !entry.is_empty())
    .map(ToString::to_string)
    .collect()
}

fn is_recurrence_tag(
  tag: &str
) -> bool {
  matches!(
    tag.split_once(':'),
    Some((key, _))
      if key == RECUR_TAG_KEY
      || key == RECUR_TIME_TAG_KEY
      || key == RECUR_DAYS_TAG_KEY
      || key == RECUR_MONTHS_TAG_KEY
      || key == RECUR_MONTH_DAY_TAG_KEY
  )
}

fn append_recurrence_tags(
  tags: &mut Vec<String>,
  state: &ModalState
) {
  remove_tags_for_key(
    tags,
    RECUR_TAG_KEY
  );
  remove_tags_for_key(
    tags,
    RECUR_TIME_TAG_KEY
  );
  remove_tags_for_key(
    tags,
    RECUR_DAYS_TAG_KEY
  );
  remove_tags_for_key(
    tags,
    RECUR_MONTHS_TAG_KEY
  );
  remove_tags_for_key(
    tags,
    RECUR_MONTH_DAY_TAG_KEY
  );

  let pattern =
    normalize_recurrence_pattern(
      &state.recurrence_pattern
    );
  if pattern == "none" {
    return;
  }

  push_tag_unique(
    tags,
    format!(
      "{RECUR_TAG_KEY}:{pattern}"
    )
  );

  let recurrence_time =
    state.recurrence_time.trim();
  if !recurrence_time.is_empty() {
    push_tag_unique(
      tags,
      format!(
        "{RECUR_TIME_TAG_KEY}:\
         {recurrence_time}"
      )
    );
  }

  if pattern == "weekly"
    && !state.recurrence_days.is_empty()
  {
    let values = state
      .recurrence_days
      .iter()
      .map(|value| {
        value
          .trim()
          .to_ascii_lowercase()
      })
      .filter(|value| {
        WEEKDAY_KEYS
          .iter()
          .any(|key| key == value)
      })
      .collect::<Vec<_>>();
    if !values.is_empty() {
      push_tag_unique(
        tags,
        format!(
          "{RECUR_DAYS_TAG_KEY}:{}",
          values.join(",")
        )
      );
    }
  }

  if pattern == "monthly"
    || pattern == "months"
    || pattern == "yearly"
  {
    let months = state
      .recurrence_months
      .iter()
      .map(|value| {
        value
          .trim()
          .to_ascii_lowercase()
      })
      .filter(|value| {
        MONTH_KEYS
          .iter()
          .any(|key| key == value)
      })
      .collect::<Vec<_>>();
    if !months.is_empty() {
      push_tag_unique(
        tags,
        format!(
          "{RECUR_MONTHS_TAG_KEY}:{}",
          months.join(",")
        )
      );
    }

    let month_day =
      state.recurrence_month_day.trim();
    if !month_day.is_empty() {
      push_tag_unique(
        tags,
        format!(
          "{RECUR_MONTH_DAY_TAG_KEY}:\
           {month_day}"
        )
      );
    }
  }
}

fn collect_tags_for_submit(
  state: &ModalState,
  board_tag: Option<String>,
  ensure_kanban_lane: bool,
  default_kanban_lane: &str
) -> Vec<String> {
  let mut tags =
    state.draft_tags.clone();
  for tag in
    split_tags(&state.draft_custom_tag)
  {
    push_tag_unique(&mut tags, tag);
  }

  remove_tags_for_key(
    &mut tags,
    BOARD_TAG_KEY
  );
  if let Some(tag) = board_tag {
    push_tag_unique(&mut tags, tag);
  }

  append_recurrence_tags(
    &mut tags, state
  );

  if ensure_kanban_lane
    && !tags.iter().any(|tag| {
      tag.starts_with(&format!(
        "{KANBAN_TAG_KEY}:"
      ))
    })
  {
    push_tag_unique(
      &mut tags,
      format!(
        "{KANBAN_TAG_KEY}:{}",
        default_kanban_lane
      )
    );
  }

  tags
}

fn push_tag_unique(
  tags: &mut Vec<String>,
  tag: String
) -> bool {
  let trimmed = tag.trim();
  if trimmed.is_empty() {
    return false;
  }

  if tags
    .iter()
    .any(|existing| existing == trimmed)
  {
    return false;
  }

  tags.push(trimmed.to_string());
  true
}

fn first_value_for_key(
  schema: &TagSchema,
  key: &str
) -> Option<String> {
  schema.key(key).and_then(|entry| {
    entry.values.first().cloned()
  })
}

fn is_single_select_key(
  schema: &TagSchema,
  key: &str
) -> bool {
  schema
    .key(key)
    .and_then(|entry| {
      entry.selection.as_deref()
    })
    .is_some_and(|selection| {
      selection
        .eq_ignore_ascii_case("single")
    })
}

fn remove_tags_for_key(
  tags: &mut Vec<String>,
  key: &str
) {
  tags.retain(|existing| {
    match existing.split_once(':') {
      | Some((existing_key, _)) => {
        existing_key != key
      }
      | None => true
    }
  });
}

fn first_tag_value<'a>(
  tags: &'a [String],
  key: &str
) -> Option<&'a str> {
  tags.iter().find_map(|tag| {
    match tag.split_once(':') {
      | Some((existing_key, value))
        if existing_key == key =>
      {
        Some(value)
      }
      | _ => None
    }
  })
}

fn task_has_tag_value(
  tags: &[String],
  key: &str,
  value: &str
) -> bool {
  tags.iter().any(|tag| {
    matches!(
      tag.split_once(':'),
      Some((existing_key, existing_value))
        if existing_key == key
          && existing_value == value
    )
  })
}

fn remove_board_tag_for_id(
  tags: &mut Vec<String>,
  board_id: &str
) {
  tags.retain(|tag| {
    match tag.split_once(':') {
      | Some((key, value)) => {
        !(key == BOARD_TAG_KEY
          && value == board_id)
      }
      | None => true
    }
  });
}

fn tag_chip_style(
  schema: &TagSchema,
  tag: &str,
  tag_colors: &BTreeMap<String, String>
) -> String {
  let Some((key, _value)) =
    tag.split_once(':')
  else {
    return String::new();
  };

  let color = schema
    .key(key)
    .and_then(|entry| {
      entry.color.as_deref()
    })
    .map(ToString::to_string)
    .or_else(|| {
      tag_colors.get(key).cloned()
    })
    .unwrap_or_else(|| {
      deterministic_tag_key_color(key)
    });

  format!("--tag-key-color:{color};")
}

fn deterministic_tag_key_color(
  key: &str
) -> String {
  let mut hash: u32 = 0x811c9dc5;
  for byte in key.as_bytes() {
    hash ^= u32::from(*byte);
    hash = hash.wrapping_mul(16777619);
  }
  let hue = hash % 360;
  format!("hsl({hue} 72% 54%)")
}

fn tag_badge_style(
  tag: &str,
  tag_colors: &BTreeMap<String, String>
) -> String {
  let Some((key, _)) =
    tag.split_once(':')
  else {
    return String::new();
  };

  let color = tag_colors
    .get(key)
    .cloned()
    .unwrap_or_else(|| {
      deterministic_tag_key_color(key)
    });

  format!("--tag-key-color:{color};")
}

fn tag_color_for_schema_key(
  key: &TagKey
) -> Option<String> {
  if let Some(color) =
    key.color.as_ref()
  {
    return Some(color.clone());
  }
  if key.id.trim().is_empty() {
    return None;
  }
  Some(deterministic_tag_key_color(
    &key.id
  ))
}

fn build_tag_color_map(
  schema: &TagSchema
) -> BTreeMap<String, String> {
  schema
    .keys
    .iter()
    .filter_map(|key| {
      tag_color_for_schema_key(key).map(
        |color| (key.id.clone(), color)
      )
    })
    .collect()
}

fn calendar_true() -> bool {
  true
}

fn calendar_default_week_start()
-> String {
  "monday".to_string()
}

fn calendar_default_red_dot_limit()
-> usize {
  5_000
}

fn calendar_default_task_list_limit()
-> usize {
  200
}

fn calendar_default_task_list_window_days()
-> i64 {
  365
}

fn calendar_default_day_view_hour_end()
-> u32 {
  23
}

fn load_calendar_config()
-> CalendarConfig {
  match toml::from_str::<CalendarConfig>(
    CALENDAR_CONFIG_TOML
  ) {
    | Ok(mut config) => {
      sanitize_calendar_config(
        &mut config
      );
      tracing::info!(
        version = config.version,
        timezone = ?config.timezone,
        week_start = %config.policies.week_start,
        "loaded calendar config"
      );
      config
    }
    | Err(error) => {
      tracing::error!(%error, "failed parsing calendar config; using defaults");
      CalendarConfig::default()
    }
  }
}

fn sanitize_calendar_config(
  config: &mut CalendarConfig
) {
  if config
    .policies
    .week_start
    .trim()
    .is_empty()
  {
    config.policies.week_start =
      calendar_default_week_start();
  }

  if config.policies.red_dot_limit == 0
  {
    config.policies.red_dot_limit =
      calendar_default_red_dot_limit();
  }

  if config.policies.task_list_limit
    == 0
  {
    config.policies.task_list_limit =
      calendar_default_task_list_limit(
      );
  }

  if config
    .policies
    .task_list_window_days
    <= 0
  {
    config
      .policies
      .task_list_window_days =
      calendar_default_task_list_window_days(
      );
  }

  if config.day_view.hour_start > 23 {
    config.day_view.hour_start = 23;
  }
  if config.day_view.hour_end > 23 {
    config.day_view.hour_end = 23;
  }
  if config.day_view.hour_end
    < config.day_view.hour_start
  {
    config.day_view.hour_end =
      config.day_view.hour_start;
  }
}

fn load_calendar_view_mode()
-> CalendarViewMode {
  let stored = web_sys::window()
    .and_then(|window| {
      window
        .local_storage()
        .ok()
        .flatten()
    })
    .and_then(|storage| {
      storage
        .get_item(
          CALENDAR_VIEW_STORAGE_KEY
        )
        .ok()
        .flatten()
    });

  stored
    .as_deref()
    .and_then(
      CalendarViewMode::from_key
    )
    .map(|view| {
      match view {
        | CalendarViewMode::List => {
          CalendarViewMode::Month
        }
        | other => other
      }
    })
    .unwrap_or(CalendarViewMode::Month)
}

fn save_calendar_view_mode(
  view: CalendarViewMode
) {
  if let Some(storage) =
    web_sys::window().and_then(
      |window| {
        window
          .local_storage()
          .ok()
          .flatten()
      }
    )
  {
    let _ = storage.set_item(
      CALENDAR_VIEW_STORAGE_KEY,
      view.as_key()
    );
  }
}

fn resolve_calendar_timezone(
  config: &CalendarConfig
) -> Tz {
  if let Some(raw) =
    config.timezone.as_ref()
    && let Some(tz) =
      parse_calendar_timezone(
        raw,
        "calendar.toml"
      )
  {
    return tz;
  }

  if let Ok(time_config) =
    toml::from_str::<ProjectTimeConfig>(
      PROJECT_TIME_CONFIG_TOML
    )
  {
    let timezone = time_config
      .timezone
      .or_else(|| {
        time_config.time.and_then(
          |section| section.timezone
        )
      });
    if let Some(raw) = timezone
      && let Some(tz) =
        parse_calendar_timezone(
          &raw,
          "rivet-time.toml"
        )
    {
      return tz;
    }
  } else {
    tracing::warn!(
      "failed to parse embedded \
       rivet-time.toml; falling back \
       to default timezone"
    );
  }

  parse_calendar_timezone(
    DEFAULT_CALENDAR_TIMEZONE,
    "calendar-default"
  )
  .unwrap_or(chrono_tz::UTC)
}

fn parse_calendar_timezone(
  raw: &str,
  source: &str
) -> Option<Tz> {
  let trimmed = raw.trim();
  if trimmed.is_empty() {
    return None;
  }

  match trimmed.parse::<Tz>() {
    | Ok(tz) => Some(tz),
    | Err(error) => {
      tracing::error!(
        source,
        timezone = %trimmed,
        error = %error,
        "invalid timezone id"
      );
      None
    }
  }
}

fn today_in_timezone(
  timezone: Tz
) -> NaiveDate {
  Utc::now()
    .with_timezone(&timezone)
    .date_naive()
}

fn calendar_week_start_day(
  raw: &str
) -> Weekday {
  if raw
    .trim()
    .eq_ignore_ascii_case("sunday")
  {
    Weekday::Sun
  } else {
    Weekday::Mon
  }
}

fn shift_calendar_focus(
  current: NaiveDate,
  view: CalendarViewMode,
  step: i64,
  _week_start: Weekday
) -> NaiveDate {
  match view {
    | CalendarViewMode::Year => {
      shift_years(current, step as i32)
    }
    | CalendarViewMode::Quarter => {
      shift_months(
        current,
        (step * 3) as i32
      )
    }
    | CalendarViewMode::Month => {
      shift_months(current, step as i32)
    }
    | CalendarViewMode::Week => {
      add_days(current, step * 7)
    }
    | CalendarViewMode::Day => {
      add_days(current, step)
    }
    | CalendarViewMode::List => {
      add_days(current, step * 7)
    }
  }
}

fn shift_years(
  date: NaiveDate,
  years: i32
) -> NaiveDate {
  let year =
    date.year().saturating_add(years);
  let month = date.month();
  let day = date
    .day()
    .min(days_in_month(year, month));
  NaiveDate::from_ymd_opt(
    year, month, day
  )
  .unwrap_or(date)
}

fn shift_months(
  date: NaiveDate,
  months: i32
) -> NaiveDate {
  let mut year = date.year();
  let mut month =
    date.month() as i32 + months;

  while month < 1 {
    month += 12;
    year = year.saturating_sub(1);
  }
  while month > 12 {
    month -= 12;
    year = year.saturating_add(1);
  }

  let month = month as u32;
  let day = date
    .day()
    .min(days_in_month(year, month));
  NaiveDate::from_ymd_opt(
    year, month, day
  )
  .unwrap_or(date)
}

fn first_day_of_month(
  year: i32,
  month: u32
) -> NaiveDate {
  NaiveDate::from_ymd_opt(
    year, month, 1
  )
  .unwrap_or(NaiveDate::MIN)
}

fn last_day_of_month(
  year: i32,
  month: u32
) -> NaiveDate {
  let (next_year, next_month) =
    if month >= 12 {
      (year.saturating_add(1), 1_u32)
    } else {
      (year, month + 1)
    };
  add_days(
    first_day_of_month(
      next_year, next_month
    ),
    -1
  )
}

fn days_in_month(
  year: i32,
  month: u32
) -> u32 {
  last_day_of_month(year, month).day()
}

fn add_days(
  date: NaiveDate,
  days: i64
) -> NaiveDate {
  date
    .checked_add_signed(Duration::days(
      days
    ))
    .unwrap_or(date)
}

fn start_of_week(
  day: NaiveDate,
  week_start: Weekday
) -> NaiveDate {
  let day_idx = day
    .weekday()
    .num_days_from_monday()
    as i64;
  let start_idx = week_start
    .num_days_from_monday()
    as i64;
  let diff =
    (7 + day_idx - start_idx) % 7;
  add_days(day, -diff)
}

fn collect_calendar_due_tasks(
  tasks: &[TaskDto],
  timezone: Tz,
  config: &CalendarConfig,
  board_colors: &BTreeMap<
    String,
    String
  >,
  calendar_colors: &BTreeMap<
    String,
    String
  >
) -> Vec<CalendarDueTask> {
  let mut entries = tasks
    .iter()
    .filter(|task| {
      calendar_status_visible(
        &task.status,
        &config.visibility
      )
    })
    .filter_map(|task| {
      let due_raw =
        task.due.as_ref()?;
      let due_utc =
        parse_taskwarrior_utc(
          due_raw.as_str()
        )?;
      let marker = marker_for_task(
        task,
        board_colors,
        calendar_colors
      );
      Some(CalendarDueTask {
        task: task.clone(),
        due_local: due_utc
          .with_timezone(&timezone),
        due_utc,
        marker
      })
    })
    .collect::<Vec<_>>();

  entries
    .sort_by_key(|entry| entry.due_utc);

  tracing::debug!(
    total_tasks = tasks.len(),
    due_tasks = entries.len(),
    timezone = %timezone,
    "calendar tasks collected"
  );
  entries
}

fn marker_for_task(
  task: &TaskDto,
  board_colors: &BTreeMap<
    String,
    String
  >,
  calendar_colors: &BTreeMap<
    String,
    String
  >
) -> CalendarTaskMarker {
  if let Some(calendar_id) =
    first_tag_value(
      &task.tags,
      CAL_SOURCE_TAG_KEY
    )
  {
    let color = first_tag_value(
      &task.tags,
      CAL_COLOR_TAG_KEY
    )
    .map(normalize_marker_color)
    .or_else(|| {
      calendar_colors
        .get(calendar_id)
        .cloned()
    })
    .unwrap_or_else(|| {
      "#d64545".to_string()
    });
    return CalendarTaskMarker {
      shape:
        CalendarMarkerShape::Circle,
      color
    };
  }

  if let Some(board_id) =
    first_tag_value(
      &task.tags,
      BOARD_TAG_KEY
    )
  {
    let color = board_colors
      .get(board_id)
      .cloned()
      .unwrap_or_else(|| {
        default_board_color()
      });
    return CalendarTaskMarker {
      shape:
        CalendarMarkerShape::Triangle,
      color
    };
  }

  CalendarTaskMarker {
    shape: CalendarMarkerShape::Square,
    color: CALENDAR_UNAFFILIATED_COLOR
      .to_string()
  }
}

fn calendar_status_visible(
  status: &TaskStatus,
  visibility: &CalendarVisibility
) -> bool {
  match status {
    | TaskStatus::Pending => {
      visibility.pending
    }
    | TaskStatus::Waiting => {
      visibility.waiting
    }
    | TaskStatus::Completed => {
      visibility.completed
    }
    | TaskStatus::Deleted => {
      visibility.deleted
    }
  }
}

fn parse_taskwarrior_utc(
  raw: &str
) -> Option<DateTime<Utc>> {
  NaiveDateTime::parse_from_str(
    raw,
    "%Y%m%dT%H%M%SZ"
  )
  .ok()
  .map(|naive| {
    DateTime::<Utc>::from_naive_utc_and_offset(
      naive, Utc
    )
  })
}

fn calendar_date_window(
  view: CalendarViewMode,
  focus: NaiveDate,
  week_start: Weekday,
  config: &CalendarConfig
) -> (NaiveDate, NaiveDate) {
  match view {
    | CalendarViewMode::Year => {
      (
        first_day_of_month(
          focus.year(),
          1
        ),
        last_day_of_month(
          focus.year(),
          12
        )
      )
    }
    | CalendarViewMode::Quarter => {
      let quarter_start_month =
        ((focus.month() - 1) / 3) * 3
          + 1;
      let start = first_day_of_month(
        focus.year(),
        quarter_start_month
      );
      let end = last_day_of_month(
        focus.year(),
        quarter_start_month + 2
      );
      (start, end)
    }
    | CalendarViewMode::Month => {
      (
        first_day_of_month(
          focus.year(),
          focus.month()
        ),
        last_day_of_month(
          focus.year(),
          focus.month()
        )
      )
    }
    | CalendarViewMode::Week => {
      let start = start_of_week(
        focus, week_start
      );
      (start, add_days(start, 6))
    }
    | CalendarViewMode::Day => {
      (focus, focus)
    }
    | CalendarViewMode::List => {
      let start = focus;
      let end = add_days(
        start,
        config
          .policies
          .task_list_window_days
          .saturating_sub(1)
      );
      (start, end)
    }
  }
}

fn summarize_calendar_period(
  due_tasks: &[CalendarDueTask],
  view: CalendarViewMode,
  focus: NaiveDate,
  week_start: Weekday,
  config: &CalendarConfig
) -> CalendarStats {
  let (start, end) =
    calendar_date_window(
      view, focus, week_start, config
    );
  let mut stats =
    CalendarStats::default();

  for entry in due_tasks {
    let day =
      entry.due_local.date_naive();
    if day < start || day > end {
      continue;
    }
    stats.push(&entry.task.status);
  }

  stats
}

fn collect_calendar_upcoming_tasks(
  due_tasks: &[CalendarDueTask],
  start_day: NaiveDate,
  config: &CalendarConfig
) -> Vec<CalendarDueTask> {
  let end_day = add_days(
    start_day,
    config
      .policies
      .task_list_window_days
      .saturating_sub(1)
  );

  due_tasks
    .iter()
    .filter(|entry| {
      let day =
        entry.due_local.date_naive();
      day >= start_day && day <= end_day
    })
    .take(
      config.policies.task_list_limit
    )
    .cloned()
    .collect()
}

fn collect_calendar_period_tasks(
  due_tasks: &[CalendarDueTask],
  view: CalendarViewMode,
  focus: NaiveDate,
  week_start: Weekday,
  config: &CalendarConfig
) -> Vec<CalendarDueTask> {
  let (start, end) =
    calendar_date_window(
      view, focus, week_start, config
    );

  due_tasks
    .iter()
    .filter(|entry| {
      let day =
        entry.due_local.date_naive();
      day >= start && day <= end
    })
    .cloned()
    .collect()
}

fn calendar_title_for_view(
  view: CalendarViewMode,
  focus: NaiveDate,
  week_start: Weekday
) -> String {
  match view {
    | CalendarViewMode::Year => {
      format!(
        "Year View {}",
        focus.year()
      )
    }
    | CalendarViewMode::Quarter => {
      let quarter =
        ((focus.month() - 1) / 3) + 1;
      let quarter_start_month =
        ((focus.month() - 1) / 3) * 3
          + 1;
      let start = first_day_of_month(
        focus.year(),
        quarter_start_month
      );
      let end = first_day_of_month(
        focus.year(),
        quarter_start_month + 2
      );
      format!(
        "Quarter View Q{} {} ({}-{})",
        quarter,
        focus.year(),
        start.format("%b"),
        end.format("%b")
      )
    }
    | CalendarViewMode::Month => {
      format!(
        "Month View {}",
        focus.format("%B %Y")
      )
    }
    | CalendarViewMode::Week => {
      let start = start_of_week(
        focus, week_start
      );
      let end = add_days(start, 6);
      format!(
        "Week View {} - {}",
        start.format("%Y-%m-%d"),
        end.format("%Y-%m-%d")
      )
    }
    | CalendarViewMode::Day => {
      format!(
        "Day View {}",
        focus.format("%A, %Y-%m-%d")
      )
    }
    | CalendarViewMode::List => {
      format!(
        "Task List from {}",
        focus.format("%Y-%m-%d")
      )
    }
  }
}

fn render_calendar_view(
  view: CalendarViewMode,
  focus: NaiveDate,
  week_start: Weekday,
  due_tasks: &[CalendarDueTask],
  config: &CalendarConfig,
  tag_colors: &BTreeMap<String, String>,
  on_navigate: Callback<(
    NaiveDate,
    CalendarViewMode
  )>
) -> Html {
  match view {
    | CalendarViewMode::Year => {
      render_calendar_year_view(
        focus,
        due_tasks,
        config,
        on_navigate
      )
    }
    | CalendarViewMode::Quarter => {
      render_calendar_quarter_view(
        focus,
        due_tasks,
        config,
        on_navigate
      )
    }
    | CalendarViewMode::Month => {
      render_calendar_month_view(
        focus,
        week_start,
        due_tasks,
        config,
        on_navigate
      )
    }
    | CalendarViewMode::Week => {
      render_calendar_week_view(
        focus,
        week_start,
        due_tasks,
        config,
        on_navigate
      )
    }
    | CalendarViewMode::Day => {
      render_calendar_day_view(
        focus, due_tasks, config,
        tag_colors
      )
    }
    | CalendarViewMode::List => {
      render_calendar_list_view(
        focus,
        due_tasks,
        config,
        tag_colors,
        on_navigate
      )
    }
  }
}

fn render_calendar_year_view(
  focus: NaiveDate,
  due_tasks: &[CalendarDueTask],
  config: &CalendarConfig,
  on_navigate: Callback<(
    NaiveDate,
    CalendarViewMode
  )>
) -> Html {
  let year = focus.year();

  html! {
      <div class="calendar-grid calendar-year-grid">
          {
              for (1_u32..=12_u32).map(|month| {
                  let month_start = first_day_of_month(year, month);
                  let markers = due_tasks
                      .iter()
                      .filter(|entry| {
                          entry.due_local.year() == year
                              && entry.due_local.month() == month
                      })
                      .map(|entry| entry.marker.clone())
                      .collect::<Vec<_>>();
                  let count = markers.len();
                  let on_navigate = on_navigate.clone();

                  html! {
                      <button
                          type="button"
                          class="calendar-period-card"
                          onclick={Callback::from(move |_| on_navigate.emit((month_start, CalendarViewMode::Month)))}
                      >
                          <div class="calendar-period-title">{ month_start.format("%B").to_string() }</div>
                          <div class="badge">{ format!("{count} tasks") }</div>
                          { render_calendar_markers(&markers, config.policies.red_dot_limit) }
                      </button>
                  }
              })
          }
      </div>
  }
}

fn render_calendar_quarter_view(
  focus: NaiveDate,
  due_tasks: &[CalendarDueTask],
  config: &CalendarConfig,
  on_navigate: Callback<(
    NaiveDate,
    CalendarViewMode
  )>
) -> Html {
  let quarter_start_month =
    ((focus.month() - 1) / 3) * 3 + 1;
  let months = [
    quarter_start_month,
    quarter_start_month + 1,
    quarter_start_month + 2
  ];

  html! {
      <div class="calendar-grid calendar-quarter-grid">
          {
              for months.into_iter().map(|month| {
                  let month_start = first_day_of_month(focus.year(), month);
                  let markers = due_tasks
                      .iter()
                      .filter(|entry| {
                          entry.due_local.year() == focus.year()
                              && entry.due_local.month() == month
                      })
                      .map(|entry| entry.marker.clone())
                      .collect::<Vec<_>>();
                  let count = markers.len();
                  let on_navigate = on_navigate.clone();
                  html! {
                      <button
                          type="button"
                          class="calendar-period-card"
                          onclick={Callback::from(move |_| on_navigate.emit((month_start, CalendarViewMode::Month)))}
                      >
                          <div class="calendar-period-title">{ month_start.format("%B").to_string() }</div>
                          <div class="badge">{ format!("{count} tasks") }</div>
                          { render_calendar_markers(&markers, config.policies.red_dot_limit) }
                      </button>
                  }
              })
          }
      </div>
  }
}

fn render_calendar_month_view(
  focus: NaiveDate,
  week_start: Weekday,
  due_tasks: &[CalendarDueTask],
  config: &CalendarConfig,
  on_navigate: Callback<(
    NaiveDate,
    CalendarViewMode
  )>
) -> Html {
  let first = first_day_of_month(
    focus.year(),
    focus.month()
  );
  let grid_start =
    start_of_week(first, week_start);
  let labels =
    weekday_labels(week_start);
  let month_last = last_day_of_month(
    focus.year(),
    focus.month()
  );
  let mut week_starts = Vec::new();
  for row in 0_i64..6_i64 {
    let week_start_day =
      add_days(grid_start, row * 7);
    let week_end_day =
      add_days(week_start_day, 6);
    if week_end_day < first
      || week_start_day > month_last
    {
      continue;
    }
    week_starts.push(week_start_day);
  }

  html! {
      <>
          <div class="calendar-weekday-row">
              {
                  for labels.into_iter().map(|label| html! {
                      <div class="calendar-weekday">{ label }</div>
                  })
              }
          </div>
          <div class="calendar-grid calendar-month-grid">
              {
                  for (0_i64..42_i64).map(|offset| {
                      let day = add_days(grid_start, offset);
                      let markers = due_tasks
                          .iter()
                          .filter(|entry| entry.due_local.date_naive() == day)
                          .map(|entry| entry.marker.clone())
                          .collect::<Vec<_>>();
                      let count = markers.len();
                      let outside = day.month() != focus.month();
                      let on_navigate = on_navigate.clone();
                      html! {
                          <button
                              type="button"
                              class={classes!("calendar-day-cell", outside.then_some("outside"), (count > 0).then_some("has-tasks"))}
                              onclick={Callback::from(move |_| on_navigate.emit((day, CalendarViewMode::Day)))}
                          >
                              <div class="calendar-day-label">{ day.day() }</div>
                              { render_calendar_markers(&markers, config.policies.red_dot_limit) }
                          </button>
                      }
                  })
              }
          </div>
          <div class="calendar-week-shortcuts">
              {
                  for week_starts.into_iter().map(|week_start_day| {
                      let week_end_day =
                        add_days(
                          week_start_day,
                          6
                        );
                      let label = format!(
                          "{} - {}",
                          week_start_day.format("%b %d"),
                          week_end_day.format("%b %d")
                      );
                      let on_navigate =
                        on_navigate.clone();
                      html! {
                          <button
                              type="button"
                              class="btn calendar-week-shortcut-btn"
                              onclick={Callback::from(move |_| on_navigate.emit((week_start_day, CalendarViewMode::Week)))}
                          >
                              { label }
                          </button>
                      }
                  })
              }
          </div>
      </>
  }
}

fn render_calendar_week_view(
  focus: NaiveDate,
  week_start: Weekday,
  due_tasks: &[CalendarDueTask],
  config: &CalendarConfig,
  on_navigate: Callback<(
    NaiveDate,
    CalendarViewMode
  )>
) -> Html {
  let start =
    start_of_week(focus, week_start);

  html! {
      <div class="calendar-grid calendar-week-grid">
          {
              for (0_i64..7_i64).map(|offset| {
                  let day = add_days(start, offset);
                  let markers = due_tasks
                      .iter()
                      .filter(|entry| entry.due_local.date_naive() == day)
                      .map(|entry| entry.marker.clone())
                      .collect::<Vec<_>>();
                  let day_tasks = due_tasks
                      .iter()
                      .filter(|entry| entry.due_local.date_naive() == day)
                      .cloned()
                      .collect::<Vec<_>>();
                  let count = day_tasks.len();
                  let on_navigate = on_navigate.clone();

                  html! {
                      <button
                          type="button"
                          class={classes!("calendar-week-day-card", (count > 0).then_some("has-tasks"))}
                          onclick={Callback::from(move |_| on_navigate.emit((day, CalendarViewMode::Day)))}
                      >
                          <div class="calendar-week-day-head">
                              <span>{ day.format("%a %d").to_string() }</span>
                              <span class="badge">{ count }</span>
                          </div>
                          { render_calendar_markers(&markers, config.policies.red_dot_limit) }
                          <div class="calendar-week-day-list">
                              {
                                  for day_tasks.iter().take(5).map(|entry| html! {
                                      <div class="calendar-week-task">{ &entry.task.title }</div>
                                  })
                              }
                              {
                                  if count > 5 {
                                      html! { <div class="calendar-week-task muted">{ format!("+{} more", count - 5) }</div> }
                                  } else {
                                      html! {}
                                  }
                              }
                          </div>
                      </button>
                  }
              })
          }
      </div>
  }
}

fn render_calendar_day_view(
  focus: NaiveDate,
  due_tasks: &[CalendarDueTask],
  config: &CalendarConfig,
  tag_colors: &BTreeMap<String, String>
) -> Html {
  let mut tasks = due_tasks
    .iter()
    .filter(|entry| {
      entry.due_local.date_naive()
        == focus
    })
    .cloned()
    .collect::<Vec<_>>();
  tasks
    .sort_by_key(|entry| entry.due_utc);

  let hour_start =
    config.day_view.hour_start;
  let hour_end =
    config.day_view.hour_end;

  html! {
      <div class="calendar-day-view">
          <div class="calendar-day-hours">
              {
                  for (hour_start..=hour_end).map(|hour| {
                      let markers = tasks
                          .iter()
                          .filter(|entry| entry.due_local.hour() == hour)
                          .map(|entry| entry.marker.clone())
                          .collect::<Vec<_>>();
                      html! {
                          <div class="calendar-hour-row">
                              <span class="calendar-hour-label">{ format!("{hour:02}:00") }</span>
                              { render_calendar_markers(&markers, config.policies.red_dot_limit) }
                          </div>
                      }
                  })
              }
          </div>
          <div class="calendar-day-task-list">
              {
                  if tasks.is_empty() {
                      html! { <div class="calendar-empty">{ "No tasks due on this day." }</div> }
                  } else {
                      html! {
                          <>
                              {
                                  for tasks.iter().map(|entry| html! {
                                      <div class="calendar-task-item">
                                          <div class="calendar-task-title">{ &entry.task.title }</div>
                                          <div class="task-subtitle">{ format_calendar_due_datetime(entry, entry.due_local.timezone()) }</div>
                                          <div class="calendar-task-meta">
                                              {
                                                  for entry.task.tags.iter().take(4).map(|tag| html! {
                                                      <span class="badge tag-badge" style={tag_badge_style(tag, tag_colors)}>{ format!("#{tag}") }</span>
                                                  })
                                              }
                                          </div>
                                      </div>
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

fn render_calendar_list_view(
  focus: NaiveDate,
  due_tasks: &[CalendarDueTask],
  config: &CalendarConfig,
  tag_colors: &BTreeMap<String, String>,
  on_navigate: Callback<(
    NaiveDate,
    CalendarViewMode
  )>
) -> Html {
  let tasks =
    collect_calendar_upcoming_tasks(
      due_tasks, focus, config
    );

  html! {
      <div class="calendar-list-view">
          {
              if tasks.is_empty() {
                  html! { <div class="calendar-empty">{ "No upcoming due tasks in configured window." }</div> }
              } else {
                  html! {
                      <>
                          {
                              for tasks.iter().map(|entry| {
                                  let day = entry.due_local.date_naive();
                                  let on_navigate = on_navigate.clone();
                                  html! {
                                      <button
                                          type="button"
                                          class="calendar-list-item"
                                          onclick={Callback::from(move |_| on_navigate.emit((day, CalendarViewMode::Day)))}
                                      >
                                          <div class="calendar-task-title">{ &entry.task.title }</div>
                                          <div class="task-subtitle">{ format_calendar_due_datetime(entry, entry.due_local.timezone()) }</div>
                                          <div class="calendar-task-meta">
                                              {
                                                  for entry.task.tags.iter().take(4).map(|tag| html! {
                                                      <span class="badge tag-badge" style={tag_badge_style(tag, tag_colors)}>{ format!("#{tag}") }</span>
                                                  })
                                              }
                                          </div>
                                      </button>
                                  }
                              })
                          }
                      </>
                  }
              }
          }
      </div>
  }
}

fn weekday_labels(
  week_start: Weekday
) -> Vec<&'static str> {
  match week_start {
    | Weekday::Sun => {
      vec![
        "Sun", "Mon", "Tue", "Wed",
        "Thu", "Fri", "Sat",
      ]
    }
    | _ => {
      vec![
        "Mon", "Tue", "Wed", "Thu",
        "Fri", "Sat", "Sun",
      ]
    }
  }
}

fn render_calendar_markers(
  markers: &[CalendarTaskMarker],
  limit: usize
) -> Html {
  if markers.is_empty() {
    return html! {};
  }

  let capped = markers.len().min(limit);
  let overflow = markers
    .len()
    .saturating_sub(capped);

  html! {
      <div class="calendar-markers">
          {
              for markers.iter().take(capped).map(|marker| {
                  let style = format!("--marker-color:{};", marker.color);
                  html! {
                      <span class={classes!("calendar-marker", marker.shape.as_class())} style={style}></span>
                  }
              })
          }
          {
              if overflow > 0 {
                  html! { <span class="badge">{ format!("+{overflow}") }</span> }
              } else {
                  html! {}
              }
          }
      </div>
  }
}

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

fn filter_visible_tasks(
  tasks: &[TaskDto],
  active_view: &str,
  query: &str,
  active_project: Option<&str>,
  active_tag: Option<&str>,
  all_filter_completion: &str,
  all_filter_project: Option<&str>,
  all_filter_tag: Option<&str>,
  all_filter_priority: &str,
  all_filter_due: &str
) -> Vec<TaskDto> {
  let q = query.to_ascii_lowercase();

  tasks
    .iter()
    .filter(|task| {
      if !q.is_empty() {
        let title_match = task
          .title
          .to_ascii_lowercase()
          .contains(&q);
        let description_match = task
          .description
          .to_ascii_lowercase()
          .contains(&q);
        if !title_match
          && !description_match
        {
          return false;
        }
      }

      match active_view {
        | "projects" => {
          if let Some(project) =
            active_project
          {
            task.project.as_deref()
              == Some(project)
          } else {
            true
          }
        }
        | "tags" => {
          if let Some(tag) = active_tag
          {
            task
              .tags
              .iter()
              .any(|value| value == tag)
          } else {
            true
          }
        }
        | "all" | "kanban" => {
          if let Some(project) =
            all_filter_project
            && task.project.as_deref()
              != Some(project)
          {
            return false;
          }

          if let Some(tag) = all_filter_tag
            && !task
              .tags
              .iter()
              .any(|value| value == tag)
          {
            return false;
          }

          let completion_match =
            match all_filter_completion {
            | "open" => matches!(
              task.status,
              TaskStatus::Pending
                | TaskStatus::Waiting
            ),
            | "pending" => {
              task.status
                == TaskStatus::Pending
            }
            | "waiting" => {
              task.status
                == TaskStatus::Waiting
            }
            | "completed" => {
              task.status
                == TaskStatus::Completed
            }
            | "deleted" => {
              task.status
                == TaskStatus::Deleted
            }
            | _ => true
          };

          let priority_match =
            match all_filter_priority {
            | "low" => task.priority
              == Some(
                rivet_gui_shared::TaskPriority::Low
              ),
            | "medium" => task.priority
              == Some(
                rivet_gui_shared::TaskPriority::Medium
              ),
            | "high" => task.priority
              == Some(
                rivet_gui_shared::TaskPriority::High
              ),
            | "none" => {
              task.priority.is_none()
            }
            | _ => true
          };

          let due_match = match all_filter_due {
            | "has_due" => {
              task.due.is_some()
            }
            | "no_due" => task.due.is_none(),
            | _ => true
          };

          completion_match
            && priority_match
            && due_match
        }
        | _ => true
      }
    })
    .cloned()
    .collect()
}

fn build_project_facets(
  tasks: &[TaskDto]
) -> Vec<(String, usize)> {
  let mut counts = BTreeMap::new();
  for task in tasks {
    if let Some(project) =
      task.project.as_ref()
    {
      *counts
        .entry(project.clone())
        .or_insert(0_usize) += 1;
    }
  }
  counts.into_iter().collect()
}

fn build_tag_facets(
  tasks: &[TaskDto]
) -> Vec<(String, usize)> {
  let mut counts = BTreeMap::new();
  for task in tasks {
    for tag in &task.tags {
      *counts
        .entry(tag.clone())
        .or_insert(0_usize) += 1;
    }
  }
  counts.into_iter().collect()
}

fn ui_debug(
  event: &str,
  detail: &str
) {
  tracing::debug!(
    event, detail, "ui-debug"
  );
  log!(format!(
    "[ui-debug] {event}: {detail}"
  ));
}
