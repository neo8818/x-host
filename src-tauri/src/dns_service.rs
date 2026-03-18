use std::process::Command;

#[cfg(windows)]
use encoding_rs::GBK;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn decode_shell_output(bytes: &[u8]) -> String {
    #[cfg(windows)]
    {
        if let Ok(text) = String::from_utf8(bytes.to_vec()) {
            return text.trim().to_string();
        }

        let (decoded, _, _) = GBK.decode(bytes);
        return decoded.trim().to_string();
    }

    #[cfg(not(windows))]
    {
        String::from_utf8_lossy(bytes).trim().to_string()
    }
}

fn run_command(command_name: &str, args: &[&str]) -> Result<std::process::Output, String> {
    let mut command = Command::new(command_name);
    command.args(args);

    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command
        .output()
        .map_err(|e| format!("Failed to execute `{command_name}`: {e}"))
}

#[cfg(windows)]
pub fn flush_dns() -> Result<String, String> {
    let output = run_command("ipconfig", &["/flushdns"])?;

    if output.status.success() {
        let stdout = decode_shell_output(&output.stdout);
        if stdout.is_empty() {
            return Ok("DNS cache flushed".to_string());
        }
        return Ok(stdout);
    }

    let stderr = decode_shell_output(&output.stderr);
    if stderr.is_empty() {
        return Err(format!(
            "Failed to flush the DNS cache. Exit code: {:?}",
            output.status.code()
        ));
    }
    Err(stderr)
}

#[cfg(target_os = "macos")]
pub fn flush_dns() -> Result<String, String> {
    let first = run_command("dscacheutil", &["-flushcache"])?;
    if !first.status.success() {
        let stderr = decode_shell_output(&first.stderr);
        return Err(if stderr.is_empty() {
            "Failed to flush the macOS DNS cache".to_string()
        } else {
            stderr
        });
    }

    let second = run_command("killall", &["-HUP", "mDNSResponder"])?;
    if !second.status.success() {
        let stderr = decode_shell_output(&second.stderr);
        return Err(if stderr.is_empty() {
            "Failed to restart mDNSResponder".to_string()
        } else {
            stderr
        });
    }

    Ok("DNS cache flushed".to_string())
}

#[cfg(target_os = "linux")]
pub fn flush_dns() -> Result<String, String> {
    let candidates: [(&str, &[&str]); 4] = [
        ("resolvectl", &["flush-caches"]),
        ("systemd-resolve", &["--flush-caches"]),
        ("service", &["nscd", "restart"]),
        ("service", &["dnsmasq", "restart"]),
    ];

    let mut last_error = None;

    for (command_name, args) in candidates {
        match run_command(command_name, args) {
            Ok(output) if output.status.success() => {
                let stdout = decode_shell_output(&output.stdout);
                return Ok(if stdout.is_empty() {
                    format!("DNS cache flushed with `{command_name}`")
                } else {
                    stdout
                });
            }
            Ok(output) => {
                let stderr = decode_shell_output(&output.stderr);
                last_error = Some(if stderr.is_empty() {
                    format!(
                        "`{command_name}` failed with exit code {:?}",
                        output.status.code()
                    )
                } else {
                    stderr
                });
            }
            Err(error) => {
                last_error = Some(error);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        "Could not find a supported DNS cache flush command on this Linux system".to_string()
    }))
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub fn flush_dns() -> Result<String, String> {
    Err("DNS cache flushing is not supported on this platform".to_string())
}
