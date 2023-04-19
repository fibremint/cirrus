#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::{window::WindowBuilder, App, AppHandle, RunEvent, WindowUrl};

pub type SetupHook = Box<dyn FnOnce(&mut App) -> Result<(), Box<dyn std::error::Error>> + Send>;
pub type OnEvent = Box<dyn FnMut(&AppHandle, RunEvent)>;

#[tokio::main]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
  #[allow(unused_mut)]
  let mut builder = tauri::Builder::default()
    .plugin(cirrus_tauri_plugin::init())
    .setup(move |app| {
      let mut window_builder = WindowBuilder::new(app, "main", WindowUrl::default());
      
      #[cfg(desktop)]
      {
        window_builder = window_builder
          .user_agent("Tauri API")
          .title("Tauri API Validation")
          .inner_size(1000., 800.)
          .min_inner_size(600., 400.)
          .content_protected(true);
      }

      #[cfg(target_os = "windows")]
      {
        window_builder = window_builder
          .transparent(true)
          .shadow(true)
          .decorations(false);
      }

      let window = window_builder.build().unwrap();
      
      #[cfg(debug_assertions)]
      window.open_devtools();

      Ok(())
    });

  #[allow(unused_mut)]
  let mut app = builder
    .build(tauri::tauri_build_context!())
    .expect("error while building tauri application");

  app.run(move |_app_handle, _event| {
    // #[cfg(desktop)]
    // if let RunEvent::ExitRequested { api, .. } = &_event {
    //   // Keep the event loop running even if all windows are closed
    //   // This allow us to catch system tray events when there is no window
    //   api.prevent_exit();
    // }
  });
}