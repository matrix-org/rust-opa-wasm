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

//! Builtins to encode and decode URL-encoded strings

use std::collections::HashMap;

use serde_json::Value;
use url::form_urlencoded::{byte_serialize, parse};

const QUERY_CONCAT_CHAR: &str = "&";

/// Decodes a URL-encoded input string.
#[tracing::instrument(name = "urlquery.decode")]
pub fn decode(x: String) -> String {
    parse(x.as_bytes())
        .map(|(key, val)| [key, val].concat())
        .collect()
}

/// Decodes the given URL query string into an object.
#[tracing::instrument(name = "urlquery.decode_object")]
pub fn decode_object(x: String) -> HashMap<String, Vec<String>> {
    let mut decoded_object: HashMap<String, Vec<String>> = HashMap::new();
    for pair in x.split(QUERY_CONCAT_CHAR) {
        let mut iter_parameter = pair.split('=').take(2);
        let (parameter_key, parameter_value) = match (iter_parameter.next(), iter_parameter.next())
        {
            (Some(k), Some(v)) => (k, v),
            _ => continue,
        };
        match decoded_object.get_mut(parameter_key) {
            Some(v) => v.push(parameter_value.to_string()),
            _ => {
                decoded_object.insert(parameter_key.to_string(), vec![parameter_value.to_string()]);
            }
        };
    }
    decoded_object
}

/// Encodes the input string into a URL-encoded string.
#[tracing::instrument(name = "urlquery.encode")]
pub fn encode(x: String) -> String {
    byte_serialize(x.as_bytes()).collect()
}

/// Encodes the given object into a URL encoded query string.
#[tracing::instrument(name = "urlquery.encode_object")]
pub fn encode_object(x: HashMap<String, serde_json::Value>) -> String {
    let mut encoded_object: Vec<String> = Vec::new();
    x.iter().for_each(
        |(parameter_key, parameter_value)| match &parameter_value {
            Value::Array(arr) => {
                arr
                    .iter()
                    .for_each(|v| {
                        encoded_object.push(concat_encode_query(parameter_key, v.as_str().unwrap_or("")));
                    });
            }
            _ => {
                encoded_object.push(concat_encode_query(parameter_key, parameter_value.as_str().unwrap_or("")));
            }
        },
    );
   encoded_object.sort();
   encoded_object.join(QUERY_CONCAT_CHAR)
}

fn concat_encode_query(key: &str, value: &str) -> String {
    format!("{}={}", key, value)
}
