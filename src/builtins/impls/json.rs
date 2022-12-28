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

//! Builtins related to JSON objects handling

use json_patch::Patch;

/// Patches an object according to RFC6902.
/// For example: `json.patch({"a": {"foo": 1}}, [{"op": "add", "path": "/a/bar",
/// "value": 2}])` results in `{"a": {"foo": 1, "bar": 2}`. The patches are
/// applied atomically: if any of them fails, the result will be undefined.
#[tracing::instrument(name = "json.patch")]
pub fn patch(mut object: serde_json::Value, patch: Patch) -> serde_json::Value {
    if json_patch::patch(&mut object, &patch).is_err() {
        serde_json::Value::Object(serde_json::Map::default())
    } else {
        object
    }
}
