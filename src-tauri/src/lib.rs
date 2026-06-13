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

/// Determine the install root directory.
///
/// Layout convention (dev and production share the same structure):
/// ```text
/// <install_root>/
/// ├── bin/
/// │   ├── goguo       (executable)
/// │   └── mihomo
/// └── data/
///     ├── config/
///     ├── baseline/
///     └── ...
/// ```
///
/// Resolution strategy:
/// - **Debug builds** (`cargo build`): always `<project_root>/release/`.
/// - **Release builds from `target/`**: detected as dev build → same as debug.
/// - **Release builds elsewhere** (production): exe parent, then go up one level
///   if the exe is inside a `bin/` subdirectory.
fn get_install_root() -> std::path::PathBuf {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir.parent().expect("src-tauri has parent");

    // Dev mode: debug assertions enabled → always use project's release dir
    if cfg!(debug_assertions) {
        return project_root.join("release");
    }

    // Release build: resolve install root from executable location
    let exe_dir = std::env::current_exe()
        .expect("failed to get current executable path")
        .parent()
        .expect("executable has no parent directory")
        .to_path_buf();

    // If running from the project's target/ directory → dev release build
    let target_dir = project_root.join("target");
    if exe_dir.starts_with(&target_dir) {
        return project_root.join("release");
    }

    // Production: if exe is inside a bin/ subdirectory, go up one level
    if exe_dir.file_name().is_some_and(|name| name == "bin") {
        exe_dir.parent().expect("bin has parent").to_path_buf()
    } else {
        exe_dir
    }
}

/// Application entry point.
///
/// # Panics
///
/// Panics if Tauri fails to initialize or encounters a runtime error.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // F111: On WSL2, force X11 backend.
    // WSLg's Weston compositor does not resume input event delivery to
    // native Wayland clients after VM pause/resume (Recv-Q=0 confirmed).
    // XWayland (used by xeyes etc.) is unaffected.
    // Native Linux keeps default backend; SleepWakeService handles Wayland freeze.
    // This must be set before GDK initialization.
    #[cfg(target_os = "linux")]
    {
        let is_wsl = std::fs::read_to_string("/proc/version")
            .is_ok_and(|content| services::wsl_detector::parse_proc_version(&content));
        if is_wsl {
            std::env::set_var("GDK_BACKEND", "x11");
            eprintln!("[GoGuo] WSL detected, GDK_BACKEND forced to x11");
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Verify GDK_BACKEND was set before GTK init
            let gdk_backend = std::env::var("GDK_BACKEND").unwrap_or_else(|_| "not set".to_string());
            eprintln!("[GoGuo] GDK_BACKEND = {gdk_backend}");

            let install_root = get_install_root();
            let data_dir = install_root.join("data");
            eprintln!("[GoGuo] install_root = {}", install_root.display());
            eprintln!("[GoGuo] data_dir = {}", data_dir.display());
            std::fs::create_dir_all(&data_dir).ok();
            // Create shared MihomoManager for AppState and SiteRulesState
            let app_config = crate::models::config::AppConfig::default_for(install_root.clone());
            let mihomo_manager = std::sync::Arc::new(std::sync::Mutex::new(
                crate::managers::mihomo_manager::MihomoManager::new(app_config.mihomo),
            ));

            app.manage(AppState::new(&install_root, mihomo_manager.clone()).expect("app state"));
            app.manage(SiteRulesState::new(&install_root, mihomo_manager));

            // F111-T6: Start sleep/wake service on native Linux + Wayland.
            // WSL2 uses X11 backend which doesn't need this workaround.
            #[cfg(target_os = "linux")]
            {
                let is_x11_forced =
                    std::env::var("GDK_BACKEND").unwrap_or_default() == "x11";
                if !is_x11_forced {
                    match crate::services::sleep_wake::start() {
                        Ok(_svc) => {
                            eprintln!(
                                "[GoGuo] Sleep/Wake service started (native Linux + Wayland)"
                            );
                        }
                        Err(e) => eprintln!("[GoGuo] Sleep/Wake service failed: {e}"),
                    }
                } else {
                    eprintln!("[GoGuo] Sleep/Wake service skipped (X11 mode)");
                }
            }

            // F105: spawn ProxyGuard background monitoring thread
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                commands::baseline::proxy_guard_loop(app_handle);
            });

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
            commands::baseline::tauri_get_snapshot_details,
            commands::baseline::tauri_get_is_restoring,
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
            commands::site_rules::refresh_ip_cache,
            commands::site_rules::get_site_reachability,
            commands::site_rules::get_diagnosis,
            commands::site_rules::get_node_pool_status,
            commands::site_rules::override_rule,
            commands::site_rules::import_subscription,
            commands::site_rules::get_subscription_sources,
            commands::site_rules::list_site_definitions,
            commands::site_rules::lookup_site,
            commands::site_rules::tauri_create_site,
            commands::site_rules::tauri_update_site_domains,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
