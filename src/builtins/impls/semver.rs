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

//! Builtins related to semver version validation and comparison

use std::cmp::Ordering;

use anyhow::Result;
use semver::Version;

/// Compares valid SemVer formatted version strings.
#[tracing::instrument(name = "semver.compare", err)]
pub fn compare(a: String, b: String) -> Result<i8> {
    let a = Version::parse(&a)?;
    let b = Version::parse(&b)?;
    match a.cmp(&b) {
        Ordering::Less => Ok(-1),
        Ordering::Equal => Ok(0),
        Ordering::Greater => Ok(1),
    }
}

/// Validates that the input is a valid SemVer string.
#[tracing::instrument(name = "semver.is_valid")]
pub fn is_valid(vsn: String) -> bool {
    Version::parse(&vsn).is_ok()
}
