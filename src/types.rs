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

use std::ffi::CStr;

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use wasmtime::{AsContext, AsContextMut, Instance, Memory};

#[derive(Debug, Deserialize, Clone)]
#[serde(transparent)]
pub struct EntrypointId(pub(crate) i32);

#[derive(Debug, Deserialize, Clone)]
#[serde(transparent)]
pub struct BuiltinId(pub(crate) i32);

#[derive(Debug)]
pub struct Value(pub(crate) i32);

#[derive(Debug)]
pub struct Addr(pub(crate) i32);

#[derive(Debug)]
pub struct Heap {
    pub(crate) ptr: i32,
    pub(crate) len: i32,
    pub(crate) freed: bool,
}

impl Heap {
    pub const fn end(&self) -> i32 {
        self.ptr + self.len
    }

    pub fn pages(&self) -> u64 {
        let page_size = 64 * 1024;
        let addr = self.end();
        let page = addr / page_size;
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

#[derive(Debug)]
pub struct NulStr(pub(crate) i32);

impl NulStr {
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

#[derive(Debug)]
pub struct Ctx(pub(crate) i32);

#[derive(Debug, thiserror::Error)]
pub enum OpaError {
    #[error("Unrecoverable internal error")]
    Internal,

    #[error("Invalid value type was encountered")]
    InvalidType,

    #[error("Invalid object path reference")]
    InvalidPath,

    #[error("Unrecognized error code: {0}")]
    Other(i32),
}

impl OpaError {
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
