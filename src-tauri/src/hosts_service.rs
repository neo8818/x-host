use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use encoding_rs::GBK;
use uuid::Uuid;

use crate::backup_service::create_backup;
use crate::diagnostic_log::{content_summary, safe_log_event};
use crate::hosts_parser::{parse_hosts, render_hosts};
use crate::models::{HostEntry, HostsLine};

#[cfg(target_os = "windows")]
const DEFAULT_HOSTS_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";
#[cfg(any(target_os = "linux", target_os = "macos"))]
const DEFAULT_HOSTS_PATH: &str = "/etc/hosts";
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
const DEFAULT_HOSTS_PATH: &str = "/etc/hosts";
const GITHUB_HOSTS_START: &str = "#Github Hosts Start";
const GITHUB_HOSTS_END: &str = "#Github Hosts End";

pub fn hosts_path() -> PathBuf {
    std::env::var("XHOSTS_HOSTS_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_HOSTS_PATH))
}

fn read_lines(path: &Path) -> Result<Vec<HostsLine>, String> {
    let content = read_hosts_text(path)?;
    Ok(parse_hosts(&content))
}

fn decode_windows_text(bytes: &[u8]) -> String {
    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
        return text;
    }

    let (decoded, _, _) = GBK.decode(bytes);
    decoded.into_owned()
}

pub fn read_hosts_text(path: &Path) -> Result<String, String> {
    safe_log_event(
        "service",
        "read_hosts_text:start",
        &format!("path={}", path.display()),
    );
    let result = fs::read(path)
        .map(|bytes| decode_windows_text(&bytes))
        .map_err(|e| format!("Failed to read the hosts file: {e}"));
    match &result {
        Ok(content) => safe_log_event(
            "service",
            "read_hosts_text:ok",
            &format!("path={} {}", path.display(), content_summary(content)),
        ),
        Err(error) => safe_log_event(
            "service",
            "read_hosts_text:error",
            &format!("path={} error={error}", path.display()),
        ),
    }
    result
}

pub fn save_hosts_text(path: &Path, content: &str) -> Result<(), String> {
    safe_log_event(
        "service",
        "save_hosts_text:start",
        &format!("path={} {}", path.display(), content_summary(content)),
    );
    let result = (|| {
        create_backup(path)?;
        write_atomic(path, content)
    })();
    match &result {
        Ok(()) => safe_log_event(
            "service",
            "save_hosts_text:ok",
            &format!("path={} {}", path.display(), content_summary(content)),
        ),
        Err(error) => safe_log_event(
            "service",
            "save_hosts_text:error",
            &format!("path={} error={error}", path.display()),
        ),
    }
    result
}

fn normalize_remote_hosts_url(url: &str) -> String {
    let trimmed = url.trim();
    if let Some(rest) = trimmed.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 5 && parts[2] == "blob" {
            let owner = parts[0];
            let repo = parts[1];
            let branch = parts[3];
            let path = parts[4..].join("/");
            return format!("https://raw.githubusercontent.com/{owner}/{repo}/{branch}/{path}");
        }
    }
    trimmed.to_string()
}

fn extract_remote_block(remote_text: &str) -> Result<String, String> {
    let lines: Vec<&str> = remote_text.lines().collect();
    let start = lines
        .iter()
        .position(|line| line.trim() == GITHUB_HOSTS_START)
        .ok_or_else(|| "The remote content is missing #Github Hosts Start".to_string())?;
    let end = lines
        .iter()
        .position(|line| line.trim() == GITHUB_HOSTS_END)
        .ok_or_else(|| "The remote content is missing #Github Hosts End".to_string())?;

    if end < start {
        return Err("The GitHub hosts block in the remote content is invalid".to_string());
    }

    let mut block = lines[start..=end].join("\n");
    block.push('\n');
    Ok(block)
}

fn replace_or_append_github_block(local_text: &str, remote_block: &str) -> String {
    let lines: Vec<&str> = local_text.lines().collect();
    let start = lines
        .iter()
        .position(|line| line.trim() == GITHUB_HOSTS_START);
    let end = lines
        .iter()
        .position(|line| line.trim() == GITHUB_HOSTS_END);

    match (start, end) {
        (Some(start_index), Some(end_index)) if end_index >= start_index => {
            let before = lines[..start_index].join("\n");
            let after = if end_index + 1 < lines.len() {
                lines[end_index + 1..].join("\n")
            } else {
                String::new()
            };

            let mut out = String::new();
            if !before.is_empty() {
                out.push_str(&before);
                out.push('\n');
            }
            out.push_str(remote_block);
            if !after.is_empty() {
                out.push_str(&after);
                out.push('\n');
            }
            out
        }
        _ => {
            let mut out = local_text.to_string();
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(remote_block);
            out
        }
    }
}

pub fn sync_remote_github_hosts(path: &Path, remote_url: &str) -> Result<(), String> {
    let resolved_url = normalize_remote_hosts_url(remote_url);
    safe_log_event(
        "service",
        "sync_remote_github_hosts:start",
        &format!(
            "path={} remote_url={} resolved_url={}",
            path.display(),
            remote_url,
            resolved_url
        ),
    );

    let remote_text = reqwest::blocking::get(resolved_url.as_str())
        .map_err(|e| format!("Failed to download the remote hosts file: {e}"))?
        .text()
        .map_err(|e| format!("Failed to read the remote hosts response body: {e}"))?;
    safe_log_event(
        "service",
        "sync_remote_github_hosts:remote_text",
        &content_summary(&remote_text),
    );

    let remote_block = extract_remote_block(&remote_text)?;
    safe_log_event(
        "service",
        "sync_remote_github_hosts:remote_block",
        &content_summary(&remote_block),
    );

    let local_text = read_hosts_text(path)?;
    safe_log_event(
        "service",
        "sync_remote_github_hosts:local_before",
        &format!("path={} {}", path.display(), content_summary(&local_text)),
    );

    let merged_text = replace_or_append_github_block(&local_text, &remote_block);
    safe_log_event(
        "service",
        "sync_remote_github_hosts:merged",
        &format!("path={} {}", path.display(), content_summary(&merged_text)),
    );

    let result = save_hosts_text(path, &merged_text);
    match &result {
        Ok(()) => safe_log_event(
            "service",
            "sync_remote_github_hosts:ok",
            &format!("path={} url={}", path.display(), remote_url),
        ),
        Err(error) => safe_log_event(
            "service",
            "sync_remote_github_hosts:error",
            &format!("path={} url={} error={error}", path.display(), remote_url),
        ),
    }
    result
}

fn write_atomic(path: &Path, content: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "The hosts directory is invalid".to_string())?;
    let tmp_name = format!("hosts.tmp.{}", Uuid::new_v4());
    let tmp_path = parent.join(tmp_name);
    safe_log_event(
        "service",
        "write_atomic:start",
        &format!(
            "path={} tmp_path={} {}",
            path.display(),
            tmp_path.display(),
            content_summary(content)
        ),
    );

    let result = (|| {
        let mut file = fs::File::create(&tmp_path)
            .map_err(|e| format!("Failed to create the temporary file: {e}"))?;
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write the temporary file: {e}"))?;
        file.flush()
            .map_err(|e| format!("Failed to flush the temporary file: {e}"))?;
        fs::copy(&tmp_path, path).map_err(|e| format!("Failed to replace the hosts file: {e}"))?;
        Ok(())
    })();

    let _ = fs::remove_file(&tmp_path);
    match &result {
        Ok(()) => safe_log_event(
            "service",
            "write_atomic:ok",
            &format!("path={} tmp_path={}", path.display(), tmp_path.display()),
        ),
        Err(error) => safe_log_event(
            "service",
            "write_atomic:error",
            &format!(
                "path={} tmp_path={} error={error}",
                path.display(),
                tmp_path.display()
            ),
        ),
    }
    result
}

fn mutate_and_save<F>(path: &Path, mutator: F) -> Result<(), String>
where
    F: FnOnce(&mut Vec<HostsLine>) -> Result<(), String>,
{
    let mut lines = read_lines(path)?;
    mutator(&mut lines)?;
    let rendered = render_hosts(&lines);
    create_backup(path)?;
    write_atomic(path, &rendered)
}

pub fn list_entries(path: &Path) -> Result<Vec<HostEntry>, String> {
    let lines = read_lines(path)?;
    Ok(lines
        .into_iter()
        .filter_map(|line| match line {
            HostsLine::Managed(entry) => Some(entry),
            HostsLine::Raw(_) => None,
        })
        .collect())
}

pub fn add_entry(path: &Path, ip: &str, domain: &str) -> Result<(), String> {
    if ip.parse::<std::net::IpAddr>().is_err() {
        return Err("Invalid IP address".to_string());
    }
    if domain.is_empty() {
        return Err("The domain cannot be empty".to_string());
    }

    mutate_and_save(path, |lines| {
        let exists = lines.iter().any(|line| match line {
            HostsLine::Managed(entry) => entry.ip == ip && entry.domain == domain,
            HostsLine::Raw(_) => false,
        });
        if exists {
            return Err("This mapping already exists".to_string());
        }

        let id = format!("manual-{}", Uuid::new_v4());
        lines.push(HostsLine::Managed(HostEntry {
            id,
            ip: ip.to_string(),
            domain: domain.to_string(),
            enabled: true,
        }));
        Ok(())
    })
}

pub fn delete_entry(path: &Path, id: &str) -> Result<(), String> {
    mutate_and_save(path, |lines| {
        let before = lines.len();
        lines.retain(|line| match line {
            HostsLine::Managed(entry) => entry.id != id,
            HostsLine::Raw(_) => true,
        });

        if before == lines.len() {
            return Err("The target entry was not found".to_string());
        }
        Ok(())
    })
}

pub fn toggle_entry(path: &Path, id: &str, enabled: bool) -> Result<(), String> {
    mutate_and_save(path, |lines| {
        for line in lines {
            if let HostsLine::Managed(entry) = line {
                if entry.id == id {
                    entry.enabled = enabled;
                    return Ok(());
                }
            }
        }
        Err("The target entry was not found".to_string())
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use crate::hosts_service::{
        add_entry, extract_remote_block, list_entries, read_hosts_text,
        replace_or_append_github_block, save_hosts_text, toggle_entry,
    };

    #[test]
    fn add_and_toggle_entry() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("hosts");
        fs::write(&path, "127.0.0.1 localhost\n").expect("write hosts");

        add_entry(&path, "127.0.0.1", "example.local").expect("add entry");
        let entries = list_entries(&path).expect("list entries");
        let entry = entries
            .iter()
            .find(|x| x.domain == "example.local")
            .expect("find created entry");

        toggle_entry(&path, &entry.id, false).expect("toggle off");
        let after = list_entries(&path).expect("list after toggle");
        let toggled = after
            .iter()
            .find(|x| x.domain == "example.local")
            .expect("find toggled entry");
        assert!(!toggled.enabled);
    }

    #[test]
    fn save_and_read_raw_hosts_text() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("hosts");
        fs::write(&path, "127.0.0.1 localhost\n").expect("write hosts");

        let raw = "127.0.0.1 localhost\n127.0.0.1 api.local\n";
        save_hosts_text(&path, raw).expect("save raw hosts");

        let read_back = read_hosts_text(&path).expect("read raw hosts");
        assert_eq!(read_back, raw);
    }

    #[test]
    fn replace_existing_github_block() {
        let local = "127.0.0.1 localhost\n#Github Hosts Start\n1.1.1.1 a.com\n#Github Hosts End\n";
        let remote = "#Github Hosts Start\n2.2.2.2 b.com\n#Github Hosts End\n";
        let merged = replace_or_append_github_block(local, remote);
        assert!(merged.contains("2.2.2.2 b.com"));
        assert!(!merged.contains("1.1.1.1 a.com"));
    }

    #[test]
    fn append_github_block_when_missing() {
        let local = "127.0.0.1 localhost\n";
        let remote = "#Github Hosts Start\n2.2.2.2 b.com\n#Github Hosts End\n";
        let merged = replace_or_append_github_block(local, remote);
        assert!(merged.starts_with("127.0.0.1 localhost"));
        assert!(merged.contains("#Github Hosts Start"));
        assert!(merged.contains("#Github Hosts End"));
    }

    #[test]
    fn replace_github_block_persists_after_save_and_read() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("hosts");
        let local = "127.0.0.1 localhost\n#Github Hosts Start\n1.1.1.1 old.example.com\n#Github Hosts End\n192.168.0.1 intranet.local\n";
        fs::write(&path, local).expect("write hosts");

        let remote = "#Github Hosts Start\n2.2.2.2 new.example.com\n#Github Hosts End\n";
        let merged = replace_or_append_github_block(local, remote);
        save_hosts_text(&path, &merged).expect("save merged hosts");

        let read_back = read_hosts_text(&path).expect("read merged hosts");
        assert!(read_back.contains("2.2.2.2 new.example.com"));
        assert!(!read_back.contains("1.1.1.1 old.example.com"));
        assert_eq!(read_back.matches("#Github Hosts Start").count(), 1);
        assert_eq!(read_back.matches("#Github Hosts End").count(), 1);
    }

    #[test]
    fn append_github_block_persists_once_after_save_and_read() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("hosts");
        let local = "127.0.0.1 localhost\n192.168.0.1 intranet.local\n";
        fs::write(&path, local).expect("write hosts");

        let remote = "#Github Hosts Start\n2.2.2.2 new.example.com\n#Github Hosts End\n";
        let merged = replace_or_append_github_block(local, remote);
        save_hosts_text(&path, &merged).expect("save merged hosts");

        let read_back = read_hosts_text(&path).expect("read merged hosts");
        assert!(read_back.starts_with(local));
        assert!(read_back.contains("2.2.2.2 new.example.com"));
        assert_eq!(read_back.matches("#Github Hosts Start").count(), 1);
        assert_eq!(read_back.matches("#Github Hosts End").count(), 1);
        assert_eq!(read_back.matches("2.2.2.2 new.example.com").count(), 1);
    }

    #[test]
    fn extract_remote_block_success() {
        let remote = "aaa\n#Github Hosts Start\n1.1.1.1 a.com\n#Github Hosts End\nbbb\n";
        let block = extract_remote_block(remote).expect("extract block");
        assert!(block.starts_with("#Github Hosts Start"));
        assert!(block.contains("1.1.1.1 a.com"));
        assert!(block.ends_with("#Github Hosts End\n"));
    }
}
