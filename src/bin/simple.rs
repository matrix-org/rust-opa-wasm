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

#![deny(clippy::pedantic)]

use std::collections::HashMap;

use anyhow::Result;
use opa_wasm::Runtime;
use wasmtime::{Config, Engine, Module, Store};

#[tokio::main]
async fn main() -> Result<()> {
    // Configure the WASM runtime
    let mut config = Config::new();
    config.async_support(true);

    let engine = Engine::new(&config)?;

    // Load the policy WASM module
    let module = tokio::fs::read("./policy.wasm").await?;
    let module = Module::new(&engine, module)?;

    // Create a store which will hold the module instance
    let mut store = Store::new(&engine, ());

    let data = HashMap::from([("hello", "world")]);
    let input = HashMap::from([("message", "world")]);

    // Instantiate the module
    let runtime = Runtime::new(&mut store, &module).await?;

    let policy = runtime.with_data(&mut store, &data).await?;

    // Evaluate the policy
    let res: serde_json::Value = policy.evaluate(&mut store, "hello/world", &input).await?;

    println!("{res}");

    Ok(())
}
