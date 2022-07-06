use std::path::Path;

use anyhow::Result as AnyResult;
use insta::assert_yaml_snapshot;
use opa_wasm::{read_bundle, Runtime};
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

    let data = serde_json::json!({});

    // Instantiate the module
    let runtime = Runtime::new(&mut store, &module).await?;

    let policy = runtime.with_data(&mut store, &data).await?;

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

/*
#[tokio::test]
async fn test_uuid() {
    assert_yaml_snapshot!(test_policy("test-uuid", "test-uuid").await);
}
*/
