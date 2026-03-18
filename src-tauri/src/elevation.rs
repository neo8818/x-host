use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const ELEVATION_ATTEMPT_ENV: &str = "XHOSTS_ELEVATION_ATTEMPTED";
const ELEVATION_ATTEMPT_ARG: &str = "--xhosts-elevation-attempted";

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn should_auto_elevate() -> bool {
    cfg!(target_os = "windows") && !cfg!(debug_assertions)
}

fn relaunch_already_attempted() -> bool {
    std::env::var(ELEVATION_ATTEMPT_ENV)
        .map(|v| v == "1")
        .unwrap_or(false)
        || std::env::args().any(|arg| arg == ELEVATION_ATTEMPT_ARG)
}

fn should_try_relaunch(is_admin: bool) -> bool {
    should_auto_elevate() && !is_admin && !relaunch_already_attempted()
}

pub fn is_elevated() -> bool {
    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("powershell");
        command.args([
                "-NoProfile",
                "-Command",
                "[Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent() | ForEach-Object { $_.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator) }",
            ]);

        #[cfg(windows)]
        {
            command.creation_flags(CREATE_NO_WINDOW);
        }

        return command
            .output()
            .map(|out| {
                if !out.status.success() {
                    return false;
                }
                let text = String::from_utf8_lossy(&out.stdout).to_ascii_lowercase();
                text.contains("true")
            })
            .unwrap_or(false);
    }

    #[cfg(unix)]
    {
        return Command::new("id")
            .arg("-u")
            .output()
            .map(|out| out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == "0")
            .unwrap_or(false);
    }

    #[cfg(not(any(target_os = "windows", unix)))]
    {
        false
    }
}

pub fn relaunch_as_admin() -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Failed to read the current executable path: {e}"))?;

    let exe_str = exe
        .to_str()
        .ok_or_else(|| "The current executable path contains invalid characters".to_string())?;

    let cwd = std::env::current_dir()
        .ok()
        .and_then(|dir| dir.to_str().map(str::to_string))
        .unwrap_or_else(|| "C:\\".to_string());

    let script = format!(
        "$env:{env_key}='1'; Start-Process -FilePath '{path}' -WorkingDirectory '{cwd}' -Verb RunAs -ArgumentList '{arg}'",
        env_key = ELEVATION_ATTEMPT_ENV,
        path = exe_str.replace('"', "\""),
        cwd = cwd.replace('"', "\""),
        arg = ELEVATION_ATTEMPT_ARG,
    );

    let mut command = Command::new("powershell");
    command.args(["-NoProfile", "-Command", &script]);

    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let status = command
        .status()
        .map_err(|e| format!("Failed to relaunch with elevated privileges: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err("The user canceled or the system denied the UAC elevation request".to_string())
    }
}

pub fn ensure_admin_startup() -> Result<bool, String> {
    if !should_auto_elevate() {
        return Ok(false);
    }

    let elevated = is_elevated();
    if elevated {
        return Ok(true);
    }

    if !should_try_relaunch(elevated) {
        return Ok(false);
    }

    if relaunch_as_admin().is_ok() {
        std::process::exit(0);
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::{
        relaunch_already_attempted, should_auto_elevate, should_try_relaunch,
        ELEVATION_ATTEMPT_ARG, ELEVATION_ATTEMPT_ENV,
    };

    #[test]
    fn debug_build_should_not_auto_elevate() {
        if cfg!(debug_assertions) {
            assert!(!should_auto_elevate());
        }
    }

    #[test]
    fn should_detect_relaunch_attempt_env() {
        unsafe {
            std::env::set_var(ELEVATION_ATTEMPT_ENV, "1");
        }
        assert!(relaunch_already_attempted());
        unsafe {
            std::env::remove_var(ELEVATION_ATTEMPT_ENV);
        }
    }

    #[test]
    fn should_have_stable_relaunch_flag_arg() {
        assert_eq!(ELEVATION_ATTEMPT_ARG, "--xhosts-elevation-attempted");
    }

    #[test]
    fn should_not_try_relaunch_when_already_admin() {
        assert!(!should_try_relaunch(true));
    }
}
