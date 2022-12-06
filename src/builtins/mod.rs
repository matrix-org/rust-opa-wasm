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

//! Handling of builtin functions.

use anyhow::{bail, Result};

use self::traits::{Builtin, BuiltinFunc};
use crate::EvaluationContext;

pub mod impls;
pub mod traits;

/// Resolve a builtin based on its name
///
/// # Errors
///
/// Returns an error if the builtin is not known
#[allow(clippy::too_many_lines)]
pub fn resolve<C: EvaluationContext>(name: &str) -> Result<Box<dyn Builtin<C>>> {
    match name {
        #[cfg(feature = "base64url-builtins")]
        "base64url.encode_no_pad" => Ok(self::impls::base64url::encode_no_pad.wrap()),

        #[cfg(all(feature = "crypto-md5-builtins", feature = "crypto-hmac-builtins"))]
        "crypto.hmac.md5" => Ok(self::impls::crypto::hmac::md5.wrap()),

        #[cfg(all(feature = "crypto-sha1-builtins", feature = "crypto-hmac-builtins"))]
        "crypto.hmac.sha1" => Ok(self::impls::crypto::hmac::sha1.wrap()),

        #[cfg(all(feature = "crypto-sha2-builtins", feature = "crypto-hmac-builtins"))]
        "crypto.hmac.sha256" => Ok(self::impls::crypto::hmac::sha256.wrap()),

        #[cfg(all(feature = "crypto-sha2-builtins", feature = "crypto-hmac-builtins"))]
        "crypto.hmac.sha512" => Ok(self::impls::crypto::hmac::sha512.wrap()),

        #[cfg(all(feature = "crypto-md5-builtins", feature = "crypto-digest-builtins"))]
        "crypto.md5" => Ok(self::impls::crypto::digest::md5.wrap()),

        #[cfg(all(feature = "crypto-sha1-builtins", feature = "crypto-digest-builtins"))]
        "crypto.sha1" => Ok(self::impls::crypto::digest::sha1.wrap()),

        #[cfg(all(feature = "crypto-sha2-builtins", feature = "crypto-digest-builtins"))]
        "crypto.sha256" => Ok(self::impls::crypto::digest::sha256.wrap()),

        "crypto.x509.parse_and_verify_certificates" => {
            Ok(self::impls::crypto::x509::parse_and_verify_certificates.wrap())
        }
        "crypto.x509.parse_certificate_request" => {
            Ok(self::impls::crypto::x509::parse_certificate_request.wrap())
        }
        "crypto.x509.parse_certificates" => {
            Ok(self::impls::crypto::x509::parse_certificates.wrap())
        }
        "crypto.x509.parse_rsa_private_key" => {
            Ok(self::impls::crypto::x509::parse_rsa_private_key.wrap())
        }
        "glob.quote_meta" => Ok(self::impls::glob::quote_meta.wrap()),
        "graph.reachable_paths" => Ok(self::impls::graph::reachable_paths.wrap()),
        "graphql.is_valid" => Ok(self::impls::graphql::is_valid.wrap()),
        "graphql.parse" => Ok(self::impls::graphql::parse.wrap()),
        "graphql.parse_and_verify" => Ok(self::impls::graphql::parse_and_verify.wrap()),
        "graphql.parse_query" => Ok(self::impls::graphql::parse_query.wrap()),
        "graphql.parse_schema" => Ok(self::impls::graphql::parse_schema.wrap()),

        #[cfg(feature = "hex-builtins")]
        "hex.decode" => Ok(self::impls::hex::decode.wrap()),

        #[cfg(feature = "hex-builtins")]
        "hex.encode" => Ok(self::impls::hex::encode.wrap()),

        "http.send" => Ok(self::impls::http::send.wrap()),
        "indexof_n" => Ok(self::impls::indexof_n.wrap()),
        "io.jwt.decode" => Ok(self::impls::io::jwt::decode.wrap()),
        "io.jwt.decode_verify" => Ok(self::impls::io::jwt::decode_verify.wrap()),
        "io.jwt.encode_sign" => Ok(self::impls::io::jwt::encode_sign.wrap()),
        "io.jwt.encode_sign_raw" => Ok(self::impls::io::jwt::encode_sign_raw.wrap()),
        "io.jwt.verify_es256" => Ok(self::impls::io::jwt::verify_es256.wrap()),
        "io.jwt.verify_es384" => Ok(self::impls::io::jwt::verify_es384.wrap()),
        "io.jwt.verify_es512" => Ok(self::impls::io::jwt::verify_es512.wrap()),
        "io.jwt.verify_hs256" => Ok(self::impls::io::jwt::verify_hs256.wrap()),
        "io.jwt.verify_hs384" => Ok(self::impls::io::jwt::verify_hs384.wrap()),
        "io.jwt.verify_hs512" => Ok(self::impls::io::jwt::verify_hs512.wrap()),
        "io.jwt.verify_ps256" => Ok(self::impls::io::jwt::verify_ps256.wrap()),
        "io.jwt.verify_ps384" => Ok(self::impls::io::jwt::verify_ps384.wrap()),
        "io.jwt.verify_ps512" => Ok(self::impls::io::jwt::verify_ps512.wrap()),
        "io.jwt.verify_rs256" => Ok(self::impls::io::jwt::verify_rs256.wrap()),
        "io.jwt.verify_rs384" => Ok(self::impls::io::jwt::verify_rs384.wrap()),
        "io.jwt.verify_rs512" => Ok(self::impls::io::jwt::verify_rs512.wrap()),

        #[cfg(feature = "json-builtins")]
        "json.patch" => Ok(self::impls::json::patch.wrap()),

        "net.cidr_contains_matches" => Ok(self::impls::net::cidr_contains_matches.wrap()),
        "net.cidr_expand" => Ok(self::impls::net::cidr_expand.wrap()),
        "net.cidr_merge" => Ok(self::impls::net::cidr_merge.wrap()),
        "net.lookup_ip_addr" => Ok(self::impls::net::lookup_ip_addr.wrap()),
        "object.union_n" => Ok(self::impls::object::union_n.wrap()),
        "opa.runtime" => Ok(self::impls::opa::runtime.wrap()),

        #[cfg(feature = "rng")]
        "rand.intn" => Ok(self::impls::rand::intn.wrap()),

        "regex.find_n" => Ok(self::impls::regex::find_n.wrap()),
        "regex.globs_match" => Ok(self::impls::regex::globs_match.wrap()),
        "regex.split" => Ok(self::impls::regex::split.wrap()),
        "regex.template_match" => Ok(self::impls::regex::template_match.wrap()),
        "rego.parse_module" => Ok(self::impls::rego::parse_module.wrap()),

        #[cfg(feature = "semver-builtins")]
        "semver.compare" => Ok(self::impls::semver::compare.wrap()),

        #[cfg(feature = "semver-builtins")]
        "semver.is_valid" => Ok(self::impls::semver::is_valid.wrap()),

        #[cfg(feature = "sprintf-builtins")]
        "sprintf" => Ok(self::impls::sprintf.wrap()),

        #[cfg(feature = "time-builtins")]
        "time.add_date" => Ok(self::impls::time::add_date.wrap()),

        #[cfg(feature = "time-builtins")]
        "time.clock" => Ok(self::impls::time::clock.wrap()),

        #[cfg(feature = "time-builtins")]
        "time.date" => Ok(self::impls::time::date.wrap()),

        #[cfg(feature = "time-builtins")]
        "time.diff" => Ok(self::impls::time::diff.wrap()),

        #[cfg(feature = "time-builtins")]
        "time.now_ns" => Ok(self::impls::time::now_ns.wrap()),

        #[cfg(feature = "time-builtins")]
        "time.parse_duration_ns" => Ok(self::impls::time::parse_duration_ns.wrap()),

        #[cfg(feature = "time-builtins")]
        "time.parse_ns" => Ok(self::impls::time::parse_ns.wrap()),

        #[cfg(feature = "time-builtins")]
        "time.parse_rfc3339_ns" => Ok(self::impls::time::parse_rfc3339_ns.wrap()),

        #[cfg(feature = "time-builtins")]
        "time.weekday" => Ok(self::impls::time::weekday.wrap()),

        "trace" => Ok(self::impls::trace.wrap()),

        #[cfg(feature = "units-builtins")]
        "units.parse" => Ok(self::impls::units::parse.wrap()),

        #[cfg(feature = "units-builtins")]
        "units.parse_bytes" => Ok(self::impls::units::parse_bytes.wrap()),

        #[cfg(feature = "urlquery-builtins")]
        "urlquery.decode" => Ok(self::impls::urlquery::decode.wrap()),

        #[cfg(feature = "urlquery-builtins")]
        "urlquery.decode_object" => Ok(self::impls::urlquery::decode_object.wrap()),

        #[cfg(feature = "urlquery-builtins")]
        "urlquery.encode" => Ok(self::impls::urlquery::encode.wrap()),

        #[cfg(feature = "urlquery-builtins")]
        "urlquery.encode_object" => Ok(self::impls::urlquery::encode_object.wrap()),

        "uuid.rfc4122" => Ok(self::impls::uuid::rfc4122.wrap()),

        #[cfg(feature = "yaml-builtins")]
        "yaml.is_valid" => Ok(self::impls::yaml::is_valid.wrap()),

        #[cfg(feature = "yaml-builtins")]
        "yaml.marshal" => Ok(self::impls::yaml::marshal.wrap()),

        #[cfg(feature = "yaml-builtins")]
        "yaml.unmarshal" => Ok(self::impls::yaml::unmarshal.wrap()),
        _ => bail!("unknown builtin"),
    }
}
