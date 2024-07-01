// Copyright 2022-2024 The Matrix.org Foundation C.I.C.
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

//! Builtins related to the current OPA environment

use std::{collections::HashMap, env};

use serde::Serialize;

/// Metadata about the OPA runtime
#[derive(Serialize)]
pub struct Runtime {
    /// A map of the current environment variables
    env: HashMap<String, String>,
    /// The version of OPA runtime. This is currently set to an empty string
    version: String,
    /// The commit hash of the OPA runtime. This is currently set to an empty
    commit: String,
}

/// Returns an object that describes the runtime environment where OPA is
/// deployed.
#[tracing::instrument(name = "opa.runtime")]
pub fn runtime() -> Runtime {
    let env = env::vars().collect();
    Runtime {
        env,
        version: String::new(),
        commit: String::new(),
    }
}
