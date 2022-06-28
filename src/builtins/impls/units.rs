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

use anyhow::{bail, Result};

/// Converts strings like "10G", "5K", "4M", "1500m" and the like into a number.
/// This number can be a non-integer, such as 1.5, 0.22, etc. Supports standard
/// metric decimal and binary SI units (e.g., K, Ki, M, Mi, G, Gi etc.) m, K, M,
/// G, T, P, and E are treated as decimal units and Ki, Mi, Gi, Ti, Pi, and Ei
/// are treated as binary units.
///
/// Note that 'm' and 'M' are case-sensitive, to allow distinguishing between
/// "milli" and "mega" units respectively. Other units are case-insensitive.
#[tracing::instrument(name = "units.parse", err)]
pub fn parse(x: String) -> Result<i64> {
    bail!("not implemented");
}

/// Converts strings like "10GB", "5K", "4mb" into an integer number of bytes.
/// Supports standard byte units (e.g., KB, KiB, etc.) KB, MB, GB, and TB are
/// treated as decimal units and KiB, MiB, GiB, and TiB are treated as binary
/// units. The bytes symbol (b/B) in the unit is optional and omitting it wil
/// give the same result (e.g. Mi and MiB).
#[tracing::instrument(name = "units.parse_bytes", err)]
pub fn parse_bytes(x: String) -> Result<i64> {
    bail!("not implemented");
}
