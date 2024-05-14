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

//! Builtins to help handling JSON objects

use anyhow::Result;
use serde_json::Value;

/// Creates a new object that is the asymmetric union of all objects merged from
/// left to right. For example: `object.union_n([{"a": 1}, {"b": 2}, {"a": 3}])`
/// will result in `{"b": 2, "a": 3}`.
#[tracing::instrument(name = "object.union_n", err)]
pub fn union_n(objects: Vec<Value>) -> Result<Value> {
    let mut result = serde_json::Value::Object(serde_json::Map::default());
    for object in objects {
        merge_value(&mut result, &object);
    }

    Ok(result)
}

fn merge_value(a: &mut Value, b: &Value) {
    match (a, b) {
        (Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge_value(a.entry(k).or_insert(Value::Null), v);
            }
        }
        (Value::Array(ref mut a), &Value::Array(ref b)) => {
            *a = vec![];
            a.extend(b.clone());
        }
        (Value::Array(ref mut a), &Value::Object(ref b)) => {
            *a = vec![];
            a.extend([Value::Object(b.clone())]);
        }
        (_, Value::Null) => {}
        (a, b) => {
            *a = b.clone();
        }
    }
}
