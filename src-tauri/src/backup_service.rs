use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use uuid::Uuid;

use crate::models::BackupItem;

const BACKUP_DIR_ENV: &str = "XHOSTS_BACKUP_DIR";

pub fn backup_dir() -> Result<PathBuf, String> {
    std::env::var(BACKUP_DIR_ENV)
        .map(PathBuf::from)
        .or_else(|_| {
            dirs::data_local_dir()
                .ok_or_else(|| "Unable to determine the local data directory".to_string())
                .map(|base| base.join("x-hosts").join("backups"))
        })
}

pub fn create_backup(hosts_path: &Path) -> Result<PathBuf, String> {
    let dir = backup_dir()?;
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create the backup directory: {e}"))?;

    let now = Local::now();
    let file_name = format!(
        "hosts-{}-{}.bak",
        now.format("%Y%m%d-%H%M%S"),
        Uuid::new_v4()
    );
    let backup_path = dir.join(file_name);

    fs::copy(hosts_path, &backup_path).map_err(|e| format!("Failed to create the backup: {e}"))?;
    Ok(backup_path)
}

pub fn list_backups() -> Result<Vec<BackupItem>, String> {
    let dir = backup_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut items = Vec::new();
    let entries =
        fs::read_dir(&dir).map_err(|e| format!("Failed to read the backup directory: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read a backup entry: {e}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let meta = entry
            .metadata()
            .map_err(|e| format!("Failed to read backup metadata: {e}"))?;

        let created_at = meta
            .modified()
            .ok()
            .map(|t| {
                let dt: DateTime<Local> = DateTime::from(t);
                dt.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .unwrap_or_else(|| "unknown".to_string());

        let name = path
            .file_name()
            .and_then(|x| x.to_str())
            .ok_or_else(|| "Invalid backup file name".to_string())?
            .to_string();

        items.push(BackupItem {
            name,
            path: path.to_string_lossy().to_string(),
            created_at,
        });
    }

    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(items)
}

pub fn restore_backup(hosts_path: &Path, backup_path: &Path) -> Result<(), String> {
    if !backup_path.exists() {
        return Err("The backup file does not exist".to_string());
    }
    create_backup(hosts_path)?;
    fs::copy(backup_path, hosts_path).map_err(|e| format!("Failed to restore the backup: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::{Mutex, MutexGuard};

    use tempfile::tempdir;

    use super::{backup_dir, create_backup, BACKUP_DIR_ENV};

    static BACKUP_ENV_LOCK: Mutex<()> = Mutex::new(());

    struct BackupDirOverride {
        _guard: MutexGuard<'static, ()>,
    }

    impl BackupDirOverride {
        fn new(path: &Path) -> Self {
            let guard = BACKUP_ENV_LOCK.lock().expect("lock backup env");
            unsafe {
                std::env::set_var(BACKUP_DIR_ENV, path);
            }
            Self { _guard: guard }
        }
    }

    impl Drop for BackupDirOverride {
        fn drop(&mut self) {
            unsafe {
                std::env::remove_var(BACKUP_DIR_ENV);
            }
        }
    }

    #[test]
    fn backup_dir_uses_env_override_when_present() {
        let dir = tempdir().expect("tempdir");
        let _override = BackupDirOverride::new(dir.path());

        let resolved = backup_dir().expect("backup dir");
        assert_eq!(resolved, dir.path());
    }

    #[test]
    fn create_backup_generates_distinct_paths_for_back_to_back_calls() {
        let dir = tempdir().expect("tempdir");
        let _override = BackupDirOverride::new(dir.path());
        let hosts_path = dir.path().join("hosts");
        std::fs::write(&hosts_path, "127.0.0.1 localhost\n").expect("write hosts");

        let first = create_backup(&hosts_path).expect("create first backup");
        let second = create_backup(&hosts_path).expect("create second backup");

        assert_ne!(first, second);
        assert!(first.exists());
        assert!(second.exists());
    }
}
