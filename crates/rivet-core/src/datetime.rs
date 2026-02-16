use anyhow::{Context, anyhow};
use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use regex::Regex;

#[tracing::instrument(skip(now), fields(input = input))]
pub fn parse_date_expr(input: &str, now: DateTime<Utc>) -> anyhow::Result<DateTime<Utc>> {
    let token = input.trim();
    let lower = token.to_ascii_lowercase();

    match lower.as_str() {
        "now" => return Ok(now),
        "today" => {
            let local_now = now.with_timezone(&Local);
            let date = local_now.date_naive();
            let midnight = date
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow!("failed to construct midnight for today"))?;
            let local_dt = Local
                .from_local_datetime(&midnight)
                .single()
                .ok_or_else(|| anyhow!("ambiguous local datetime for today"))?;
            return Ok(local_dt.with_timezone(&Utc));
        }
        "tomorrow" => {
            let today = parse_date_expr("today", now)?;
            return Ok(today + Duration::days(1));
        }
        "yesterday" => {
            let today = parse_date_expr("today", now)?;
            return Ok(today - Duration::days(1));
        }
        _ => {}
    }

    let rel_re = Regex::new(r"^(?P<sign>[+-])(?P<num>\d+)(?P<unit>[dhm])$")
        .map_err(|e| anyhow!("internal regex compile failure: {e}"))?;

    if let Some(caps) = rel_re.captures(token) {
        let sign = caps
            .name("sign")
            .map(|m| m.as_str())
            .ok_or_else(|| anyhow!("missing relative sign"))?;
        let num: i64 = caps
            .name("num")
            .map(|m| m.as_str())
            .ok_or_else(|| anyhow!("missing relative amount"))?
            .parse()
            .context("invalid relative number")?;
        let unit = caps
            .name("unit")
            .map(|m| m.as_str())
            .ok_or_else(|| anyhow!("missing relative unit"))?;

        let duration = match unit {
            "d" => Duration::days(num),
            "h" => Duration::hours(num),
            "m" => Duration::minutes(num),
            _ => return Err(anyhow!("unknown relative unit: {unit}")),
        };

        return Ok(if sign == "-" {
            now - duration
        } else {
            now + duration
        });
    }

    if let Ok(ndt) = NaiveDateTime::parse_from_str(token, "%Y%m%dT%H%M%SZ") {
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc));
    }

    if let Ok(dt) = DateTime::parse_from_rfc3339(token) {
        return Ok(dt.with_timezone(&Utc));
    }

    if let Ok(date) = NaiveDate::parse_from_str(token, "%Y-%m-%d") {
        let midnight = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| anyhow!("failed to construct midnight for date"))?;
        let local_dt = Local
            .from_local_datetime(&midnight)
            .single()
            .ok_or_else(|| anyhow!("ambiguous local datetime for date"))?;
        return Ok(local_dt.with_timezone(&Utc));
    }

    for fmt in ["%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M"] {
        if let Ok(ndt) = NaiveDateTime::parse_from_str(token, fmt) {
            let local_dt = Local
                .from_local_datetime(&ndt)
                .single()
                .ok_or_else(|| anyhow!("ambiguous local datetime"))?;
            return Ok(local_dt.with_timezone(&Utc));
        }
    }

    Err(anyhow!("unrecognized date expression: {input}")).with_context(
        || {
            "supported formats: now/today/tomorrow/yesterday, +Nd/+Nh/+Nm, RFC3339, YYYY-MM-DD, YYYY-MM-DDTHH:MM, YYYY-MM-DD HH:MM, YYYYMMDDTHHMMSSZ"
        },
    )
}

pub mod taskwarrior_date_serde {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&dt.format("%Y%m%dT%H%M%SZ").to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&raw, "%Y%m%dT%H%M%SZ")
            .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            .map_err(serde::de::Error::custom)
    }

    pub mod option {
        use chrono::{DateTime, NaiveDateTime, Utc};
        use serde::{Deserialize, Deserializer, Serializer};

        pub fn serialize<S>(dt: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match dt {
                Some(value) => super::serialize(value, serializer),
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let opt = Option::<String>::deserialize(deserializer)?;
            match opt {
                Some(raw) => NaiveDateTime::parse_from_str(&raw, "%Y%m%dT%H%M%SZ")
                    .map(|ndt| Some(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc)))
                    .map_err(serde::de::Error::custom),
                None => Ok(None),
            }
        }
    }
}
