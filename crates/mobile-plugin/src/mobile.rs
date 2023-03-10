use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PluginApi, PluginHandle},
  AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "android")]
// const PLUGIN_IDENTIFIER: &str = "com.plugin.sample";
const PLUGIN_IDENTIFIER: &str = "com.cirrus.plugin";

#[cfg(target_os = "ios")]
extern "C" {
    fn init_plugin_sample(webview: tauri::cocoa::base::id);
}

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<MobilePlugin<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "CirrusPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_sample)?;

    Ok(MobilePlugin(handle))
}

/// A helper class to access the sample APIs.
pub struct MobilePlugin<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> MobilePlugin<R> {
    // pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    //   self
    //     .0
    //     .run_mobile_plugin("ping", payload)
    //     .map_err(Into::into)
    // }

    pub fn set_player_status(&self, payload: SetPlayerStatusRequest) -> crate::Result<SetPlayerStatusResponse> {
        self
          .0
          .run_mobile_plugin("setPlayerStatus", payload)
          .map_err(Into::into)
      }
}
  