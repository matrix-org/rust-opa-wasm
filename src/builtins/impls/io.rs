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

//! Builtins related to JWTs

/// Builtins related to JWT encode/decode and verification/signature
pub mod jwt {
    use std::collections::HashMap;

    use anyhow::{bail, Result};

    type Headers = serde_json::Value;
    type Payload = serde_json::Value;
    type Jwk = serde_json::Value;

    /// Decodes a JSON Web Token and outputs it as an object.
    #[tracing::instrument(name = "io.jwt.decode", err)]
    pub fn decode(jwt: String) -> Result<(Headers, Payload, String)> {
        bail!("not implemented");
    }

    /// Verifies a JWT signature under parameterized constraints and decodes the
    /// claims if it is valid.
    ///
    /// Supports the following algorithms: HS256, HS384, HS512, RS256, RS384,
    /// RS512, ES256, ES384, ES512, PS256, PS384 and PS512.
    #[tracing::instrument(name = "io.jwt.decode_verify", err)]
    pub fn decode_verify(
        jwt: String,
        constraints: HashMap<String, serde_json::Value>,
    ) -> Result<(bool, Headers, Payload)> {
        bail!("not implemented");
    }

    /// Encodes and optionally signs a JSON Web Token. Inputs are taken as
    /// objects, not encoded strings (see `io.jwt.encode_sign_raw`).
    #[tracing::instrument(name = "io.jwt.encode_sign", err)]
    pub fn encode_sign(
        headers: Headers,
        payload: Payload,
        key: Jwk,
    ) -> Result<(bool, Headers, Payload)> {
        bail!("not implemented");
    }

    /// Encodes and optionally signs a JSON Web Token.
    #[tracing::instrument(name = "io.jwt.encode_sign_raw", err)]
    pub fn encode_sign_raw(headers: String, payload: String, key: String) -> Result<String> {
        bail!("not implemented");
    }

    /// Verifies if a ES256 JWT signature is valid.
    #[tracing::instrument(name = "io.jwt.verify_es256", err)]
    pub fn verify_es256(jwt: String, certificate: String) -> Result<bool> {
        bail!("not implemented");
    }

    /// Verifies if a ES384 JWT signature is valid.
    #[tracing::instrument(name = "io.jwt.verify_es384", err)]
    pub fn verify_es384(jwt: String, certificate: String) -> Result<bool> {
        bail!("not implemented");
    }

    /// Verifies if a ES512 JWT signature is valid.
    #[tracing::instrument(name = "io.jwt.verify_es512", err)]
    pub fn verify_es512(jwt: String, certificate: String) -> Result<bool> {
        bail!("not implemented");
    }

    /// Verifies if a HS256 (secret) JWT signature is valid.
    #[tracing::instrument(name = "io.jwt.verify_hs256", err)]
    pub fn verify_hs256(jwt: String, secret: String) -> Result<bool> {
        bail!("not implemented");
    }

    /// Verifies if a HS384 (secret) JWT signature is valid.
    #[tracing::instrument(name = "io.jwt.verify_hs384", err)]
    pub fn verify_hs384(jwt: String, secret: String) -> Result<bool> {
        bail!("not implemented");
    }

    /// Verifies if a HS512 (secret) JWT signature is valid.
    #[tracing::instrument(name = "io.jwt.verify_hs512", err)]
    pub fn verify_hs512(jwt: String, secret: String) -> Result<bool> {
        bail!("not implemented");
    }

    /// Verifies if a PS256 JWT signature is valid.
    #[tracing::instrument(name = "io.jwt.verify_ps256", err)]
    pub fn verify_ps256(jwt: String, certificate: String) -> Result<bool> {
        bail!("not implemented");
    }

    /// Verifies if a PS384 JWT signature is valid.
    #[tracing::instrument(name = "io.jwt.verify_ps384", err)]
    pub fn verify_ps384(jwt: String, certificate: String) -> Result<bool> {
        bail!("not implemented");
    }

    /// Verifies if a PS512 JWT signature is valid.
    #[tracing::instrument(name = "io.jwt.verify_ps512", err)]
    pub fn verify_ps512(jwt: String, certificate: String) -> Result<bool> {
        bail!("not implemented");
    }

    /// Verifies if a RS256 JWT signature is valid.
    #[tracing::instrument(name = "io.jwt.verify_rs256", err)]
    pub fn verify_rs256(jwt: String, certificate: String) -> Result<bool> {
        bail!("not implemented");
    }

    /// Verifies if a RS384 JWT signature is valid.
    #[tracing::instrument(name = "io.jwt.verify_rs384", err)]
    pub fn verify_rs384(jwt: String, certificate: String) -> Result<bool> {
        bail!("not implemented");
    }

    /// Verifies if a RS512 JWT signature is valid.
    #[tracing::instrument(name = "io.jwt.verify_rs512", err)]
    pub fn verify_rs512(jwt: String, certificate: String) -> Result<bool> {
        bail!("not implemented");
    }
}
