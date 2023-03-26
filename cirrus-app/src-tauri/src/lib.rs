#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::Serialize;
use tauri::{window::WindowBuilder, App, AppHandle, RunEvent, WindowUrl};
// use tauri_plugin_sample::{PingRequest, SampleExt};
use tauri::Manager;

// #[cfg(mobile)]
// mod mobile;
// #[cfg(mobile)]
// pub use mobile::*;

#[cfg(mobile)]
use mobile_plugin::MobilePluginExt;

#[derive(Clone, Serialize)]
struct Reply {
  data: String,
}


pub type SetupHook = Box<dyn FnOnce(&mut App) -> Result<(), Box<dyn std::error::Error>> + Send>;
pub type OnEvent = Box<dyn FnMut(&AppHandle, RunEvent)>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(cirrus_tauri_plugin::init())
        .setup(move |app| {
            // #[cfg(desktop)]
            // tray::create_try(app)?;

            let mut window_builder = WindowBuilder::new(app, "main", WindowUrl::default());
            #[cfg(desktop)]
            {
                window_builder = window_builder
                    .user_agent("Tauri API")
                    .title("Cirrus")
                    .inner_size(1000., 800.)
                    .min_inner_size(600., 400.)
            }

            #[cfg(target_os = "windows")]
            {
                window_builder = window_builder
                    .transparent(true)
                    .shadow(true)
                    // .decorations(false);
            }

            let window = window_builder.build().unwrap();

            #[cfg(debug_assertions)]
            window.open_devtools();

            // let value = Some("test".to_string());
            // let response = app.sample().ping(PingRequest {
            //   value: value.clone(),
            // });
            // println!("got response: {:?}", response);
            // if let Ok(res) = response {
            //   assert_eq!(res.value, value);
            // }
            #[cfg(desktop)]
            std::thread::spawn(|| {
                let server = match tiny_http::Server::http("localhost:3003") {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
                };
                loop {
                if let Ok(mut request) = server.recv() {
                    let mut body = Vec::new();
                    let _ = request.as_reader().read_to_end(&mut body);
                    let response = tiny_http::Response::new(
                    tiny_http::StatusCode(200),
                    request.headers().to_vec(),
                    std::io::Cursor::new(body),
                    request.body_length(),
                    None,
                    );
                    let _ = request.respond(response);
                }
                }
            });

            Ok(())
        })
        .on_page_load(|window, _| {
            let window_ = window.clone();
            window.listen("js-event", move |event| {
                println!("got js-event with message '{:?}'", event.payload());
                let reply = Reply {
                    data: "something else".to_string(),
                };

                window_
                    .emit("rust-event", Some(reply))
                    .expect("failed to emit");
            });
        });

    #[cfg(target_os = "macos")]
    {
        builder = builder.menu(tauri::Menu::os_default("Cirrus"));
    }

    #[allow(unused_mut)]
    let mut app = builder
        .build(tauri::tauri_build_context!())
        .expect("error while building tauri application");

    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Regular);

    app.run(move |_app_handle, _event| {
        // #[cfg(desktop)]
        // if let RunEvent::ExitRequested { api, .. } = &_event {
        //     // Keep the event loop running even if all windows are closed
        //     // This allow us to catch system tray events when there is no window
        //     api.prevent_exit();
        // }
    })
}

// #[derive(Default)]
// pub struct AppBuilder {
//     setup: Option<SetupHook>,
// }

// impl AppBuilder {
//     pub fn new() -> Self {
//         Self::default()
//     }

//     #[must_use]
//     pub fn setup<F>(mut self, setup: F) -> Self
//     where
//         F: FnOnce(&mut App) -> Result<(), Box<dyn std::error::Error>> + Send + 'static,
//     {
//         self.setup.replace(Box::new(setup));
//         self
//     }

//     pub fn run(self) {
//         let setup = self.setup;
//         tauri::Builder::default()
//             .plugin(cirrus_tauri_plugin::init())
//             .setup(move |app| {
//                 if let Some(setup) = setup {
//                     (setup)(app)?;
//                 }
//                 Ok(())
//             })
//         .run(tauri::generate_context!())
//         .expect("error while running tauri application");
//     }
// }
