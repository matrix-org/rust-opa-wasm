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

//! Builtins related to regular expressions

use anyhow::{bail, Result};

/// Returns the specified number of matches when matching the input against the
/// pattern.
#[tracing::instrument(name = "regex.find_n", err)]
pub fn find_n(pattern: String, value: String, number: i64) -> Result<Vec<String>> {
    bail!("not implemented");
}

/// Checks if the intersection of two glob-style regular expressions matches a
/// non-empty set of non-empty strings.
///
/// The set of regex symbols is limited for this builtin: only `.`, `*`, `+`,
/// `[`, `-`, `]` and `\\` are treated as special symbols.
#[tracing::instrument(name = "regex.globs_match", err)]
pub fn globs_match(glob1: String, glob2: String) -> Result<bool> {
    bail!("not implemented");
}

/// Splits the input string by the occurrences of the given pattern.
#[tracing::instrument(name = "regex.split", err)]
pub fn split(pattern: String, value: String) -> Result<Vec<String>> {
    bail!("not implemented");
}

/// Matches a string against a pattern, where there pattern may be glob-like
#[tracing::instrument(name = "regex.template_match", err)]
pub fn template_match(
    pattern: String,
    value: String,
    delimiter_start: String,
    delimiter_end: String,
) -> Result<bool> {
    bail!("not implemented");
}
