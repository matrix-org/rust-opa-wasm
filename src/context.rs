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

//! Trait definition for the context passed through builtin evaluation

#![allow(clippy::module_name_repetitions)]

use std::{collections::HashMap, time::Duration};

use anyhow::Result;
#[cfg(feature = "time")]
use chrono::TimeZone;
use serde::{de::DeserializeOwned, Serialize};

/// Context passed through builtin evaluation
pub trait EvaluationContext: Send + 'static {
    /// The type of random number generator used by this context
    #[cfg(feature = "rng")]
    type Rng: rand::Rng;

    /// Get a [`rand::Rng`]
    #[cfg(feature = "rng")]
    fn get_rng(&mut self) -> Self::Rng;

    /// Get the current date and time
    #[cfg(feature = "time")]
    fn now(&self) -> chrono::DateTime<chrono::Utc>;

    /// Send an HTTP request
    fn send_http(
        &self,
        req: http::Request<String>,
        timeout: Option<Duration>,
        enable_redirect: Option<bool>,
    ) -> impl std::future::Future<Output = Result<http::Response<String>>> + Send + Sync;

    /// Notify the context on evaluation start, so it can clean itself up
    fn evaluation_start(&mut self);

    /// Get a value from the evaluation cache
    ///
    /// # Errors
    ///
    /// If the key failed to serialize, or the value failed to deserialize
    fn cache_get<K: Serialize, C: DeserializeOwned>(&mut self, key: &K) -> Result<Option<C>>;

    /// Push a value to the evaluation cache
    ///
    /// # Errors
    ///
    /// If the key or the value failed to serialize
    fn cache_set<K: Serialize, C: Serialize>(&mut self, key: &K, content: &C) -> Result<()>;
}

/// The default evaluation context implementation
pub struct DefaultContext {
    /// The cache used to store values during evaluation
    cache: HashMap<String, serde_json::Value>,

    /// The time at which the evaluation started
    #[cfg(feature = "time")]
    evaluation_time: chrono::DateTime<chrono::Utc>,

    /// The client used to send HTTP requests
    #[cfg(feature = "http")]
    http_client: reqwest::Client,
}

/// Builds a [`reqwest::Client`] with the given timeout and redirect policy.
fn build_reqwest_client(timeout: Duration, enable_redirect: bool) -> reqwest::Client {
    let mut client_builder = reqwest::Client::builder();
    client_builder = client_builder.timeout(timeout);
    client_builder = client_builder.redirect(if enable_redirect {
        reqwest::redirect::Policy::default()
    } else {
        reqwest::redirect::Policy::none()
    });
    #[allow(clippy::unwrap_used)]
    client_builder.build().unwrap()
}

/// The default HTTP timeout (5 seconds as specified in the OPA specification)
static DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(5);

/// The default HTTP redirect policy (disabled as specified in the OPA
/// specification)
static DEFAULT_HTTP_ENABLE_REDIRECT: bool = false;

#[allow(clippy::derivable_impls)]
impl Default for DefaultContext {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),

            #[cfg(feature = "time")]
            evaluation_time: chrono::Utc.timestamp_nanos(0),

            #[cfg(feature = "http")]
            http_client: build_reqwest_client(DEFAULT_HTTP_TIMEOUT, DEFAULT_HTTP_ENABLE_REDIRECT),
        }
    }
}

impl DefaultContext {
    /// Returns a [`reqwest::Client`] with the given timeout and redirect
    /// policy.
    ///
    /// If the timeout or redirect policy is different from the default ones, a
    /// new [`reqwest::Client`] is built. Otherwise, the existing client is
    /// returned.
    fn get_reqwest_client(&self, timeout: Duration, enable_redirect: bool) -> reqwest::Client {
        if timeout != DEFAULT_HTTP_TIMEOUT || enable_redirect != DEFAULT_HTTP_ENABLE_REDIRECT {
            build_reqwest_client(timeout, enable_redirect)
        } else {
            self.http_client.clone()
        }
    }
}

impl EvaluationContext for DefaultContext {
    #[cfg(feature = "rng")]
    type Rng = rand::rngs::ThreadRng;

    #[cfg(feature = "rng")]
    fn get_rng(&mut self) -> Self::Rng {
        rand::thread_rng()
    }

    #[cfg(feature = "time")]
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        self.evaluation_time
    }

    #[cfg(feature = "http")]
    async fn send_http(
        &self,
        req: http::Request<String>,
        timeout: Option<Duration>,
        enable_redirect: Option<bool>,
    ) -> Result<http::Response<String>> {
        let client = self.get_reqwest_client(
            timeout.unwrap_or(DEFAULT_HTTP_TIMEOUT),
            enable_redirect.unwrap_or(DEFAULT_HTTP_ENABLE_REDIRECT),
        );

        let response: reqwest::Response = client.execute(reqwest::Request::try_from(req)?).await?;

        let mut builder = http::Response::builder().status(response.status());
        for (name, value) in response.headers() {
            builder = builder.header(name, value);
        }

        let bytes_body = response.bytes().await?;
        let string_body = String::from_utf8(bytes_body.to_vec())?;
        builder
            .body(string_body)
            .map_err(|e| anyhow::anyhow!("Failed to build response: {}", e))
    }

    fn evaluation_start(&mut self) {
        // Clear the cache
        self.cache = HashMap::new();

        #[cfg(feature = "time")]
        {
            // Set the evaluation time to now
            self.evaluation_time = chrono::Utc::now();
        }
    }

    fn cache_get<K: Serialize, C: DeserializeOwned>(&mut self, key: &K) -> Result<Option<C>> {
        let key = serde_json::to_string(&key)?;
        let Some(value) = self.cache.get(&key) else {
            return Ok(None);
        };

        let value = serde_json::from_value(value.clone())?;
        Ok(value)
    }

    fn cache_set<K: Serialize, C: Serialize>(&mut self, key: &K, content: &C) -> Result<()> {
        let key = serde_json::to_string(key)?;
        let content = serde_json::to_value(content)?;
        self.cache.insert(key, content);
        Ok(())
    }
}

/// Test utilities
pub mod tests {
    use std::time::Duration;

    use anyhow::Result;
    #[cfg(feature = "time")]
    use chrono::TimeZone;
    use serde::{de::DeserializeOwned, Serialize};

    use crate::{DefaultContext, EvaluationContext};

    /// A context used in tests
    pub struct TestContext {
        /// The inner [`DefaultContext`]
        inner: DefaultContext,

        /// The mocked time
        #[cfg(feature = "time")]
        clock: chrono::DateTime<chrono::Utc>,

        /// The seed used for the random number generator
        #[cfg(feature = "rng")]
        seed: u64,
    }

    #[allow(clippy::derivable_impls)]
    impl Default for TestContext {
        fn default() -> Self {
            Self {
                inner: DefaultContext::default(),

                #[cfg(feature = "time")]
                clock: chrono::Utc
                    // Corresponds to 2020-07-14T12:53:22Z
                    // We're using this method because it's available on old versions of chrono
                    .timestamp_opt(1_594_731_202, 0)
                    .unwrap(),

                #[cfg(feature = "rng")]
                seed: 0,
            }
        }
    }

    impl EvaluationContext for TestContext {
        #[cfg(feature = "rng")]
        type Rng = rand::rngs::StdRng;

        fn evaluation_start(&mut self) {
            self.inner.evaluation_start();
        }

        #[cfg(feature = "time")]
        fn now(&self) -> chrono::DateTime<chrono::Utc> {
            self.clock
        }

        #[cfg(feature = "rng")]
        fn get_rng(&mut self) -> Self::Rng {
            use rand::SeedableRng;

            rand::rngs::StdRng::seed_from_u64(self.seed)
        }

        #[cfg(feature = "http")]
        async fn send_http(
            &self,
            req: http::Request<String>,
            timeout: Option<Duration>,
            enable_redirect: Option<bool>,
        ) -> Result<http::Response<String>> {
            self.inner.send_http(req, timeout, enable_redirect).await
        }

        fn cache_get<K: Serialize, C: DeserializeOwned>(&mut self, key: &K) -> Result<Option<C>> {
            self.inner.cache_get(key)
        }

        fn cache_set<K: Serialize, C: Serialize>(&mut self, key: &K, content: &C) -> Result<()> {
            self.inner.cache_set(key, content)
        }
    }
}
