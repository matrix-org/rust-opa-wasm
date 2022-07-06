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

//! Builtins related to GraphQL schema and query parsing and validation

use anyhow::{bail, Result};

/// Checks that a GraphQL query is valid against a given schema.
#[tracing::instrument(name = "graphql.is_valid", err)]
pub fn is_valid(query: String, schema: String) -> Result<bool> {
    bail!("not implemented");
}

/// Returns AST objects for a given GraphQL query and schema after validating
/// the query against the schema. Returns undefined if errors were encountered
/// during parsing or validation.
#[tracing::instrument(name = "graphql.parse", err)]
pub fn parse(query: String, schema: String) -> Result<(serde_json::Value, serde_json::Value)> {
    bail!("not implemented");
}

/// Returns a boolean indicating success or failure alongside the parsed ASTs
/// for a given GraphQL query and schema after validating the query against the
/// schema.
#[tracing::instrument(name = "graphql.parse_and_verify", err)]
pub fn parse_and_verify(
    query: String,
    schema: String,
) -> Result<(bool, serde_json::Value, serde_json::Value)> {
    bail!("not implemented");
}

/// Returns an AST object for a GraphQL query.
#[tracing::instrument(name = "graphql.parse_query", err)]
pub fn parse_query(query: String) -> Result<serde_json::Value> {
    bail!("not implemented");
}

/// Returns an AST object for a GraphQL schema.
#[tracing::instrument(name = "graphql.parse_schema", err)]
pub fn parse_schema(schema: String) -> Result<serde_json::Value> {
    bail!("not implemented");
}
