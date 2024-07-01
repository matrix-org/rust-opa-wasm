// Copyright 2022-2024 The Matrix.org Foundation C.I.C.
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

//! Type wrappers to help with interacting with the OPA WASM module

use std::ffi::CStr;

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use wasmtime::{AsContext, AsContextMut, Instance, Memory};

/// An entrypoint ID, as returned by the `entrypoints` export, and given to the
/// `opa_eval_ctx_set_entrypoint` exports
#[derive(Debug, Deserialize, Clone)]
#[serde(transparent)]
pub struct EntrypointId(pub(crate) i32);

/// The ID of a builtin, as returned by the `builtins` export, and passed to the
/// `opa_builtin*` imports
#[derive(Debug, Deserialize, Clone)]
#[serde(transparent)]
pub struct BuiltinId(pub(crate) i32);

/// A value stored on the WASM heap, as used by the `opa_value_*` exports
#[derive(Debug)]
pub struct Value(pub(crate) i32);

/// A generic value on the WASM memory
#[derive(Debug)]
pub struct Addr(pub(crate) i32);

/// A heap allocation on the WASM memory
#[derive(Debug)]
pub struct Heap {
    /// The pointer to the start of the heap allocation
    pub(crate) ptr: i32,

    /// The length of the heap allocation
    pub(crate) len: i32,

    /// Whether the heap allocation has been freed
    pub(crate) freed: bool,
}

impl Heap {
    /// Get the end of the heap allocation
    pub const fn end(&self) -> i32 {
        self.ptr + self.len
    }

    /// Calculate the number of pages this heap allocation occupies
    pub fn pages(&self) -> u64 {
        let page_size = 64 * 1024;
        let addr = self.end();
        let page = addr / page_size;
        // This is safe as the heap pointers will never be negative. We use i32 for
        // convenience to avoid having to cast all the time.
        #[allow(clippy::expect_used)]
        if addr % page_size > 0 { page + 1 } else { page }
            .try_into()
            .expect("invalid heap address")
    }
}

impl Drop for Heap {
    fn drop(&mut self) {
        if !self.freed {
            tracing::warn!("forgot to free heap allocation");
            self.freed = true;
        }
    }
}

/// A null-terminated string on the WASM memory
#[derive(Debug)]
pub struct NulStr(pub(crate) i32);

impl NulStr {
    /// Read the null-terminated string from the WASM memory
    pub fn read<'s, T: AsContext>(&self, store: &'s T, memory: &Memory) -> Result<&'s CStr> {
        let mem = memory.data(store);
        let start: usize = self.0.try_into().context("invalid address")?;
        let mem = mem.get(start..).context("memory address out of bounds")?;
        let nul = mem
            .iter()
            .position(|c| *c == 0)
            .context("malformed string")?;
        let mem = mem
            .get(..=nul)
            .context("issue while extracting nul-terminated string")?;
        let res = CStr::from_bytes_with_nul(mem)?;
        Ok(res)
    }
}

/// The address of the evaluation context, used by the `opa_eval_ctx_*` exports
#[derive(Debug)]
pub struct Ctx(pub(crate) i32);

/// An error returned by the OPA module
#[derive(Debug, thiserror::Error)]
pub enum OpaError {
    /// Unrecoverable internal error
    #[error("Unrecoverable internal error")]
    Internal,

    /// Invalid value type was encountered
    #[error("Invalid value type was encountered")]
    InvalidType,

    /// Invalid object path reference
    #[error("Invalid object path reference")]
    InvalidPath,

    /// Unrecognized error code
    #[error("Unrecognized error code: {0}")]
    Other(i32),
}

impl OpaError {
    /// Convert an error code to an `OpaError`
    pub(crate) fn from_code(code: i32) -> Result<(), OpaError> {
        match code {
            0 => Ok(()),
            1 => Err(Self::Internal),
            2 => Err(Self::InvalidType),
            3 => Err(Self::InvalidPath),
            x => Err(Self::Other(x)),
        }
    }
}

/// Represents the ABI version of a WASM OPA module
#[derive(Debug, Clone, Copy)]
pub enum AbiVersion {
    /// Version 1.0
    V1_0,

    /// Version 1.1
    V1_1,

    /// Version 1.2
    V1_2,

    /// Version >1.2, <2.0
    V1_2Plus(i32),
}

impl AbiVersion {
    /// Get the ABI version out of an instanciated WASM policy
    ///
    /// # Errors
    ///
    /// Returns an error if the WASM module lacks ABI version information
    pub(crate) fn from_instance<T: Send>(
        mut store: impl AsContextMut<Data = T>,
        instance: &Instance,
    ) -> Result<Self> {
        let abi_version = instance
            .get_global(&mut store, "opa_wasm_abi_version")
            .context("missing global opa_wasm_abi_version")?
            .get(&mut store)
            .i32()
            .context("opa_wasm_abi_version is not an i32")?;

        let abi_minor_version = instance
            .get_global(&mut store, "opa_wasm_abi_minor_version")
            .context("missing global opa_wasm_abi_minor_version")?
            .get(&mut store)
            .i32()
            .context("opa_wasm_abi_minor_version is not an i32")?;

        Self::new(abi_version, abi_minor_version)
    }

    /// Create a new ABI version out of the minor and major version numbers.
    fn new(major: i32, minor: i32) -> Result<Self> {
        match (major, minor) {
            (1, 0) => Ok(Self::V1_0),
            (1, 1) => Ok(Self::V1_1),
            (1, 2) => Ok(Self::V1_2),
            (1, n @ 2..) => Ok(Self::V1_2Plus(n)),
            (major, minor) => bail!("unsupported ABI version {}.{}", major, minor),
        }
    }

    /// Check if this ABI version has support for the `eval` fastpath
    #[must_use]
    pub(crate) const fn has_eval_fastpath(self) -> bool {
        matches!(self, Self::V1_2 | Self::V1_2Plus(_))
    }
}

impl std::fmt::Display for AbiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AbiVersion::V1_0 => write!(f, "1.0"),
            AbiVersion::V1_1 => write!(f, "1.1"),
            AbiVersion::V1_2 => write!(f, "1.2"),
            AbiVersion::V1_2Plus(n) => write!(f, "1.{n} (1.2+ compatible)"),
        }
    }
}
