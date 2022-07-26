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

//! Builtins parse and serialize YAML documents

use anyhow::Result;
use serde_yaml;

/// Verifies the input string is a valid YAML document.
#[tracing::instrument(name = "yaml.is_valid")]
pub fn is_valid(x: String) -> bool {
    let parse: Result<serde_yaml::Value, _> = serde_yaml::from_str(&x);
    parse.is_ok()
}

/// Serializes the input term to YAML.
#[tracing::instrument(name = "yaml.marshal", err)]
pub fn marshal(x: serde_yaml::Value) -> Result<String> {
    let parse: String = serde_yaml::to_string(&x)?;
    Ok(parse)
}

/// Deserializes the input string.
#[tracing::instrument(name = "yaml.unmarshal", err)]
pub fn unmarshal(x: String) -> Result<serde_json::Value> {
    let parse: serde_json::Value = serde_yaml::from_str(&x)?;
    Ok(parse)
}
