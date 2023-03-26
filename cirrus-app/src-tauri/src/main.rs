// #[cfg(desktop)]
// mod desktop;

// fn main() {
//     #[cfg(desktop)]
//     desktop::main();
// }

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  #[cfg(desktop)]
  app::run();
}

// #[tokio::main]
// async fn main() {
//   tauri::async_runtime::set(tokio::runtime::Handle::current());
  
//   tauri::Builder::default()
//     .plugin(cirrus_tauri_plugin::init())
//     .run(tauri::generate_context!())
//     .expect("error while running tauri application");
// }
