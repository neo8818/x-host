pub mod backup_service;
pub mod commands;
pub mod diagnostic_log;
pub mod dns_service;
pub mod elevation;
pub mod hosts_parser;
pub mod hosts_service;
pub mod models;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(error) = elevation::ensure_admin_startup() {
        eprintln!("startup elevation failed: {error}");
    }

    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::ensure_admin,
            commands::get_hosts_path,
            commands::get_hosts_text,
            commands::save_hosts_text,
            commands::list_entries,
            commands::add_entry,
            commands::delete_entry,
            commands::toggle_entry,
            commands::create_backup,
            commands::list_backups,
            commands::restore_backup,
            commands::flush_dns,
            commands::sync_remote_hosts,
            commands::append_diagnostic_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
