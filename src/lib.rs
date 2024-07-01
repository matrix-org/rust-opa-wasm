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

#![doc = include_str!("../README.md")]
#![deny(missing_docs, clippy::pedantic, clippy::missing_docs_in_private_items)]
#![allow(clippy::blocks_in_conditions)]

mod builtins;
mod context;
mod funcs;
#[cfg(feature = "loader")]
mod loader;
mod policy;
mod types;

// Re-export wasmtime to make it easier to keep the verisons in sync
pub use wasmtime;

#[cfg(feature = "loader")]
pub use self::loader::{load_bundle, read_bundle};
pub use self::{
    context::{tests::TestContext, DefaultContext, EvaluationContext},
    policy::{Policy, Runtime},
    types::AbiVersion,
};
