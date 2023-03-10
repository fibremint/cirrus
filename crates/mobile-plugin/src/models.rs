// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct PingRequest {
  pub value: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PingResponse {
  pub value: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SetPlayerStatusRequest {
  pub is_playing: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SetPlayerStatusResponse {
  pub is_playing: Option<bool>,
}
