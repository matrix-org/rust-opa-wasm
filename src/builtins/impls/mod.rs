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

// Arguments are passed by value because of the way the builtin trait works
#![allow(clippy::needless_pass_by_value)]

use anyhow::{bail, Result};

pub mod base64url;
pub mod crypto;
pub mod glob;
pub mod graph;
pub mod graphql;
pub mod hex;
pub mod http;
pub mod io;
pub mod json;
pub mod net;
pub mod object;
pub mod opa;
pub mod rand;
pub mod regex;
pub mod rego;
pub mod semver;
pub mod time;
pub mod units;
pub mod urlquery;
pub mod uuid;
pub mod yaml;

/// Returns a list of all the indexes of a substring contained inside a string.
#[tracing::instrument(err)]
pub fn indexof_n(string: String, search: String) -> Result<Vec<u32>> {
    bail!("not implemented");
}

/// Returns a list of all the indexes of a substring contained inside a string.
#[tracing::instrument(err)]
pub fn sprintf(string: String, search: Vec<serde_json::Value>) -> Result<String> {
    bail!("not implemented");
}

/// Emits `note` as a `Note` event in the query explanation. Query explanations show the exact
/// expressions evaluated by OPA during policy execution. For example, `trace("Hello There!")`
/// includes `Note "Hello There!"` in the query explanation. To include variables in the message,
/// use `sprintf`. For example, `person := "Bob"; trace(sprintf("Hello There! %v", [person]))` will
/// emit `Note "Hello There! Bob"` inside of the explanation.
#[tracing::instrument(err)]
pub fn trace(note: String) -> Result<bool> {
    bail!("not implemented");
}
