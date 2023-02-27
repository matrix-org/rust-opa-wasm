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

#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;

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
    cache: HashMap<String, serde_json::Value>,

    #[cfg(feature = "time")]
    evaluation_time: chrono::DateTime<chrono::Utc>,
}

#[allow(clippy::derivable_impls)]
impl Default for DefaultContext {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),

            #[cfg(feature = "time")]
            evaluation_time: chrono::Utc.timestamp_nanos(0),
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

pub mod tests {
    use anyhow::Result;
    #[cfg(feature = "time")]
    use chrono::TimeZone;
    use serde::{de::DeserializeOwned, Serialize};

    use crate::{DefaultContext, EvaluationContext};

    /// A context used in tests
    pub struct TestContext {
        inner: DefaultContext,

        #[cfg(feature = "time")]
        clock: chrono::DateTime<chrono::Utc>,

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
                    .with_ymd_and_hms(2020, 7, 14, 12, 53, 22)
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

        fn cache_get<K: Serialize, C: DeserializeOwned>(&mut self, key: &K) -> Result<Option<C>> {
            self.inner.cache_get(key)
        }

        fn cache_set<K: Serialize, C: Serialize>(&mut self, key: &K, content: &C) -> Result<()> {
            self.inner.cache_set(key, content)
        }
    }
}
