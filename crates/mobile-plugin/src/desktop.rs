use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<CirrusCore<R>> {
  Ok(CirrusCore(app.clone()))
}

/// A helper class to access the sample APIs.
pub struct CirrusCore<R: Runtime>(AppHandle<R>);

impl<R: Runtime> CirrusCore<R> {
    pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
        Ok(PingResponse {
            value: payload.value,
        })
    }
}
