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

use anyhow::{bail, Result};

/// Returns the nanoseconds since epoch after adding years, months and days to nanoseconds.
/// `undefined` if the result would be outside the valid time range that can fit within an `int64`.
#[tracing::instrument(name = "time.add_date", err)]
pub fn add_date(ns: i64, years: i64, months: i64, number: i64) -> Result<serde_json::Value> {
    bail!("not implemented");
}

/// Returns the `[hour, minute, second]` of the day for the nanoseconds since epoch.
#[tracing::instrument(name = "time.clock", err)]
pub fn clock(x: serde_json::Value) -> Result<(u8, u8, u8)> {
    bail!("not implemented");
}

/// Returns the `[year, month, day]` for the nanoseconds since epoch.
#[tracing::instrument(name = "time.date", err)]
pub fn date(x: serde_json::Value) -> Result<(u8, u8, u8)> {
    bail!("not implemented");
}

/// Returns the difference between two unix timestamps in nanoseconds (with optional timezone strings).
#[tracing::instrument(name = "time.diff", err)]
pub fn diff(ns1: serde_json::Value, ns2: serde_json::Value) -> Result<(u8, u8, u8, u8, u8, u8)> {
    bail!("not implemented");
}

/// Returns the current time since epoch in nanoseconds.
#[tracing::instrument(name = "time.now_ns", err)]
pub fn now_ns() -> Result<i64> {
    bail!("not implemented");
}

/// Returns the duration in nanoseconds represented by a string.
#[tracing::instrument(name = "time.parse_duration_ns", err)]
pub fn parse_duration_ns(duration: String) -> Result<i64> {
    bail!("not implemented");
}

/// Returns the time in nanoseconds parsed from the string in the given format. `undefined` if the
/// result would be outside the valid time range that can fit within an `int64`.
#[tracing::instrument(name = "time.parse_ns", err)]
pub fn parse_ns(layout: String, value: String) -> Result<i64> {
    bail!("not implemented");
}

/// Returns the time in nanoseconds parsed from the string in RFC3339 format. `undefined` if the
/// result would be outside the valid time range that can fit within an `int64`.
#[tracing::instrument(name = "time.parse_rfc3339_ns", err)]
pub fn parse_rfc3339_ns(value: String) -> Result<i64> {
    bail!("not implemented");
}

/// Returns the day of the week (Monday, Tuesday, ...) for the nanoseconds since epoch.
#[tracing::instrument(name = "time.weekday", err)]
pub fn weekday(x: serde_json::Value) -> Result<String> {
    bail!("not implemented");
}
