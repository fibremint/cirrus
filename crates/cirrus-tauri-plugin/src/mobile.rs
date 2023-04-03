use anyhow;
use serde::de::DeserializeOwned;
use tauri::{Runtime, Manager, plugin::{PluginApi, PluginHandle}, AppHandle};

use crate::models::*;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.fibremint.cirrus";

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the Cirrus Core APIs.
pub trait CirrusMobilePluginExt<R: Runtime> {
    fn cirrus_mobile_plugin(&self) -> &CirrusMobilePlugin<R>;
}

impl<R: Runtime, T: Manager<R>> CirrusMobilePluginExt<R> for T {
    fn cirrus_mobile_plugin(&self) -> &CirrusMobilePlugin<R> {
      self.state::<CirrusMobilePlugin<R>>().inner()
    }
}

#[cfg(mobile)]
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> Result<CirrusMobilePlugin<R>, anyhow::Error> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "CirrusPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_sample)?;

    Ok(CirrusMobilePlugin(handle))
}

/// A helper class to access the sample APIs.
pub struct CirrusMobilePlugin<R: Runtime>(PluginHandle<R>);

#[cfg(mobile)]
impl<R: Runtime> CirrusMobilePlugin<R> {
    pub fn set_player_status(&self, payload: SetPlayerStatusRequest) -> Result<SetPlayerStatusResponse, anyhow::Error> {
        self
          .0
          .run_mobile_plugin("setPlayerStatus", payload)
          .map_err(Into::into)
      }
}