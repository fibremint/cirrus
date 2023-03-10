mod models;
mod error;

use tauri::{
    Runtime,
    plugin::{TauriPlugin, Builder}, 
    Invoke, 
    AppHandle, Manager, Window,
};

pub use error::*;
pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

#[cfg(desktop)]
use desktop::MobilePlugin;
#[cfg(mobile)]
use mobile::MobilePlugin;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the Cirrus Core APIs.
pub trait MobilePluginExt<R: Runtime> {
    fn mobile_plugin(&self) -> &MobilePlugin<R>;
}

impl<R: Runtime, T: Manager<R>> crate::MobilePluginExt<R> for T {
    fn mobile_plugin(&self) -> &MobilePlugin<R> {
      self.state::<MobilePlugin<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("sample")
      .setup(|app, api| {
        #[cfg(mobile)]
        let sample = mobile::init(app, api)?;
        #[cfg(desktop)]
        let sample = desktop::init(app, api)?;

        // let sample = mobile::init(app, api)?;

        app.manage(sample);
  
        Ok(())
      })
      .build()
  }