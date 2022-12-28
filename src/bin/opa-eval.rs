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

use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{ArgGroup, Parser};
use opa_wasm::Runtime;
use tracing::Instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use wasmtime::{Config, Engine, Module, Store};

/// Evaluates OPA policies compiled as WASM modules
#[derive(Parser)]
#[clap(group(
    ArgGroup::new("policy")
        .required(true)
))]
struct Cli {
    /// Path to the WASM module
    #[arg(short, long, group = "policy")]
    module: Option<Utf8PathBuf>,

    /// Path to the OPA bundle
    #[arg(short, long, group = "policy")]
    bundle: Option<Utf8PathBuf>,

    /// Entrypoint to use
    #[arg(short, long)]
    entrypoint: String,

    /// JSON literal to use as data
    #[arg(short, long = "data", group = "data", value_name = "JSON")]
    data_value: Option<serde_json::Value>,

    /// Path to a JSON file to load as data
    #[arg(short = 'D', long, group = "data", value_name = "PATH")]
    data_path: Option<Utf8PathBuf>,

    /// JSON literal to use as input
    #[arg(short, long = "input", group = "input", value_name = "JSON")]
    input_value: Option<serde_json::Value>,

    /// Path to a JSON file to load as input
    #[arg(short = 'I', long, group = "input", value_name = "PATH")]
    input_path: Option<Utf8PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    Registry::default()
        .with(tracing_forest::ForestLayer::default())
        .with(EnvFilter::from_default_env())
        .init();

    let (data, input, module, entrypoint) = (async move {
        let cli = Cli::parse();

        let data = if let Some(path) = cli.data_path {
            let content = tokio::fs::read(path).await?;
            serde_json::from_slice(&content)?
        } else if let Some(data) = cli.data_value {
            data
        } else {
            serde_json::Value::Object(serde_json::Map::default())
        };

        let input = if let Some(path) = cli.input_path {
            let content = tokio::fs::read(path).await?;
            serde_json::from_slice(&content)?
        } else if let Some(input) = cli.input_value {
            input
        } else {
            serde_json::Value::Object(serde_json::Map::default())
        };

        let module = if let Some(path) = cli.module {
            tokio::fs::read(path)
                .instrument(tracing::info_span!("read_module"))
                .await?
        } else if let Some(path) = cli.bundle {
            opa_wasm::read_bundle(path).await?
        } else {
            // This should be enforced by clap
            unreachable!()
        };

        let entrypoint = cli.entrypoint;
        Ok::<_, anyhow::Error>((data, input, module, entrypoint))
    })
    .instrument(tracing::info_span!("load_args"))
    .await?;

    let (mut store, module) = (async move {
        // Configure the WASM runtime
        let mut config = Config::new();
        config.async_support(true);

        let engine = Engine::new(&config)?;

        // Load the policy WASM module
        let module = Module::new(&engine, module)?;

        // Create a store which will hold the module instance
        let store = Store::new(&engine, ());
        Ok::<_, anyhow::Error>((store, module))
    })
    .instrument(tracing::info_span!("compile_module"))
    .await?;

    // Instantiate the module
    let runtime = Runtime::new(&mut store, &module)
        .instrument(tracing::info_span!("instanciate_module"))
        .await?;

    let policy = runtime
        .with_data(&mut store, &data)
        .instrument(tracing::info_span!("load_data"))
        .await?;

    // Evaluate the policy
    let res: serde_json::Value = policy
        .evaluate(&mut store, &entrypoint, &input)
        .instrument(tracing::info_span!("evaluate"))
        .await?;

    println!("{res}");

    Ok(())
}
