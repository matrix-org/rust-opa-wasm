# Rust Open Policy Agent SDK

A crate to use OPA policies compiled to WASM.

## Try it out

This includes a CLI tool to try out the SDK implementation.

```text
cargo run --features=cli --      \
    --module ./policy.wasm       \
    --data-path ./data.json      \
    --input '{"hello": "world"}' \
    --entrypoint 'hello/world'
```

Set the `RUST_LOG` environment variable to `info` to show timings informations about the execution.

```text
opa-wasm
Evaluates OPA policies compiled as WASM modules

USAGE:
    opa-eval [OPTIONS] --entrypoint <ENTRYPOINT> <--module <MODULE>|--bundle <BUNDLE>>

OPTIONS:
    -m, --module <MODULE>            Path to the WASM module
    -b, --bundle <BUNDLE>            Path to the OPA bundle
    -e, --entrypoint <ENTRYPOINT>    Entrypoint to use
    -d, --data <JSON>                JSON literal to use as data
    -D, --data-path <PATH>           Path to a JSON file to load as data
    -i, --input <JSON>               JSON literal to use as input
    -I, --input-path <PATH>          Path to a JSON file to load as data
    -h, --help                       Print help information
```

## As a library

```rust,no_run
use std::collections::HashMap;

use anyhow::Result;
use wasmtime::{Config, Engine, Module, Store};

use opa_wasm::Runtime;

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

    println!("{}", res);

    Ok(())
}
```
