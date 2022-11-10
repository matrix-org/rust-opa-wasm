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

//! Builtins used when working with globs.

use anyhow::Result;

/// Returns a string which represents a version of the pattern where all
/// asterisks have been escaped.
#[tracing::instrument(name = "glob.quote_meta", err)]
pub fn quote_meta(pattern: String) -> Result<String> {
    let mut needs_escape = false;

    // shortcircuit if possible to avoid copy
    for c in pattern.chars() {
        match c {
            '*' | '?' | '\\' | '[' | ']' | '{' | '}' => needs_escape = true,
            _ => {}
        }
    }
    if !needs_escape {
        return Ok(pattern);
    }

    let mut out = String::with_capacity(pattern.len());
    for c in pattern.chars() {
        match c {
            '*' | '?' | '\\' | '[' | ']' | '{' | '}' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    Ok(out)
}
