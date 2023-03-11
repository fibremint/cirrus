#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[tokio::main]
async fn main() {
  tauri::async_runtime::set(tokio::runtime::Handle::current());
  
  tauri::Builder::default()
    .plugin(cirrus_tauri_plugin::init())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
