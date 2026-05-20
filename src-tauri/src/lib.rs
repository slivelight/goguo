pub mod adapters;
pub mod commands;
pub mod engines;
pub mod managers;
pub mod models;
pub mod services;
pub mod storage;

/// Application entry point.
///
/// # Panics
///
/// Panics if Tauri fails to initialize or encounters a runtime error.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
