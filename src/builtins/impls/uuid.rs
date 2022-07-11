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

//! Builtins to generate UUIDs
use anyhow::{bail, Result};

/// Returns a new UUIDv4.
#[tracing::instrument(name = "uuid.rfc4122", err)]
pub fn rfc4122(k: String) -> Result<String> {
    // note: the semantics required here is to generate a UUID that is similar *for
    // the duration of the query for every k* the Go implementation uses a
    // global builtin cache so that UUIDs per `k` are stored through a life of a
    // query.
    bail!("not implemented")
}
