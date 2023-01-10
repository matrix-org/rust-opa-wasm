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
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use chronoutil::RelativeDuration;
use serde::{Deserialize, Serialize};

use crate::EvaluationContext;

/// A type which olds either a timestamp (in nanoseconds) or a timestamp and a
/// timezone string
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum TimestampWithOptionalTimezone {
    /// Holds a timestamp
    Timestamp(i64),

    /// Holds a timestamp and a timezone
    TimestampAndTimezone(i64, String),
}

impl TimestampWithOptionalTimezone {
    fn into_datetime(self) -> Result<DateTime<Tz>> {
        let (ts, tz) = match self {
            Self::Timestamp(ts) => (ts, Tz::UTC),
            Self::TimestampAndTimezone(ts, tz) => (
                ts,
                tz.parse()
                    .map_err(|e| anyhow!("Could not parse timezone: {}", e))?,
            ),
        };

        Ok(tz.timestamp_nanos(ts))
    }
}

/// Returns the nanoseconds since epoch after adding years, months and days to
/// nanoseconds. `undefined` if the result would be outside the valid time range
/// that can fit within an `int64`.
#[tracing::instrument(name = "time.add_date", err)]
pub fn add_date(ns: i64, years: i32, months: i32, days: i64) -> Result<i64> {
    let date_time = {
        Utc.timestamp_nanos(ns)
            + RelativeDuration::years(years)
            + RelativeDuration::months(months)
            + RelativeDuration::days(days)
    };
    Ok(date_time.timestamp_nanos())
}

/// Returns the `[hour, minute, second]` of the day for the nanoseconds since
/// epoch.
#[tracing::instrument(name = "time.clock", err)]
pub fn clock(x: TimestampWithOptionalTimezone) -> Result<(u32, u32, u32)> {
    let date_time = x.into_datetime()?;
    Ok((date_time.hour(), date_time.minute(), date_time.second()))
}

/// Returns the `[year, month, day]` for the nanoseconds since epoch.
#[tracing::instrument(name = "time.date", err)]
pub fn date(x: TimestampWithOptionalTimezone) -> Result<(i32, u32, u32)> {
    let date_time = x.into_datetime()?;
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
#[tracing::instrument(name = "time.now_ns", skip(ctx))]
pub fn now_ns<C: EvaluationContext>(ctx: &mut C) -> i64 {
    ctx.now().timestamp_nanos()
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
pub fn weekday(x: TimestampWithOptionalTimezone) -> Result<&'static str> {
    let date_time = x.into_datetime()?;
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
