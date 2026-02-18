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
  MouseEvent,
  Properties,
  TargetCast,
  classes,
  function_component,
  html,
  use_effect_with,
  use_state,
  UseStateHandle
};

use crate::api::invoke_tauri;
use crate::components::{
  Details,
  FacetPanel,
  KanbanBoard,
  Sidebar,
  TaskList,
  WindowChrome,
  WorkspaceTabs
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
  allow_recurrence:     bool,
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
  #[serde(default)]
  imported_ics_file: bool,
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

#[derive(
  Clone, PartialEq, Serialize,
)]
struct ExternalCalendarImportArgs {
  source:   ExternalCalendarSource,
  ics_text: String
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

#[derive(Clone, PartialEq)]
struct ExternalCalendarDeleteState {
  id:   String,
  name: String
}

#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Serialize,
  Deserialize,
)]
enum DueNotificationPermission {
  Default,
  Granted,
  Denied,
  Unsupported
}

impl DueNotificationPermission {
  fn as_label(
    self
  ) -> &'static str {
    match self {
      | Self::Default => {
        "Permission not requested"
      }
      | Self::Granted => {
        "Permission granted"
      }
      | Self::Denied => {
        "Permission denied"
      }
      | Self::Unsupported => {
        "Notifications unsupported"
      }
    }
  }
}

#[derive(
  Clone,
  PartialEq,
  Eq,
  Serialize,
  Deserialize,
)]
struct DueNotificationConfig {
  enabled:            bool,
  pre_notify_enabled: bool,
  pre_notify_minutes: u32
}

impl Default for DueNotificationConfig {
  fn default() -> Self {
    Self {
      enabled:            false,
      pre_notify_enabled: false,
      pre_notify_minutes: 15
    }
  }
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
  Day
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
      | Self::Day => "day"
    }
  }

  fn label(self) -> &'static str {
    match self {
      | Self::Year => "Year",
      | Self::Quarter => "Quarter",
      | Self::Month => "Month",
      | Self::Week => "Week",
      | Self::Day => "Day"
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
      | "list" => Some(Self::Month),
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

#[derive(Clone, PartialEq)]
struct CalendarDueTask {
  task:      TaskDto,
  due_utc:   DateTime<Utc>,
  due_local: DateTime<Tz>,
  marker:    CalendarTaskMarker
}

#[derive(
  Clone, Copy, PartialEq, Eq,
)]
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

#[derive(Clone, PartialEq)]
struct CalendarTaskMarker {
  shape: CalendarMarkerShape,
  color: String
}

#[derive(
  Clone, Default, PartialEq,
)]
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
const DUE_NOTIFICATION_SETTINGS_STORAGE_KEY:
  &str =
  "rivet.notifications.due.settings";
const DUE_NOTIFICATION_SENT_STORAGE_KEY:
  &str = "rivet.notifications.due.sent";
const KANBAN_BOARDS_STORAGE_KEY: &str =
  "rivet.kanban.boards";
const KANBAN_ACTIVE_BOARD_STORAGE_KEY:
  &str = "rivet.kanban.active_board";
const TAG_SCHEMA_TOML: &str =
  include_str!(
    "../../assets/tags.toml"
  );
const CALENDAR_CONFIG_TOML: &str = include_str!(
  "../../assets/calendar.toml"
);
const PROJECT_TIME_CONFIG_TOML: &str = include_str!(
  "../../../../../rivet-time.toml"
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
