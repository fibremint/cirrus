mod models;
mod error;

use tauri::{
    Runtime,
    plugin::{TauriPlugin, Builder}, 
    Invoke, 
    AppHandle, Manager, Window,
};

pub use error::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

#[cfg(desktop)]
use desktop::CirrusCore;
#[cfg(mobile)]
use mobile::CirrusCore;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the Cirrus Core APIs.
pub trait MobilePluginExt<R: Runtime> {
    fn cirrus_core(&self) -> &CirrusCore<R>;
}

impl<R: Runtime, T: Manager<R>> crate::MobilePluginExt<R> for T {
    fn cirrus_core(&self) -> &CirrusCore<R> {
      self.state::<CirrusCore<R>>().inner()
    }
}
  