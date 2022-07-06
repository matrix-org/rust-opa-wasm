#![feature(try_find)]
use anyhow::bail;
use anyhow::{Context, Result as AnyResult};
use flate2::read::GzDecoder;
use insta::assert_yaml_snapshot;
use opa_wasm::Runtime;
use serde_json::json;
use std::io::Read;
use std::path::Path;
use tar::Archive;
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
    let module = load_wasm(bundle).await?;

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

async fn load_wasm(bundle: &str) -> AnyResult<Vec<u8>> {
    let f = tokio::fs::read(bundle).await?;
    let mut archive = Archive::new(GzDecoder::new(&f[..]));

    match archive.entries()?.flatten().try_find(|e| {
        Ok(e.path()
            .context("tar malformed: entry has no path")?
            .ends_with("policy.wasm"))
    }) {
        Ok(Some(mut e)) => {
            let mut v = Vec::new();
            e.read_to_end(&mut v)?;
            Ok(v)
        }
        Ok(None) => bail!("no wasm entry found"),
        Err(err) => Err(err),
    }
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
        load_wasm("tests/infra-fixtures/test-loader.rego.tar.gz")
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
