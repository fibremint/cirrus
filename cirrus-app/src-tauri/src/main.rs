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
