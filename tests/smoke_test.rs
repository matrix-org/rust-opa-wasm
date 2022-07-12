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

use std::path::Path;

use anyhow::Result as AnyResult;
use insta::assert_yaml_snapshot;
use opa_wasm::{read_bundle, Runtime, TestContext};
use serde_json::json;
use wasmtime::{Config, Engine, Module, Store};

macro_rules! integration_test {
    ($name:ident, $suite:expr) => {
        #[tokio::test]
        async fn $name() {
            assert_yaml_snapshot!(test_policy($suite, None)
                .await
                .expect("error in test suite"));
        }
    };
    ($name:ident, $suite:expr, input = $input:expr) => {
        #[tokio::test]
        async fn $name() {
            assert_yaml_snapshot!(test_policy($suite, Some($input))
                .await
                .expect("error in test suite"));
        }
    };
}

async fn eval_policy(
    bundle: &str,
    entrypoint: &str,
    input: &serde_json::Value,
) -> AnyResult<serde_json::Value> {
    let module = read_bundle(bundle).await?;

    // Configure the WASM runtime
    let mut config = Config::new();
    config.async_support(true);

    let engine = Engine::new(&config)?;

    let module = Module::new(&engine, module)?;

    // Create a store which will hold the module instance
    let mut store = Store::new(&engine, ());

    let ctx = TestContext::default();

    // Instantiate the module
    let runtime = Runtime::new_with_evaluation_context(&mut store, &module, ctx).await?;

    let policy = runtime.without_data(&mut store).await?;

    // Evaluate the policy
    let p: serde_json::Value = policy.evaluate(&mut store, entrypoint, &input).await?;
    Ok(p)
}

fn bundle(name: &str) -> String {
    Path::new("tests/infra-fixtures")
        .join(name)
        .to_string_lossy()
        .into()
}

fn input(name: &str) -> String {
    Path::new("tests/infra-fixtures")
        .join(name)
        .to_string_lossy()
        .into()
}

async fn test_policy(bundle_name: &str, data: Option<&str>) -> AnyResult<serde_json::Value> {
    let input = if let Some(data) = data {
        let input_bytes = tokio::fs::read(input(&format!("{}.json", data))).await?;
        serde_json::from_slice(&input_bytes[..])?
    } else {
        json!({})
    };
    eval_policy(
        &bundle(&format!("{}.rego.tar.gz", bundle_name)),
        "test",
        &input,
    )
    .await
}

#[tokio::test]
async fn infra_loader_works() {
    assert_eq!(
        133_988,
        read_bundle("tests/infra-fixtures/test-loader.rego.tar.gz")
            .await
            .unwrap()
            .len()
    );
}

integration_test!(
    test_loader_false,
    "test-loader",
    input = "test-loader.false"
);
integration_test!(test_loader_true, "test-loader", input = "test-loader.true");
integration_test!(test_loader_empty, "test-loader");
integration_test!(test_rand, "test-rand");

/*
#[tokio::test]
async fn test_uuid() {
    assert_yaml_snapshot!(test_policy("test-uuid", "test-uuid").await);
}
*/
