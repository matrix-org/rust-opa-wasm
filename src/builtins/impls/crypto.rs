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

//! Builtins related to cryptographic operations

/// Builtins for computing HMAC signatures
#[cfg(all(
    feature = "crypto-hmac-builtins",
    any(
        feature = "crypto-md5-builtins",
        feature = "crypto-sha1-builtins",
        feature = "crypto-sha2-builtins"
    )
))]
pub mod hmac {
    use anyhow::Result;
    use hmac::{Hmac, Mac};

    #[cfg(feature = "crypto-md5-builtins")]
    /// Returns a string representing the MD5 HMAC of the input message using
    /// the input key.
    #[tracing::instrument(name = "crypto.hmac.md5", err)]
    pub fn md5(x: String, key: String) -> Result<String> {
        let mut mac = Hmac::<md5::Md5>::new_from_slice(key.as_bytes())?;
        mac.update(x.as_bytes());
        let res = mac.finalize();
        Ok(hex::encode(res.into_bytes()))
    }

    #[cfg(feature = "crypto-sha1-builtins")]
    /// Returns a string representing the SHA1 HMAC of the input message using
    /// the input key.
    #[tracing::instrument(name = "crypto.hmac.sha1", err)]
    pub fn sha1(x: String, key: String) -> Result<String> {
        let mut mac = Hmac::<sha1::Sha1>::new_from_slice(key.as_bytes())?;
        mac.update(x.as_bytes());
        let res = mac.finalize();
        Ok(hex::encode(res.into_bytes()))
    }

    #[cfg(feature = "crypto-sha2-builtins")]
    /// Returns a string representing the SHA256 HMAC of the input message using
    /// the input key.
    #[tracing::instrument(name = "crypto.hmac.sha256", err)]
    pub fn sha256(x: String, key: String) -> Result<String> {
        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(key.as_bytes())?;
        mac.update(x.as_bytes());
        let res = mac.finalize();
        Ok(hex::encode(res.into_bytes()))
    }

    #[cfg(feature = "crypto-sha2-builtins")]
    /// Returns a string representing the SHA512 HMAC of the input message using
    /// the input key.
    #[tracing::instrument(name = "crypto.hmac.sha512", err)]
    pub fn sha512(x: String, key: String) -> Result<String> {
        let mut mac = Hmac::<sha2::Sha512>::new_from_slice(key.as_bytes())?;
        mac.update(x.as_bytes());
        let res = mac.finalize();
        Ok(hex::encode(res.into_bytes()))
    }
}

/// Builtins for computing hashes
#[cfg(all(
    feature = "crypto-digest-builtins",
    any(
        feature = "crypto-md5-builtins",
        feature = "crypto-sha1-builtins",
        feature = "crypto-sha2-builtins"
    )
))]
pub mod digest {
    use digest::Digest;

    #[cfg(feature = "crypto-md5-builtins")]
    /// Returns a string representing the input string hashed with the MD5
    /// function
    #[tracing::instrument(name = "crypto.md5")]
    pub fn md5(x: String) -> String {
        let mut hasher = md5::Md5::new();
        hasher.update(x.as_bytes());
        let res = hasher.finalize();
        hex::encode(res)
    }

    #[cfg(all(feature = "crypto-digest-builtins", feature = "crypto-sha1-builtins"))]
    /// Returns a string representing the input string hashed with the SHA1
    /// function
    #[tracing::instrument(name = "crypto.sha1")]
    pub fn sha1(x: String) -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update(x.as_bytes());
        let res = hasher.finalize();
        hex::encode(res)
    }

    #[cfg(all(feature = "crypto-digest-builtins", feature = "crypto-sha2-builtins"))]
    /// Returns a string representing the input string hashed with the SHA256
    /// function
    #[tracing::instrument(name = "crypto.sha256")]
    pub fn sha256(x: String) -> String {
        let mut hasher = sha2::Sha256::new();
        hasher.update(x.as_bytes());
        let res = hasher.finalize();
        hex::encode(res)
    }
}

/// Builtins related to X509 certificates, keys and certificate requests parsing
/// and validation
pub mod x509 {
    use std::collections::HashMap;

    use anyhow::{bail, Result};

    type X509 = HashMap<String, serde_json::Value>;
    type Jwk = HashMap<String, serde_json::Value>;

    /// Returns one or more certificates from the given string containing PEM or
    /// base64 encoded DER certificates after verifying the supplied
    /// certificates form a complete certificate chain back to a trusted
    /// root.
    ///
    /// The first certificate is treated as the root and the last is treated as
    /// the leaf, with all others being treated as intermediates.
    #[tracing::instrument(name = "crypto.x509.parse_and_verify_certificates", err)]
    pub fn parse_and_verify_certificates(certs: String) -> Result<(bool, Vec<X509>)> {
        bail!("not implemented");
    }

    /// Returns a PKCS #10 certificate signing request from the given
    /// PEM-encoded PKCS#10 certificate signing request.
    #[tracing::instrument(name = "crypto.x509.parse_certificate_request", err)]
    pub fn parse_certificate_request(csr: String) -> Result<X509> {
        bail!("not implemented");
    }

    /// Returns one or more certificates from the given base64 encoded string
    /// containing DER encoded certificates that have been concatenated.
    #[tracing::instrument(name = "crypto.x509.parse_certificates", err)]
    pub fn parse_certificates(certs: String) -> Result<Vec<X509>> {
        bail!("not implemented");
    }

    /// Returns a JWK for signing a JWT from the given PEM-encoded RSA private
    /// key.
    #[tracing::instrument(name = "crypto.x509.parse_rsa_private_key", err)]
    pub fn parse_rsa_private_key(pem: String) -> Result<Jwk> {
        bail!("not implemented");
    }
}
