// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Builtins for date and time-related operations

use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, LocalResult, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use chronoutil::RelativeDuration;
use serde::{Deserialize, Serialize};

const ERROR_INVALID_ARGUMENTS: &str = "invalid argument(s)";

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum TimestampWithOptionalTimezone {
    Timestamp(i64),
    TimestampAndTimezone(i64, String),
}

impl TimestampWithOptionalTimezone {
    #[allow(clippy::wrong_self_convention)]
    fn to_datetime(self) -> Result<DateTime<Tz>> {
        match self {
            Self::Timestamp(ts) => nanoseconds_to_date(ts, None),
            Self::TimestampAndTimezone(ts, tz) => nanoseconds_to_date(ts, Some(&tz)),
        }
    }
}

/// Returns the nanoseconds since epoch after adding years, months and days to
/// nanoseconds. `undefined` if the result would be outside the valid time range
/// that can fit within an `int64`.
#[tracing::instrument(name = "time.add_date", err)]
pub fn add_date(ns: i64, years: i32, months: i32, days: i64) -> Result<i64> {
    let date_time = {
        TimestampWithOptionalTimezone::Timestamp(ns).to_datetime()?
            + RelativeDuration::years(years)
            + RelativeDuration::months(months)
            + RelativeDuration::days(days)
    };
    Ok(date_time.timestamp_nanos())
}

/// Returns the `[hour, minute, second]` of the day for the nanoseconds since
/// epoch.
#[tracing::instrument(name = "time.clock", err)]
pub fn clock(x: serde_json::Value) -> Result<(u32, u32, u32)> {
    let date_time = extract_data_from_value(&x)?;
    Ok((date_time.hour(), date_time.minute(), date_time.second()))
}

/// Returns the `[year, month, day]` for the nanoseconds since epoch.
#[tracing::instrument(name = "time.date", err)]
pub fn date(x: serde_json::Value) -> Result<(i32, u32, u32)> {
    let date_time = extract_data_from_value(&x)?;
    Ok((date_time.year(), date_time.month(), date_time.day()))
}

/// Returns the difference between two unix timestamps in nanoseconds (with
/// optional timezone strings).
#[tracing::instrument(name = "time.diff", err)]
// todo:: need to implement
pub fn diff(ns1: serde_json::Value, ns2: serde_json::Value) -> Result<(u8, u8, u8, u8, u8, u8)> {
    Err(anyhow!("not implemented"))
}

/// Returns the current time since epoch in nanoseconds.
#[tracing::instrument(name = "time.now_ns")]
pub fn now_ns() -> i64 {
    let utc: DateTime<Utc> = Utc::now();
    utc.timestamp_nanos()
}

/// Returns the duration in nanoseconds represented by a string.
#[tracing::instrument(name = "time.parse_duration_ns", err)]
pub fn parse_duration_ns(duration: String) -> Result<u128> {
    Ok(duration_str::parse(duration.as_str())?.as_nanos())
}

/// Returns the time in nanoseconds parsed from the string in the given format.
/// `undefined` if the result would be outside the valid time range that can fit
/// within an `int64`.
#[tracing::instrument(name = "time.parse_ns", err)]
pub fn parse_ns(layout: String, value: String) -> Result<i64> {
    Err(anyhow!("not implemented"))
}

/// Returns the time in nanoseconds parsed from the string in RFC3339 format.
/// `undefined` if the result would be outside the valid time range that can fit
/// within an `int64`.
#[tracing::instrument(name = "time.parse_rfc3339_ns", err)]
pub fn parse_rfc3339_ns(value: String) -> Result<i64> {
    Ok(DateTime::parse_from_rfc3339(value.as_ref())?.timestamp_nanos())
}

/// Returns the day of the week (Monday, Tuesday, ...) for the nanoseconds since
/// epoch.
#[tracing::instrument(name = "time.weekday", err)]
pub fn weekday(x: serde_json::Value) -> Result<&'static str> {
    let date_time = extract_data_from_value(&x)?;
    Ok(match date_time.weekday() {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    })
}

/// convert nanoseconds to [`chrono::DateTime`] by the given timezone
///
/// # Errors
///
/// - the `time_zone` when could not parse the given timezone
/// - the given `ns` representation is invalid.
fn nanoseconds_to_date(ns: i64, time_zone: Option<&str>) -> Result<DateTime<Tz>> {
    let tz: Tz = match time_zone.map_or(Ok(Tz::UTC), str::parse) {
        Ok(tz) => tz,
        Err(e) => return Err(anyhow!("timezone parse error: {}", e)),
    };

    #[allow(clippy::cast_sign_loss)]
    match tz.timestamp_opt(ns / 1_000_000_000, (ns % 1_000_000_000) as u32) {
        LocalResult::None => Err(anyhow!("No such local time")),
        LocalResult::Single(t) => Ok(t),
        LocalResult::Ambiguous(t1, t2) => Err(anyhow!(
            "ambiguous local time, ranging from {:?} to {:?}",
            t1,
            t2
        )),
    }
}

/// extract ns and timezone from `serde_json::Value`
///
/// expected from `value` to be an array of i64
/// - when it array, take only the position 0 (ns) and position 1 (timezone) and
///   convert to [`chrono::DateTime`]
/// - when it i64, convert the value (with default timezone UTC) to a
///   [`chrono::DateTime`]
///
/// # Errors
///
/// - the value is empty or has only one field
/// - the single value is not i64
/// - when has an error to convert ns to [`chrono::DateTime`]
fn extract_data_from_value(value: &serde_json::Value) -> Result<DateTime<Tz>> {
    match value.as_array() {
        Some(_data) => match (value.get(0).unwrap_or(value).as_i64(), value.get(1)) {
            (Some(ns), Some(tz)) => TimestampWithOptionalTimezone::TimestampAndTimezone(
                ns,
                tz.as_str().unwrap_or_default().to_string(),
            )
            .to_datetime(),
            _ => Err(anyhow!(ERROR_INVALID_ARGUMENTS)),
        },
        _ => match value.as_i64() {
            Some(ns) => TimestampWithOptionalTimezone::Timestamp(ns).to_datetime(),
            None => Err(anyhow!(ERROR_INVALID_ARGUMENTS)),
        },
    }
}
