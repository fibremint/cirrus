use tauri::{
    Runtime, 
    plugin::{TauriPlugin, Builder, Plugin}, 
    Invoke, 
    AppHandle, Manager
};

pub mod state;
pub mod commands;

// use serde_json::Value as JsonValue;

// struct CirrusPlugin<R: Runtime> {
//     invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
// }

// impl<R: Runtime> CirrusPlugin<R> {
//     pub fn new() -> Self {
//         Self {
//             invoke_handler: Box::new(
//                 tauri::generate_handler![

//                 ]
//             )
//         }
//     }
// }

// impl<R: Runtime> Plugin<R> for CirrusPlugin<R> {
//     fn name(&self) -> &'static str {
//         "cirrus"
//     }

//     fn initialize(&mut self, app: &AppHandle<R>, config: JsonValue) -> Result<()> {
//         Ok(())
//       }
// }

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("cirrus")
        .invoke_handler(tauri::generate_handler![
            commands::load_audio,
        ])
        .setup(|app_handle| {
            app_handle.manage(state::AppState::new());
            Ok(())
        })
        .build()
}