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

//! Builtins related to network operations and IP handling

use std::collections::HashSet;

use anyhow::{bail, Result};

/// Checks if collections of cidrs or ips are contained within another
/// collection of cidrs and returns matches. This function is similar to
/// `net.cidr_contains` except it allows callers to pass collections of CIDRs or
/// IPs as arguments and returns the matches (as opposed to a boolean
/// result indicating a match between two CIDRs/IPs).
#[tracing::instrument(name = "net.cidr_contains_matches", err)]
pub fn cidr_contains_matches(
    cidrs: serde_json::Value,
    cidrs_or_ips: serde_json::Value,
) -> Result<serde_json::Value> {
    bail!("not implemented");
}

/// Expands CIDR to set of hosts  (e.g., `net.cidr_expand("192.168.0.0/30")`
/// generates 4 hosts: `{"192.168.0.0", "192.168.0.1", "192.168.0.2",
/// "192.168.0.3"}`).
#[tracing::instrument(name = "net.cidr_expand", err)]
pub fn cidr_expand(cidr: String) -> Result<HashSet<String>> {
    bail!("not implemented");
}

/// Merges IP addresses and subnets into the smallest possible list of CIDRs
/// (e.g., `net.cidr_merge(["192.0.128.0/24", "192.0.129.0/24"])` generates
/// `{"192.0.128.0/23"}`. This function merges adjacent subnets where possible,
/// those contained within others and also removes any duplicates.
///
/// Supports both IPv4 and IPv6 notations. IPv6 inputs need a prefix length
/// (e.g. "/128").
#[tracing::instrument(name = "net.cidr_merge", err)]
pub fn cidr_merge(addrs: serde_json::Value) -> Result<HashSet<String>> {
    bail!("not implemented");
}

/// Returns the set of IP addresses (both v4 and v6) that the passed-in `name`
/// resolves to using the standard name resolution mechanisms available.
#[tracing::instrument(name = "net.lookup_ip_addr", err)]
pub async fn lookup_ip_addr(name: String) -> Result<HashSet<String>> {
    bail!("not implemented");
}
