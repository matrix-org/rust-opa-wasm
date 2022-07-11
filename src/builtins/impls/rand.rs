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

//! Builtins used to generate pseudo-random values

use anyhow::{bail, Result};
use rand::Rng;

use crate::EvaluationContext;

/// Returns a random integer between `0` and `n` (`n` exlusive). If `n` is `0`,
/// then `y` is always `0`. For any given argument pair (`str`, `n`), the output
/// will be consistent throughout a query evaluation.
#[tracing::instrument(name = "rand.intn", skip(ctx), err)]
pub fn intn<C: EvaluationContext>(ctx: &mut C, str: String, n: i64) -> Result<i64> {
    if n == 0 {
        return Ok(0);
    }

    if n < 0 {
        bail!("rand.intn: n must be a positive integer")
    }

    let cache_key = ("rand", str, n);
    if let Some(v) = ctx.cache_get(&cache_key)? {
        return Ok(v);
    };

    let mut rng = ctx.get_rng();
    let val = rng.gen_range(0..n);
    ctx.cache_set(&cache_key, &val)?;
    Ok(val)
}
