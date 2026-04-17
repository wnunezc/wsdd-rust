use std::process::Command;

use crate::config::environment::env_config;

use super::pwsh::{supported_pwsh_executable, which_pwsh};

/// Launches a program without capturing output.
pub fn launch(program: &str, args: &[&str], work_dir: Option<&str>) {
    let mut cmd = Command::new(program);
    cmd.args(args);
    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }
    if let Err(e) = cmd.spawn() {
        tracing::warn!(program, error = %e, "Could not launch program");
    }
}

/// Opens a URL with the system default handler.
pub fn launch_url(url: &str) {
    #[cfg(windows)]
    {
        use windows::core::PCWSTR;
        use windows::Win32::UI::Shell::ShellExecuteW;
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let verb: Vec<u16> = "open".encode_utf16().chain(std::iter::once(0)).collect();
        let target: Vec<u16> = url.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            let result = ShellExecuteW(
                None,
                PCWSTR(verb.as_ptr()),
                PCWSTR(target.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );

            if result.0 as usize <= 32 {
                tracing::warn!(url, code = result.0 as usize, "Could not open URL");
            }
        }
    }

    #[cfg(not(windows))]
    {
        launch("xdg-open", &[url], None);
    }
}

/// Opens an interactive PowerShell window and keeps it open after the command exits.
pub fn launch_shell_window(command: &str) {
    let program = supported_pwsh_executable().unwrap_or_else(|| {
        if which_pwsh() {
            env_config().pwsh_exe().to_string()
        } else {
            env_config().windows_powershell_exe().to_string()
        }
    });

    if let Err(e) = Command::new(program)
        .args(["-NoLogo", "-NoProfile", "-NoExit", "-Command", command])
        .spawn()
    {
        tracing::warn!(command, error = %e, "Could not open PowerShell window");
    }
}
