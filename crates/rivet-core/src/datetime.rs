use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::{
  Context,
  anyhow
};
use chrono::{
  DateTime,
  Duration,
  LocalResult,
  NaiveDate,
  NaiveDateTime,
  TimeZone,
  Utc
};
use chrono_tz::Tz;
use regex::Regex;
use serde::Deserialize;

const TIMEZONE_CONFIG_FILE: &str =
  "rivet-time.toml";
const TIMEZONE_ENV_VAR: &str =
  "RIVET_TIMEZONE";
const TIMEZONE_CONFIG_ENV_VAR: &str =
  "RIVET_TIME_CONFIG";
const DEFAULT_PROJECT_TIMEZONE: &str =
  "America/Mexico_City";

#[derive(Debug, Deserialize)]
struct TimezoneConfig {
  timezone: Option<String>,
  time:     Option<TimezoneSection>
}

#[derive(Debug, Deserialize)]
struct TimezoneSection {
  timezone: Option<String>
}

pub fn project_timezone() -> &'static Tz
{
  static PROJECT_TZ: OnceLock<Tz> =
    OnceLock::new();
  PROJECT_TZ.get_or_init(
    resolve_project_timezone
  )
}

#[must_use]
pub fn to_project_date(
  dt: DateTime<Utc>
) -> chrono::NaiveDate {
  dt.with_timezone(project_timezone())
    .date_naive()
}

#[must_use]
pub fn format_project_date(
  dt: DateTime<Utc>
) -> String {
  dt.with_timezone(project_timezone())
    .format("%Y-%m-%d")
    .to_string()
}

fn resolve_project_timezone() -> Tz {
  if let Ok(raw) =
    std::env::var(TIMEZONE_ENV_VAR)
  {
    if let Some(tz) = parse_timezone(
      &raw,
      TIMEZONE_ENV_VAR
    ) {
      return tz;
    }
  }

  if let Some(path) =
    timezone_config_path()
    && let Some(tz) =
      load_timezone_from_file(&path)
  {
    return tz;
  }

  parse_timezone(
    DEFAULT_PROJECT_TIMEZONE,
    "DEFAULT_PROJECT_TIMEZONE"
  )
  .unwrap_or_else(|| {
    tracing::error!(
      "failed to parse fallback \
       timezone; using UTC"
    );
    chrono_tz::UTC
  })
}

fn timezone_config_path()
-> Option<PathBuf> {
  if let Ok(raw) = std::env::var(
    TIMEZONE_CONFIG_ENV_VAR
  ) {
    let trimmed = raw.trim();
    if !trimmed.is_empty() {
      return Some(PathBuf::from(
        trimmed
      ));
    }
  }

  std::env::current_dir().ok().map(
    |dir| {
      dir.join(TIMEZONE_CONFIG_FILE)
    }
  )
}

fn load_timezone_from_file(
  path: &PathBuf
) -> Option<Tz> {
  if !path.exists() {
    tracing::info!(
      file = %path.display(),
      "timezone config file not found"
    );
    return None;
  }

  let raw = match fs::read_to_string(
    path
  ) {
    | Ok(raw) => raw,
    | Err(err) => {
      tracing::error!(
        file = %path.display(),
        error = %err,
        "failed reading timezone config file"
      );
      return None;
    }
  };

  let parsed = match toml::from_str::<
    TimezoneConfig
  >(&raw)
  {
    | Ok(parsed) => parsed,
    | Err(err) => {
      tracing::error!(
        file = %path.display(),
        error = %err,
        "failed parsing timezone config file"
      );
      return None;
    }
  };

  let timezone =
    parsed.timezone.or_else(|| {
      parsed.time.and_then(|section| {
        section.timezone
      })
    });
  let Some(timezone) = timezone else {
    tracing::warn!(
      file = %path.display(),
      "timezone config had no timezone field"
    );
    return None;
  };

  parse_timezone(
    timezone.as_str(),
    &format!("file:{}", path.display())
  )
}

fn parse_timezone(
  raw: &str,
  source: &str
) -> Option<Tz> {
  let trimmed = raw.trim();
  if trimmed.is_empty() {
    tracing::warn!(
      source,
      "timezone source was empty"
    );
    return None;
  }

  match trimmed.parse::<Tz>() {
    | Ok(tz) => {
      tracing::info!(
        source,
        timezone = %trimmed,
        "configured project timezone"
      );
      Some(tz)
    }
    | Err(err) => {
      tracing::error!(
        source,
        timezone = %trimmed,
        error = %err,
        "failed to parse timezone id"
      );
      None
    }
  }
}

fn to_utc_from_project_local(
  local_naive: NaiveDateTime,
  context: &str
) -> anyhow::Result<DateTime<Utc>> {
  match project_timezone()
    .from_local_datetime(&local_naive)
  {
    | LocalResult::Single(local_dt) => {
      Ok(local_dt.with_timezone(&Utc))
    }
    | LocalResult::Ambiguous(
      first,
      second
    ) => {
      tracing::warn!(
        context,
        first = %first,
        second = %second,
        "ambiguous local datetime; using earliest"
      );
      let chosen = if first <= second {
        first
      } else {
        second
      };
      Ok(chosen.with_timezone(&Utc))
    }
    | LocalResult::None => {
      Err(anyhow!(
        "local datetime does not \
         exist in configured \
         timezone: {context}"
      ))
    }
  }
}

#[tracing::instrument(skip(now), fields(input = input))]
pub fn parse_date_expr(
  input: &str,
  now: DateTime<Utc>
) -> anyhow::Result<DateTime<Utc>> {
  let token = input.trim();
  let lower =
    token.to_ascii_lowercase();

  match lower.as_str() {
    | "now" => return Ok(now),
    | "today" => {
      let local_now = now
        .with_timezone(
          project_timezone()
        );
      let date = local_now.date_naive();
      let midnight = date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| {
          anyhow!(
            "failed to construct \
             midnight for today"
          )
        })?;
      return to_utc_from_project_local(
        midnight, "today"
      );
    }
    | "tomorrow" => {
      let today =
        parse_date_expr("today", now)?;
      return Ok(
        today + Duration::days(1)
      );
    }
    | "yesterday" => {
      let today =
        parse_date_expr("today", now)?;
      return Ok(
        today - Duration::days(1)
      );
    }
    | _ => {}
  }

  let rel_re = Regex::new(r"^(?P<sign>[+-])(?P<num>\d+)(?P<unit>[dhm])$")
        .map_err(|e| anyhow!("internal regex compile failure: {e}"))?;

  if let Some(caps) =
    rel_re.captures(token)
  {
    let sign = caps
      .name("sign")
      .map(|m| m.as_str())
      .ok_or_else(|| {
        anyhow!("missing relative sign")
      })?;
    let num: i64 = caps
      .name("num")
      .map(|m| m.as_str())
      .ok_or_else(|| {
        anyhow!(
          "missing relative amount"
        )
      })?
      .parse()
      .context(
        "invalid relative number"
      )?;
    let unit = caps
      .name("unit")
      .map(|m| m.as_str())
      .ok_or_else(|| {
        anyhow!("missing relative unit")
      })?;

    let duration = match unit {
      | "d" => Duration::days(num),
      | "h" => Duration::hours(num),
      | "m" => Duration::minutes(num),
      | _ => {
        return Err(anyhow!(
          "unknown relative unit: \
           {unit}"
        ))
      }
    };

    return Ok(
      if sign == "-" {
        now - duration
      } else {
        now + duration
      }
    );
  }

  if let Ok(ndt) =
    NaiveDateTime::parse_from_str(
      token,
      "%Y%m%dT%H%M%SZ"
    )
  {
    return Ok(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc));
  }

  if let Ok(dt) =
    DateTime::parse_from_rfc3339(token)
  {
    return Ok(dt.with_timezone(&Utc));
  }

  if let Ok(date) =
    NaiveDate::parse_from_str(
      token, "%Y-%m-%d"
    )
  {
    let midnight = date
      .and_hms_opt(0, 0, 0)
      .ok_or_else(|| {
        anyhow!(
          "failed to construct \
           midnight for date"
        )
      })?;
    return to_utc_from_project_local(
      midnight, "date"
    );
  }

  for fmt in
    ["%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M"]
  {
    if let Ok(ndt) =
      NaiveDateTime::parse_from_str(
        token, fmt
      )
    {
      return to_utc_from_project_local(
        ndt, fmt
      );
    }
  }

  Err(anyhow!(
    "unrecognized date expression: \
     {input}"
  ))
  .with_context(|| {
    "supported formats: \
     now/today/tomorrow/yesterday, \
     +Nd/+Nh/+Nm, RFC3339, YYYY-MM-DD, \
     YYYY-MM-DDTHH:MM, YYYY-MM-DD \
     HH:MM, YYYYMMDDTHHMMSSZ"
  })
}

pub mod taskwarrior_date_serde {
  use chrono::{
    DateTime,
    NaiveDateTime,
    Utc
  };
  use serde::{
    Deserialize,
    Deserializer,
    Serializer
  };

  pub fn serialize<S>(
    dt: &DateTime<Utc>,
    serializer: S
  ) -> Result<S::Ok, S::Error>
  where
    S: Serializer
  {
    serializer.serialize_str(
      &dt
        .format("%Y%m%dT%H%M%SZ")
        .to_string()
    )
  }

  pub fn deserialize<'de, D>(
    deserializer: D
  ) -> Result<DateTime<Utc>, D::Error>
  where
    D: Deserializer<'de>
  {
    let raw = String::deserialize(
      deserializer
    )?;
    NaiveDateTime::parse_from_str(&raw, "%Y%m%dT%H%M%SZ")
            .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            .map_err(serde::de::Error::custom)
  }

  pub mod option {
    use chrono::{
      DateTime,
      NaiveDateTime,
      Utc
    };
    use serde::{
      Deserialize,
      Deserializer,
      Serializer
    };

    pub fn serialize<S>(
      dt: &Option<DateTime<Utc>>,
      serializer: S
    ) -> Result<S::Ok, S::Error>
    where
      S: Serializer
    {
      match dt {
        | Some(value) => {
          super::serialize(
            value, serializer
          )
        }
        | None => {
          serializer.serialize_none()
        }
      }
    }

    pub fn deserialize<'de, D>(
      deserializer: D
    ) -> Result<
      Option<DateTime<Utc>>,
      D::Error
    >
    where
      D: Deserializer<'de>
    {
      let opt =
        Option::<String>::deserialize(
          deserializer
        )?;
      match opt {
                Some(raw) => NaiveDateTime::parse_from_str(&raw, "%Y%m%dT%H%M%SZ")
                    .map(|ndt| Some(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc)))
                    .map_err(serde::de::Error::custom),
                None => Ok(None),
            }
    }
  }
}
