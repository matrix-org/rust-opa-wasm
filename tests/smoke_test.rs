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
use wasmtime::{Config, Engine, Module, Store};

macro_rules! integration_test {
    ($name:ident, $suite:expr) => {
        #[tokio::test]
        async fn $name() {
            assert_yaml_snapshot!(test_policy_with_datafile($suite, None)
                .await
                .expect("error in test suite"));
        }
    };
    ($name:ident, $suite:expr, input = $input:expr) => {
        #[tokio::test]
        async fn $name() {
            assert_yaml_snapshot!(test_policy_with_datafile($suite, Some($input))
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
    Path::new("tests/fixtures")
        .join(name)
        .to_string_lossy()
        .into()
}

fn input(name: &str) -> String {
    Path::new("tests/fixtures")
        .join(name)
        .to_string_lossy()
        .into()
}

async fn test_policy_with_datafile(
    bundle_name: &str,
    datafile_path: Option<&str>,
) -> AnyResult<serde_json::Value> {
    let input = match datafile_path {
        Some(path) => {
            let bytes = tokio::fs::read(input(&format!("{}.json", path))).await?;
            Some(serde_json::from_slice(&bytes[..])?)
        }
        None => None,
    };
    test_policy(bundle_name, input).await
}

async fn test_policy(
    bundle_name: &str,
    data: Option<serde_json::Value>,
) -> AnyResult<serde_json::Value> {
    let input = data.unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::default()));
    eval_policy(
        &bundle(&format!("{}.rego.tar.gz", bundle_name)),
        "fixtures",
        &input,
    )
    .await
}

#[tokio::test]
async fn infra_loader_works() {
    let module = read_bundle("tests/fixtures/test-loader.rego.tar.gz")
        .await
        .unwrap();

    // Look for the WASM magic preamble
    assert_eq!(module[..4], [0x00, 0x61, 0x73, 0x6D]);
    // And for the WASM binary format version
    assert_eq!(module[4..8], [0x01, 0x00, 0x00, 0x00]);
}

integration_test!(
    test_loader_false,
    "test-loader",
    input = "test-loader.false"
);
integration_test!(test_loader_true, "test-loader", input = "test-loader.true");
integration_test!(test_loader_empty, "test-loader");
integration_test!(test_units, "test-units");
integration_test!(test_rand, "test-rand");
integration_test!(test_yaml, "test-yaml");
integration_test!(test_urlquery, "test-urlquery");
integration_test!(test_time, "test-time");

#[tokio::test]
async fn test_http() {
    let server = httpmock::MockServer::start();

    let content_value: serde_json::Value = serde_json::json!({ "key": "value" });

    let get_json_mock = server.mock(|when, then| {
        when.method(httpmock::Method::GET).path("/json");
        then.status(200)
            .header("Content-Type", "application/json")
            .body(content_value.to_string());
    });

    let get_yaml_mock = server.mock(|when, then| {
        when.method(httpmock::Method::GET).path("/yaml");
        then.status(200)
            .header("Content-Type", "application/yaml")
            .body(serde_yaml::to_string(&content_value).unwrap());
    });

    let post_json_mock = server.mock(|when, then| {
        when.method(httpmock::Method::POST)
            .path("/post")
            .json_body(content_value.clone());
        then.status(200);
    });

    let redirect_json_mock = server.mock(|when, then| {
        when.method(httpmock::Method::GET).path("/redirect");
        then.status(302).header("Location", "/target");
    });

    let target_json_mock = server.mock(|when, then| {
        when.method(httpmock::Method::GET).path("/target");
        then.status(200);
    });

    let res = test_policy(
        "test-http",
        Some(serde_json::json!({"base_url": server.url("")})),
    )
    .await
    .expect("error in test suite");

    let result = res.as_array().unwrap()[0]
        .as_object()
        .unwrap()
        .get("result")
        .unwrap()
        .as_object()
        .unwrap();

    get_json_mock.assert();
    let get_json_res = result.get("get_json").unwrap().as_object().unwrap();
    assert_eq!(
        get_json_res.get("raw_body").unwrap(),
        &content_value.to_string()
    );
    assert_eq!(get_json_res.get("body").unwrap(), &content_value);

    get_yaml_mock.assert();
    let get_yaml_res = result.get("get_yaml").unwrap().as_object().unwrap();
    assert_eq!(
        get_yaml_res.get("raw_body").unwrap(),
        &serde_yaml::to_string(&content_value).unwrap()
    );
    assert_eq!(get_yaml_res.get("body").unwrap(), &content_value);

    post_json_mock.assert();
    let post_json_res = result.get("post_json").unwrap().as_object().unwrap();
    assert_eq!(post_json_res.get("status_code").unwrap(), &200);

    let get_no_conn_res = result.get("get_no_conn").unwrap().as_object().unwrap();
    assert_eq!(get_no_conn_res.get("status_code").unwrap(), &0);

    redirect_json_mock.assert_hits(2);
    target_json_mock.assert();

    let get_redirect_res = result.get("get_redirect").unwrap().as_object().unwrap();
    assert_eq!(get_redirect_res.get("status_code").unwrap(), &302);

    let get_redirect_follow_res = result
        .get("get_redirect_follow")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(get_redirect_follow_res.get("status_code").unwrap(), &200);
}

/*
#[tokio::test]
async fn test_uuid() {
    assert_yaml_snapshot!(test_policy("test-uuid", "test-uuid").await);
}
*/
