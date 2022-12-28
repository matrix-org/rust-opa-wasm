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

//! Builtins related to base64 encoding and decoding

use base64::{
    alphabet::URL_SAFE,
    engine::fast_portable::{FastPortable, NO_PAD},
};

const URL_SAFE_NO_PAD: FastPortable = FastPortable::from(&URL_SAFE, NO_PAD);

/// Serializes the input string into base64url encoding without padding.
#[tracing::instrument]
pub fn encode_no_pad(x: String) -> String {
    base64::encode_engine(&x, &URL_SAFE_NO_PAD)
}
