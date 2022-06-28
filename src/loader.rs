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

use std::path::Path;

use anyhow::Context;
use async_compression::tokio::bufread::GzipDecoder;
use futures_util::TryStreamExt;
use tokio::io::{AsyncBufRead, AsyncReadExt, BufReader};
use tokio_tar::Archive;
use tracing::{info_span, Instrument};

/// Read an OPA compiled bundle from disk
#[tracing::instrument(err)]
pub async fn read_bundle(path: impl AsRef<Path> + std::fmt::Debug) -> anyhow::Result<Vec<u8>> {
    let file = tokio::fs::File::open(path).await?;
    let reader = BufReader::new(file);
    load_bundle(reader).await
}

/// Load an OPA compiled bundle
#[tracing::instrument(skip_all, err)]
pub async fn load_bundle(
    reader: impl AsyncBufRead + Unpin + Send + Sync,
) -> anyhow::Result<Vec<u8>> {
    // Wrap the reader in a gzip decoder, then in a tar unarchiver
    let reader = GzipDecoder::new(reader);
    let mut archive = Archive::new(reader);

    // Go through the archive entries to find the /policy.wasm one
    let entries = archive.entries()?;
    let mut entry = entries
        .try_filter(|e| {
            std::future::ready(
                e.path()
                    .map(|p| p.as_os_str() == "/policy.wasm")
                    .unwrap_or(false),
            )
        })
        .try_next()
        .instrument(info_span!("find_bundle_entry"))
        .await?
        .context("could not find WASM policy in tar archive")?;

    // Once we found it, read it completely to a buffer
    let mut buf = Vec::new();
    entry
        .read_to_end(&mut buf)
        .instrument(info_span!("read_module"))
        .await?;

    Ok(buf)
}
