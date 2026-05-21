pub mod adapters;
pub mod commands;
pub mod engines;
pub mod managers;
pub mod models;
pub mod services;
pub mod storage;

use commands::baseline::AppState;
use commands::site_rules::SiteRulesState;
use tauri::Manager;

/// Application entry point.
///
/// # Panics
///
/// Panics if Tauri fails to initialize or encounters a runtime error.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("app data directory");
            std::fs::create_dir_all(&data_dir).ok();
            app.manage(AppState::new(&data_dir).expect("app state"));
            app.manage(SiteRulesState::new(&data_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::baseline::tauri_start_initial_assessment,
            commands::baseline::tauri_get_state_summary,
            commands::baseline::tauri_trigger_readjustment,
            commands::baseline::tauri_confirm_baseline,
            commands::baseline::tauri_get_baseline_status,
            commands::baseline::tauri_stop_service,
            commands::baseline::tauri_get_service_status,
            commands::baseline::tauri_get_recovery_progress,
            commands::baseline::tauri_get_audit_log,
            commands::baseline::tauri_detect_deployment_mode,
            commands::baseline::tauri_get_deployment_mode,
            commands::baseline::tauri_set_deployment_mode,
            #[cfg(target_os = "linux")]
            commands::baseline::tauri_get_wsl_status,
            #[cfg(target_os = "linux")]
            commands::baseline::tauri_get_network_mode,
            commands::site_rules::add_target_site,
            commands::site_rules::remove_target_site,
            commands::site_rules::apply_preset_template,
            commands::site_rules::preview_rules,
            commands::site_rules::apply_rules,
            commands::site_rules::get_site_reachability,
            commands::site_rules::get_diagnosis,
            commands::site_rules::get_node_pool_status,
            commands::site_rules::override_rule,
            commands::site_rules::import_subscription,
            commands::site_rules::get_subscription_sources,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
