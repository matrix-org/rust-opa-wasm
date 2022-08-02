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

//! Builtins to parse and convert units

use anyhow::{Context, Result};
use parse_size::Config;
use serde::Serialize;

/// `UsizeOrFloat` is used to give back either type because per Go OPA, `parse`
/// returns usize _or_ float, and this allows to return multiple types.
#[derive(Serialize)]
#[serde(untagged)]
pub enum UsizeOrFloat {
    /// Size
    Usize(u64),
    /// Size (Float)
    Float(f64),
}

/// Converts strings like "10G", "5K", "4M", "1500m" and the like into a number.
/// This number can be a non-integer, such as 1.5, 0.22, etc. Supports standard
/// metric decimal and binary SI units (e.g., K, Ki, M, Mi, G, Gi etc.) m, K, M,
/// G, T, P, and E are treated as decimal units and Ki, Mi, Gi, Ti, Pi, and Ei
/// are treated as binary units.
///
/// Note that 'm' and 'M' are case-sensitive, to allow distinguishing between
/// "milli" and "mega" units respectively. Other units are case-insensitive.
#[allow(clippy::cast_precision_loss)]
#[tracing::instrument(name = "units.parse", err)]
pub fn parse(x: String) -> Result<UsizeOrFloat> {
    let p = Config::new().with_decimal();
    // edge case here, when 'm' is lowercase that's mili
    if let [init @ .., b'm'] = x.as_bytes() {
        return Ok(UsizeOrFloat::Float(p.parse_size(init)? as f64 * 0.001));
    }
    Ok(UsizeOrFloat::Usize(p.parse_size(x.as_str())?))
}

/// Converts strings like "10GB", "5K", "4mb" into an integer number of bytes.
/// Supports standard byte units (e.g., KB, KiB, etc.) KB, MB, GB, and TB are
/// treated as decimal units and KiB, MiB, GiB, and TiB are treated as binary
/// units. The bytes symbol (b/B) in the unit is optional and omitting it wil
/// give the same result (e.g. Mi and MiB).
#[tracing::instrument(name = "units.parse_bytes", err)]
pub fn parse_bytes(x: String) -> Result<u64> {
    Config::new()
        .with_decimal()
        .parse_size(x.as_str())
        .context("could not parse value")
}
