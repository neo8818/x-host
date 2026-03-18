use std::collections::hash_map::DefaultHasher;
use std::fs::{self, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;

use chrono::Local;

const LOG_DIR_ENV: &str = "XHOSTS_LOG_DIR";
const LOG_FILE_NAME: &str = "diagnostic.log";

pub fn log_dir() -> Result<PathBuf, String> {
    std::env::var(LOG_DIR_ENV).map(PathBuf::from).or_else(|_| {
        dirs::data_local_dir()
            .ok_or_else(|| "Unable to determine the local log directory".to_string())
            .map(|base| base.join("x-hosts").join("logs"))
    })
}

pub fn log_file_path() -> Result<PathBuf, String> {
    Ok(log_dir()?.join(LOG_FILE_NAME))
}

fn fingerprint(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn content_summary(content: &str) -> String {
    let lines = content.lines().count();
    let github_start = content.matches("#Github Hosts Start").count();
    let github_end = content.matches("#Github Hosts End").count();
    let preview = content
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.chars().take(80).collect::<String>().replace('"', "'"))
        .unwrap_or_default();

    format!(
        "len={} lines={} github_start={} github_end={} hash={} first_line=\"{}\"",
        content.len(),
        lines,
        github_start,
        github_end,
        fingerprint(content),
        preview
    )
}

pub fn append_log_line(line: &str) -> Result<(), String> {
    let file_path = log_file_path()?;
    let dir = file_path
        .parent()
        .ok_or_else(|| "Invalid log directory".to_string())?;
    fs::create_dir_all(dir).map_err(|e| format!("Failed to create the log directory: {e}"))?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .map_err(|e| format!("Failed to open the log file: {e}"))?;

    writeln!(file, "{line}").map_err(|e| format!("Failed to write the log entry: {e}"))
}

pub fn log_event(component: &str, action: &str, details: &str) -> Result<(), String> {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    append_log_line(&format!("[{now}] [{component}] {action} {details}"))
}

pub fn safe_log_event(component: &str, action: &str, details: &str) {
    let _ = log_event(component, action, details);
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::{Mutex, MutexGuard};

    use tempfile::tempdir;

    use super::{append_log_line, content_summary, log_event, log_file_path, LOG_DIR_ENV};

    static LOG_ENV_LOCK: Mutex<()> = Mutex::new(());

    struct LogDirOverride {
        _guard: MutexGuard<'static, ()>,
    }

    impl LogDirOverride {
        fn new(path: &std::path::Path) -> Self {
            let guard = LOG_ENV_LOCK.lock().expect("lock log env");
            unsafe {
                std::env::set_var(LOG_DIR_ENV, path);
            }
            Self { _guard: guard }
        }
    }

    impl Drop for LogDirOverride {
        fn drop(&mut self) {
            unsafe {
                std::env::remove_var(LOG_DIR_ENV);
            }
        }
    }

    #[test]
    fn append_log_line_creates_file_and_appends() {
        let dir = tempdir().expect("tempdir");
        let _override = LogDirOverride::new(dir.path());

        append_log_line("first line").expect("append first line");
        append_log_line("second line").expect("append second line");

        let file_path = log_file_path().expect("log path");
        let content = fs::read_to_string(file_path).expect("read log file");
        assert!(content.contains("first line"));
        assert!(content.contains("second line"));
    }

    #[test]
    fn content_summary_reports_key_markers() {
        let content =
            "127.0.0.1 localhost\n#Github Hosts Start\n1.1.1.1 a.com\n#Github Hosts End\n";
        let summary = content_summary(content);
        assert!(summary.contains("len="));
        assert!(summary.contains("lines=4"));
        assert!(summary.contains("github_start=1"));
        assert!(summary.contains("github_end=1"));
    }

    #[test]
    fn log_event_writes_timestamped_entry() {
        let dir = tempdir().expect("tempdir");
        let _override = LogDirOverride::new(dir.path());

        log_event(
            "backend",
            "save_hosts_text:start",
            "path=C:/Windows/System32/drivers/etc/hosts",
        )
        .expect("write log event");

        let file_path = log_file_path().expect("log path");
        let content = fs::read_to_string(file_path).expect("read log file");
        assert!(content.contains("[backend]"));
        assert!(content.contains("save_hosts_text:start"));
        assert!(content.contains("path=C:/Windows/System32/drivers/etc/hosts"));
    }
}
