use std::path::PathBuf;

use tauri::State;

use crate::backup_service;
use crate::diagnostic_log::{append_log_line, content_summary, safe_log_event};
use crate::dns_service;
use crate::elevation;
use crate::hosts_service;
use crate::models::{BackupItem, HostEntry};

#[derive(Default)]
pub struct AppState {
    pub hosts_path: PathBuf,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            hosts_path: hosts_service::hosts_path(),
        }
    }
}

#[tauri::command]
pub fn ensure_admin() -> Result<bool, String> {
    let elevated = elevation::is_elevated();
    safe_log_event("backend", "ensure_admin", &format!("elevated={elevated}"));
    Ok(elevated)
}

#[tauri::command]
pub fn list_entries(state: State<'_, AppState>) -> Result<Vec<HostEntry>, String> {
    safe_log_event(
        "backend",
        "list_entries:start",
        &format!("path={}", state.hosts_path.display()),
    );
    let result = hosts_service::list_entries(&state.hosts_path);
    match &result {
        Ok(entries) => safe_log_event(
            "backend",
            "list_entries:ok",
            &format!(
                "count={} path={}",
                entries.len(),
                state.hosts_path.display()
            ),
        ),
        Err(error) => safe_log_event(
            "backend",
            "list_entries:error",
            &format!("path={} error={error}", state.hosts_path.display()),
        ),
    }
    result
}

#[tauri::command]
pub fn get_hosts_path(state: State<'_, AppState>) -> Result<String, String> {
    let path = state.hosts_path.to_string_lossy().to_string();
    safe_log_event("backend", "get_hosts_path", &format!("path={path}"));
    Ok(path)
}

#[tauri::command]
pub fn get_hosts_text(state: State<'_, AppState>) -> Result<String, String> {
    safe_log_event(
        "backend",
        "get_hosts_text:start",
        &format!("path={}", state.hosts_path.display()),
    );
    let result = hosts_service::read_hosts_text(&state.hosts_path);
    match &result {
        Ok(content) => safe_log_event(
            "backend",
            "get_hosts_text:ok",
            &format!(
                "path={} {}",
                state.hosts_path.display(),
                content_summary(content)
            ),
        ),
        Err(error) => safe_log_event(
            "backend",
            "get_hosts_text:error",
            &format!("path={} error={error}", state.hosts_path.display()),
        ),
    }
    result
}

#[tauri::command]
pub fn save_hosts_text(state: State<'_, AppState>, content: String) -> Result<(), String> {
    safe_log_event(
        "backend",
        "save_hosts_text:start",
        &format!(
            "path={} {}",
            state.hosts_path.display(),
            content_summary(&content)
        ),
    );
    let result = hosts_service::save_hosts_text(&state.hosts_path, &content);
    match &result {
        Ok(()) => {
            let persisted = hosts_service::read_hosts_text(&state.hosts_path);
            match persisted {
                Ok(persisted_text) => safe_log_event(
                    "backend",
                    "save_hosts_text:ok",
                    &format!(
                        "path={} {}",
                        state.hosts_path.display(),
                        content_summary(&persisted_text)
                    ),
                ),
                Err(error) => safe_log_event(
                    "backend",
                    "save_hosts_text:post_read_error",
                    &format!("path={} error={error}", state.hosts_path.display()),
                ),
            }
        }
        Err(error) => safe_log_event(
            "backend",
            "save_hosts_text:error",
            &format!("path={} error={error}", state.hosts_path.display()),
        ),
    }
    result
}

#[tauri::command]
pub fn add_entry(state: State<'_, AppState>, ip: String, domain: String) -> Result<(), String> {
    safe_log_event(
        "backend",
        "add_entry:start",
        &format!(
            "path={} ip={} domain={}",
            state.hosts_path.display(),
            ip,
            domain
        ),
    );
    let result = hosts_service::add_entry(&state.hosts_path, &ip, &domain);
    match &result {
        Ok(()) => safe_log_event(
            "backend",
            "add_entry:ok",
            &format!(
                "path={} ip={} domain={}",
                state.hosts_path.display(),
                ip,
                domain
            ),
        ),
        Err(error) => safe_log_event(
            "backend",
            "add_entry:error",
            &format!(
                "path={} ip={} domain={} error={error}",
                state.hosts_path.display(),
                ip,
                domain
            ),
        ),
    }
    result
}

#[tauri::command]
pub fn delete_entry(state: State<'_, AppState>, id: String) -> Result<(), String> {
    safe_log_event(
        "backend",
        "delete_entry:start",
        &format!("path={} id={id}", state.hosts_path.display()),
    );
    let result = hosts_service::delete_entry(&state.hosts_path, &id);
    match &result {
        Ok(()) => safe_log_event(
            "backend",
            "delete_entry:ok",
            &format!("path={} id={id}", state.hosts_path.display()),
        ),
        Err(error) => safe_log_event(
            "backend",
            "delete_entry:error",
            &format!("path={} id={id} error={error}", state.hosts_path.display()),
        ),
    }
    result
}

#[tauri::command]
pub fn toggle_entry(state: State<'_, AppState>, id: String, enabled: bool) -> Result<(), String> {
    safe_log_event(
        "backend",
        "toggle_entry:start",
        &format!(
            "path={} id={id} enabled={enabled}",
            state.hosts_path.display()
        ),
    );
    let result = hosts_service::toggle_entry(&state.hosts_path, &id, enabled);
    match &result {
        Ok(()) => safe_log_event(
            "backend",
            "toggle_entry:ok",
            &format!(
                "path={} id={id} enabled={enabled}",
                state.hosts_path.display()
            ),
        ),
        Err(error) => safe_log_event(
            "backend",
            "toggle_entry:error",
            &format!(
                "path={} id={id} enabled={enabled} error={error}",
                state.hosts_path.display()
            ),
        ),
    }
    result
}

#[tauri::command]
pub fn create_backup(state: State<'_, AppState>) -> Result<String, String> {
    safe_log_event(
        "backend",
        "create_backup:start",
        &format!("path={}", state.hosts_path.display()),
    );
    let result = backup_service::create_backup(&state.hosts_path)
        .map(|path| path.to_string_lossy().to_string());
    match &result {
        Ok(path) => safe_log_event("backend", "create_backup:ok", &format!("backup={path}")),
        Err(error) => safe_log_event("backend", "create_backup:error", error),
    }
    result
}

#[tauri::command]
pub fn list_backups() -> Result<Vec<BackupItem>, String> {
    safe_log_event("backend", "list_backups:start", "");
    let result = backup_service::list_backups();
    match &result {
        Ok(items) => safe_log_event(
            "backend",
            "list_backups:ok",
            &format!("count={}", items.len()),
        ),
        Err(error) => safe_log_event("backend", "list_backups:error", error),
    }
    result
}

#[tauri::command]
pub fn restore_backup(state: State<'_, AppState>, path: String) -> Result<(), String> {
    safe_log_event(
        "backend",
        "restore_backup:start",
        &format!(
            "hosts_path={} backup_path={path}",
            state.hosts_path.display()
        ),
    );
    let result = backup_service::restore_backup(&state.hosts_path, &PathBuf::from(&path));
    match &result {
        Ok(()) => safe_log_event(
            "backend",
            "restore_backup:ok",
            &format!(
                "hosts_path={} backup_path={path}",
                state.hosts_path.display()
            ),
        ),
        Err(error) => safe_log_event(
            "backend",
            "restore_backup:error",
            &format!(
                "hosts_path={} backup_path={path} error={error}",
                state.hosts_path.display()
            ),
        ),
    }
    result
}

#[tauri::command]
pub fn flush_dns() -> Result<String, String> {
    safe_log_event("backend", "flush_dns:start", "");
    let result = dns_service::flush_dns();
    match &result {
        Ok(message) => safe_log_event("backend", "flush_dns:ok", message),
        Err(error) => safe_log_event("backend", "flush_dns:error", error),
    }
    result
}

#[tauri::command]
pub fn sync_remote_hosts(state: State<'_, AppState>, url: String) -> Result<String, String> {
    safe_log_event(
        "backend",
        "sync_remote_hosts:start",
        &format!("path={} url={url}", state.hosts_path.display()),
    );
    let result = hosts_service::sync_remote_github_hosts(&state.hosts_path, &url)
        .map(|_| "Remote hosts synced successfully".to_string());
    match &result {
        Ok(message) => {
            let persisted = hosts_service::read_hosts_text(&state.hosts_path);
            match persisted {
                Ok(content) => safe_log_event(
                    "backend",
                    "sync_remote_hosts:ok",
                    &format!(
                        "path={} url={} message={} {}",
                        state.hosts_path.display(),
                        url,
                        message,
                        content_summary(&content)
                    ),
                ),
                Err(error) => safe_log_event(
                    "backend",
                    "sync_remote_hosts:post_read_error",
                    &format!(
                        "path={} url={} error={error}",
                        state.hosts_path.display(),
                        url
                    ),
                ),
            }
        }
        Err(error) => safe_log_event(
            "backend",
            "sync_remote_hosts:error",
            &format!(
                "path={} url={} error={error}",
                state.hosts_path.display(),
                url
            ),
        ),
    }
    result
}

#[tauri::command]
pub fn append_diagnostic_log(
    component: String,
    action: String,
    details: String,
) -> Result<(), String> {
    append_log_line(&format!("[frontend] [{component}] {action} {details}"))
}
