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

use std::collections::BTreeMap;

use anyhow::Result;
use serde::Deserialize;

/// A wrapper type which can deserialize either one value or an array of values
#[derive(Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    /// Represents only one value
    One(T),

    /// Represents an array of values
    Many(Vec<T>),
}

impl<T: std::fmt::Debug> std::fmt::Debug for OneOrMany<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::One(t) => t.fmt(f),
            Self::Many(v) => v.fmt(f),
        }
    }
}

/// Decodes a URL-encoded input string.
#[tracing::instrument(name = "urlquery.decode", err)]
pub fn decode(x: String) -> Result<String> {
    Ok(urlencoding::decode(&x)?.into_owned())
}

/// Decodes the given URL query string into an object.
#[tracing::instrument(name = "urlquery.decode_object")]
pub fn decode_object(x: String) -> BTreeMap<String, Vec<String>> {
    let parsed = form_urlencoded::parse(x.as_bytes()).into_owned();
    let mut decoded_object: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (k, v) in parsed {
        decoded_object.entry(k).or_default().push(v);
    }
    decoded_object
}

/// Encodes the input string into a URL-encoded string.
#[tracing::instrument(name = "urlquery.encode")]
pub fn encode(x: String) -> String {
    form_urlencoded::byte_serialize(x.as_bytes()).collect()
}

/// Encodes the given object into a URL encoded query string.
#[tracing::instrument(name = "urlquery.encode_object")]
pub fn encode_object(x: BTreeMap<String, OneOrMany<String>>) -> String {
    let mut encoded = form_urlencoded::Serializer::new(String::new());

    for (key, value) in x {
        match value {
            OneOrMany::One(value) => {
                encoded.append_pair(&key, &value);
            }
            OneOrMany::Many(values) => {
                for value in values {
                    encoded.append_pair(&key, &value);
                }
            }
        }
    }

    encoded.finish()
}
