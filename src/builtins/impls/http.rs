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

//! Builtins used to make HTTP request

use std::{collections::HashMap, future::Future, pin::Pin, time::Duration};

use anyhow::{Context, Result};
use serde_json::{self, Map};
use serde_yaml;
use tokio::time::sleep;

use crate::{builtins::traits::Builtin, EvaluationContext};

/// This builtin is needed because the wrapper in traits.rs doesn't work when
/// dealing with async+context.
pub struct HttpSendBuiltin;

impl<C: 'static> Builtin<C> for HttpSendBuiltin
where
    C: EvaluationContext,
{
    fn call<'a>(
        &'a self,
        context: &'a mut C,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, anyhow::Error>> + Send + 'a>> {
        Box::pin(async move {
            let [opa_req]: [&'a [u8]; 1] = args.try_into().ok().context("invalid arguments")?;
            let opa_req: serde_json::Value =
                serde_json::from_slice(opa_req).context("failed to convert opa_req argument")?;
            let res = send(context, opa_req).await?;
            let res = serde_json::to_vec(&res).context("could not serialize result")?;
            Ok(res)
        })
    }
}

/// Returns a HTTP response to the given HTTP request.
///
/// Wraps [`internal_send`] to add error handling regarding the `raise_error`
/// field in the OPA request.
#[tracing::instrument(name = "http.send", skip(ctx), err)]
pub async fn send<C: EvaluationContext>(
    ctx: &mut C,
    opa_req: serde_json::Value,
) -> Result<serde_json::Value> {
    let raise_error = opa_req
        .get("raise_error")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);

    match internal_send(ctx, opa_req).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            if raise_error {
                Err(e)
            } else {
                Ok(serde_json::json!({
                    "status_code": 0,
                    "error": { "message": e.to_string() },
                }))
            }
        }
    }
}

/// Sends a HTTP request and returns the response.
async fn internal_send<C: EvaluationContext>(
    ctx: &mut C,
    opa_req: serde_json::Value,
) -> Result<serde_json::Value> {
    let opa_req = opa_req
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("request must be a JSON object"))?;

    let http_req = convert_opa_req_to_http_req(opa_req)?;

    let timeout_value = opa_req.get("timeout");
    let timeout = if let Some(timeout_value) = timeout_value {
        if let Some(timeout_nanos) = timeout_value.as_u64() {
            Some(Duration::from_nanos(timeout_nanos))
        } else if let Some(timeout_str) = timeout_value.as_str() {
            duration_str::parse(timeout_str).ok()
        } else {
            None
        }
    } else {
        None
    };

    let enable_redirect = opa_req
        .get("enable_redirect")
        .and_then(serde_json::Value::as_bool);

    let max_retry_attempts = opa_req
        .get("max_retry_attempts")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);

    let mut http_resp_res: Result<http::Response<String>> = Err(anyhow::anyhow!("unreachable"));

    for attempt in 0..=max_retry_attempts {
        http_resp_res = ctx
            .send_http(http_req.clone(), timeout, enable_redirect)
            .await;
        if http_resp_res.is_ok() {
            break;
        }
        if max_retry_attempts > 0 {
            #[allow(clippy::cast_possible_truncation)]
            sleep(Duration::from_millis(500 * 2_u64.pow(attempt as u32))).await;
        }
    }

    let http_resp = http_resp_res?;

    let force_json_decode = opa_req
        .get("force_json_decode")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let force_yaml_decode = opa_req
        .get("force_yaml_decode")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    Ok(convert_http_resp_to_opa_resp(
        http_resp,
        force_json_decode,
        force_yaml_decode,
    ))
}

/// Converts an OPA request to an HTTP request.
fn convert_opa_req_to_http_req(
    opa_req: &Map<String, serde_json::Value>,
) -> Result<http::Request<String>> {
    let url = opa_req
        .get("url")
        .ok_or_else(|| anyhow::anyhow!("missing url"))?
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("url must be a string"))?;
    let method = opa_req
        .get("method")
        .ok_or_else(|| anyhow::anyhow!("missing method"))?
        .as_str()
        .map(str::to_uppercase)
        .ok_or_else(|| anyhow::anyhow!("method must be a string"))?;
    let headers = opa_req.get("headers").and_then(|v| v.as_object());

    let mut req_builder = http::Request::builder().method(method.as_str()).uri(url);
    if let Some(headers) = headers {
        for (key, value) in headers {
            req_builder = req_builder.header(key, value.to_string());
        }
    }

    let json_req_body = opa_req.get("body");
    let http_req = if let Some(json_req_body) = json_req_body {
        req_builder.body(json_req_body.to_string())?
    } else {
        let raw_req_body = opa_req
            .get("raw_body")
            .map(ToString::to_string)
            .unwrap_or_default();
        req_builder.body(raw_req_body)?
    };

    Ok(http_req)
}

/// Converts an HTTP response to an OPA response.
fn convert_http_resp_to_opa_resp(
    response: http::Response<String>,
    force_json_decode: bool,
    force_yaml_decode: bool,
) -> serde_json::Value {
    let response_headers = response
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str(), v.to_str().unwrap_or("")))
        .collect::<HashMap<_, _>>();

    let mut opa_resp = serde_json::json!({
        "status_code": response.status().as_u16(),
        "headers": response_headers,
    });

    let raw_resp_body = response.body().clone();
    opa_resp["raw_body"] = serde_json::Value::String(raw_resp_body.clone());

    let content_type = response
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap_or_default());

    if force_json_decode || content_type == Some("application/json") {
        if let Ok(parsed_body) = serde_json::from_str::<serde_json::Value>(&raw_resp_body) {
            opa_resp["body"] = parsed_body;
        }
    } else if force_yaml_decode
        || content_type == Some("application/yaml")
        || content_type == Some("application/x-yaml")
    {
        if let Ok(parsed_body) = serde_yaml::from_str::<serde_json::Value>(&raw_resp_body) {
            opa_resp["body"] = parsed_body;
        }
    }

    opa_resp
}
