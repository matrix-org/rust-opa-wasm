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

use anyhow::{bail, Context, Result};
use regex::Regex;

/// Returns the specified number of matches when matching the input against the
/// pattern.
#[tracing::instrument(name = "regex.find_n", err)]
pub fn find_n(pattern: String, value: String, number: i32) -> Result<Vec<String>> {
    let re = Regex::new(&pattern)?;
    Ok(re
        .find_iter(&value)
        .take(usize::try_from(number).unwrap_or(usize::MAX))
        .map(|m| m.as_str().to_string())
        .collect::<Vec<_>>())
}

/// Checks if the intersection of two glob-style regular expressions matches a
/// non-empty set of non-empty strings.
///
/// The set of regex symbols is limited for this builtin: only `.`, `*`, `+`,
/// `[`, `-`, `]` and `\\` are treated as special symbols.
#[tracing::instrument(name = "regex.globs_match", err)]
pub fn globs_match(glob1: String, glob2: String) -> Result<bool> {
    regex_intersect::non_empty(&glob1, &glob2).context("expressions should parse")
}

/// Splits the input string by the occurrences of the given pattern.
#[tracing::instrument(name = "regex.split", err)]
pub fn split(pattern: String, value: String) -> Result<Vec<String>> {
    let re = Regex::new(&pattern)?;
    Ok(re
        .split(&value)
        .into_iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>())
}

/// Matches a string against a pattern, where there pattern may be glob-like
#[tracing::instrument(name = "regex.template_match", err)]
pub fn template_match(
    pattern: String,
    value: String,
    delimiter_start: String,
    delimiter_end: String,
) -> Result<bool> {
    if let (Some(dstart), Some(dend)) =
        (delimiter_start.chars().next(), delimiter_end.chars().next())
    {
        route_pattern::is_match(&pattern, dstart, dend, &value)
            .context("route pattern should parse")
    } else {
        bail!("delimiters must be a single character each");
    }
}

/// Find and replaces the text using the regular expression pattern.
/// The semantics of `replace` in OPA is actually `replace_all`
#[tracing::instrument(name = "regex.replace", err)]
pub fn replace(s: String, pattern: String, value: String) -> Result<String> {
    let re = Regex::new(&pattern)?;
    Ok(re.replace_all(&s, &value).to_string())
}

/// Matches a string against a regular expression.
#[tracing::instrument(name = "regex.match", err)]
pub fn regex_match(pattern: String, value: String) -> Result<bool> {
    let re = Regex::new(&pattern)?;
    Ok(re.is_match(&value))
}

/// Checks if a string is a valid regular expression.
#[tracing::instrument(name = "regex.is_valid")]
pub fn is_valid(pattern: String) -> bool {
    // Note: we're using `regex` and not `regex-syntax` directly which may be
    // cheaper because `regex-syntax` is considered an implementation detail of
    // `regex`. So, since we're going to use `regex` in all other places, it's
    // better to use it to validate in case it decides to change its
    // implementation of parsing.
    Regex::new(&pattern).is_ok()
}

/// Returns all successive matches of the expression.
#[tracing::instrument(name = "regex.find_all_string_submatch_n", err)]
pub fn find_all_string_submatch_n(
    pattern: String,
    value: String,
    number: usize,
) -> Result<Vec<Vec<String>>> {
    let re = Regex::new(&pattern)?;
    Ok(re
        .captures_iter(&value)
        .take(number)
        .map(|m| {
            m.iter()
                .flatten()
                .map(|m| m.as_str().to_string())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>())
}
