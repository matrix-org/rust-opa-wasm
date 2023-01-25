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

//! Implementations of all SDK-dependant builtin functions

// Arguments are passed by value because of the way the builtin trait works
#![allow(clippy::needless_pass_by_value)]

use anyhow::{bail, Result};

#[cfg(feature = "base64url-builtins")]
pub mod base64url;
pub mod crypto;
#[cfg(feature = "glob-builtins")]
pub mod glob;

pub mod graph;
pub mod graphql;
#[cfg(feature = "hex-builtins")]
pub mod hex;
pub mod http;
pub mod io;
#[cfg(feature = "json-builtins")]
pub mod json;
pub mod net;
#[cfg(feature = "object-builtins")]
pub mod object;
pub mod opa;
#[cfg(feature = "rng")]
pub mod rand;
#[cfg(feature = "regex-builtins")]
pub mod regex;

pub mod rego;
#[cfg(feature = "semver-builtins")]
pub mod semver;
#[cfg(feature = "time-builtins")]
pub mod time;
#[cfg(feature = "units-builtins")]
pub mod units;
#[cfg(feature = "urlquery-builtins")]
pub mod urlquery;
pub mod uuid;
#[cfg(feature = "yaml-builtins")]
pub mod yaml;

/// Returns a list of all the indexes of a substring contained inside a string.
#[tracing::instrument(err)]
pub fn indexof_n(string: String, search: String) -> Result<Vec<u32>> {
    bail!("not implemented");
}

#[cfg(feature = "sprintf-builtins")]
/// Returns the given string, formatted.
#[tracing::instrument(err)]
pub fn sprintf(format: String, values: Vec<serde_json::Value>) -> Result<String> {
    use sprintf::{vsprintf, Printf};

    let orig_values = values.clone();

    let values: Result<Vec<Box<dyn Printf>>, _> = values
        .into_iter()
        .map(|v| -> Result<Box<dyn Printf>, _> {
            match v {
                serde_json::Value::Null => Err(anyhow::anyhow!("can't format null")),
                serde_json::Value::Bool(_) => Err(anyhow::anyhow!("can't format a boolean")),
                serde_json::Value::Number(n) => {
                    if let Some(n) = n.as_u64() {
                        Ok(Box::new(n))
                    } else if let Some(n) = n.as_i64() {
                        Ok(Box::new(n))
                    } else if let Some(n) = n.as_f64() {
                        Ok(Box::new(n))
                    } else {
                        Err(anyhow::anyhow!("unreachable"))
                    }
                }
                serde_json::Value::String(s) => Ok(Box::new(s)),
                serde_json::Value::Array(_) => Err(anyhow::anyhow!("can't format array")),
                serde_json::Value::Object(_) => Err(anyhow::anyhow!("can't format object")),
            }
        })
        .collect();
    let values = values?;
    let values: Vec<&dyn Printf> = values.iter().map(std::convert::AsRef::as_ref).collect();

    match vsprintf(&format, &values) {
        Ok(str) => Ok(str),
        Err(err) => handle_errors_like_go(err, format, orig_values),
    }
}

fn handle_errors_like_go(
    err: sprintf::PrintfError,
    format: String,
    values: Vec<serde_json::Value>,
) -> Result<String> {
    match err {
        sprintf::PrintfError::WrongType
        | sprintf::PrintfError::TooManyArgs
        | sprintf::PrintfError::NotEnoughArgs => Ok(format!("{format}, ({err:?}, {values:?})")),
        _ => anyhow::bail!("failed to call printf"),
    }
}

/// Emits `note` as a `Note` event in the query explanation. Query explanations
/// show the exact expressions evaluated by OPA during policy execution. For
/// example, `trace("Hello There!")` includes `Note "Hello There!"` in the query
/// explanation. To include variables in the message, use `sprintf`. For
/// example, `person := "Bob"; trace(sprintf("Hello There! %v", [person]))` will
/// emit `Note "Hello There! Bob"` inside of the explanation.
#[tracing::instrument(err)]
pub fn trace(note: String) -> Result<bool> {
    bail!("not implemented");
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn can_sprintf() {
        let val_1 = serde_json::Value::String("first".to_string());
        let val_2 = serde_json::Value::String("sec".to_string());
        assert_debug_snapshot!(sprintf("number %s".to_string(), vec![val_1.clone()]));
        assert_debug_snapshot!(sprintf("number %s".to_string(), vec![val_1.clone(), val_2]));
        assert_debug_snapshot!(sprintf("number %d".to_string(), vec![val_1.clone()]));
        assert_debug_snapshot!(sprintf("number".to_string(), vec![val_1]));
    }
}
