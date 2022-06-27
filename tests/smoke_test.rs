use anyhow::anyhow;
use anyhow::Result as AnyResult;
use flate2::read::GzDecoder;
use insta::assert_yaml_snapshot;
use opa_wasm::Runtime;
use std::io::Read;
use std::path::Path;
use tar::Archive;
use wasmtime::{Config, Engine, Module, Store};

async fn eval_policy(bundle: &str, entrypoint: &str, input: &str) -> AnyResult<serde_json::Value> {
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
    let input_bytes = tokio::fs::read(input).await?;
    let input: serde_json::Value = serde_json::from_slice(&input_bytes[..])?;

    // Evaluate the policy
    let p: serde_json::Value = policy.evaluate(&mut store, entrypoint, &input).await?;
    Ok(p)
}

async fn load_wasm(bundle: &str) -> AnyResult<Vec<u8>> {
    let f = tokio::fs::read(bundle).await?;
    let mut archive = Archive::new(GzDecoder::new(&f[..]));

    match archive.entries()?.flatten().find(|e| {
        e.path()
            .context("tar malformed: entry has no path")?
            .ends_with("policy.wasm")
    }) {
        Some(mut e) => {
            let mut v = Vec::new();
            e.read_to_end(&mut v)?;
            Ok(v)
        }
        None => Err(anyhow!("no wasm entry found")),
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

async fn test_policy(bundle_name: &str, data: &str) -> serde_json::Value {
    eval_policy(
        &bundle(&format!("{}.rego.tar.gz", bundle_name)),
        "example",
        &input(&format!("{}.json", data)),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn infra_loader_works() {
    assert_eq!(
        134000,
        load_wasm("tests/infra-fixtures/test-loader.rego.tar.gz")
            .await
            .unwrap()
            .len()
    );
}

const TESTS: &[(&str, &str)] = &[
    ("test-loader", "test-loader.true"),
    ("test-loader", "test-loader.false"),
    // ("test-uuid", "test-uuid"), this test should be tested with redactions because a uuid is random
];

macro_rules! set_snapshot_suffix {
    ($($expr:expr),*) => {{
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_suffix(format!($($expr,)*));
        settings.bind_to_thread();
    }}
}
#[tokio::test]
async fn test_uuid() {
    assert_yaml_snapshot!(test_policy("test-uuid", "test-uuid").await, {
    "[0].result.policy[0]" => insta::dynamic_redaction(|value, _path| {
        // assert that the value looks like a uuid here
        assert_eq!(value
            .as_str()
            .unwrap()
            .chars()
            .filter(|&c| c == '-')
            .count(),
            4
        );
        "[uuid]"
    })})
}

#[tokio::test]
async fn test_cycle() {
    for (bundle, data) in TESTS {
        set_snapshot_suffix!("{}-{}", bundle, data);
        assert_yaml_snapshot!(test_policy(bundle, data).await)
    }
}
