use std::collections::{
  BTreeMap,
  BTreeSet
};
use std::io::BufReader;
use std::time::Duration;

use anyhow::Context;
use chrono::{
  DateTime,
  Datelike,
  LocalResult,
  NaiveDate,
  NaiveDateTime,
  TimeZone,
  Utc
};
use chrono_tz::Tz;
use ical::IcalParser;
use ical::parser::ical::component::IcalEvent;
use ical::property::Property;
use rivet_core::datetime::project_timezone;
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
use tauri::State;
use tracing::{
  error,
  info,
  instrument,
  warn
};

use crate::state::AppState;

const CAL_SOURCE_TAG_KEY: &str =
  "cal_source";
const CAL_EVENT_TAG_KEY: &str =
  "cal_event";
const CAL_COLOR_TAG_KEY: &str =
  "cal_color";
const RECUR_TAG_KEY: &str = "recur";
const RECUR_TIME_TAG_KEY: &str =
  "recur_time";
const RECUR_DAYS_TAG_KEY: &str =
  "recur_days";
const RECUR_MONTHS_TAG_KEY: &str =
  "recur_months";
const RECUR_MONTH_DAY_TAG_KEY: &str =
  "recur_day";

fn err_to_string(
  err: anyhow::Error
) -> String {
  err.to_string()
}

