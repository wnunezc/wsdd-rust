use std::path::Path;
use std::process::{Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::config::environment::env_config;
use crate::errors::InfraError;
use crate::handlers::log_types::OutputSender;

use super::pwsh::{supported_pwsh_executable, which_pwsh};
use super::types::ProcOutput;

pub(super) fn exec_powershell(
    command: &str,
    work_dir: Option<&Path>,
    tx: Option<&OutputSender>,
) -> Result<ProcOutput, InfraError> {
    let ps_exe = if let Some(ps_exe) = supported_pwsh_executable() {
        ps_exe
    } else if which_pwsh() {
        env_config().pwsh_exe().to_string()
    } else {
        env_config().windows_powershell_exe().to_string()
    };

    let cmd_with_encoding = format!(
        "$OutputEncoding = [System.Text.Encoding]::UTF8 ; \
         [Console]::OutputEncoding = [System.Text.Encoding]::UTF8 ; \
         {command}"
    );

    exec_direct(
        &ps_exe,
        &[
            "-NoLogo".to_string(),
            "-NoProfile".to_string(),
            "-NonInteractive".to_string(),
            "-Command".to_string(),
            cmd_with_encoding,
        ],
        work_dir,
        tx,
    )
}

pub(super) fn exec_direct(
    program: &str,
    args: &[String],
    work_dir: Option<&Path>,
    tx: Option<&OutputSender>,
) -> Result<ProcOutput, InfraError> {
    if let Some(sender) = tx {
        return exec_direct_streaming(program, args, work_dir, sender);
    }

    let mut cmd = Command::new(program);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

    #[cfg(windows)]
    cmd.creation_flags(super::CREATE_NO_WINDOW);

    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }

    let child = cmd
        .spawn()
        .map_err(|e| InfraError::ProcessNotFound(format!("{program}: {e}")))?;
    let output = child.wait_with_output().map_err(InfraError::Io)?;

    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&output.stderr));
    let text = merge_streams(stdout, stderr);

    tracing::trace!(
        program,
        success = output.status.success(),
        output_len = text.len(),
        "Process completed"
    );

    Ok(ProcOutput {
        text,
        success: output.status.success(),
    })
}

fn exec_direct_streaming(
    program: &str,
    args: &[String],
    work_dir: Option<&Path>,
    tx: &OutputSender,
) -> Result<ProcOutput, InfraError> {
    use std::io::{BufRead, BufReader};

    let mut cmd = Command::new(program);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

    #[cfg(windows)]
    cmd.creation_flags(super::CREATE_NO_WINDOW);

    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| InfraError::ProcessNotFound(format!("{program}: {e}")))?;

    let stderr_pipe = child
        .stderr
        .take()
        .ok_or_else(|| InfraError::Io(std::io::Error::other("stderr pipe unavailable")))?;
    let stdout_pipe = child
        .stdout
        .take()
        .ok_or_else(|| InfraError::Io(std::io::Error::other("stdout pipe unavailable")))?;

    let tx_err = tx.clone();
    let stderr_handle = std::thread::spawn(move || {
        let mut text = String::new();
        let mut reader = BufReader::new(stderr_pipe);
        let mut buf = Vec::new();
        loop {
            buf.clear();
            match reader.read_until(b'\n', &mut buf) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    let raw = String::from_utf8_lossy(&buf);
                    let clean = strip_ansi(raw.trim_end_matches(['\n', '\r']));
                    if !clean.trim().is_empty() {
                        let _ = tx_err.send(clean.clone());
                        text.push_str(&clean);
                        text.push('\n');
                    }
                }
            }
        }
        text
    });

    let mut stdout_text = String::new();
    let mut reader = BufReader::new(stdout_pipe);
    let mut buf = Vec::new();
    loop {
        buf.clear();
        match reader.read_until(b'\n', &mut buf) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let raw = String::from_utf8_lossy(&buf);
                let clean = strip_ansi(raw.trim_end_matches(['\n', '\r']));
                if !clean.trim().is_empty() {
                    let _ = tx.send(clean.to_string());
                    stdout_text.push_str(&clean);
                    stdout_text.push('\n');
                }
            }
        }
    }

    let stderr_text = stderr_handle.join().unwrap_or_default();
    let status = child.wait().map_err(InfraError::Io)?;

    tracing::trace!(
        program,
        success = status.success(),
        "Streaming process completed"
    );

    Ok(ProcOutput {
        text: merge_streams(stdout_text, stderr_text),
        success: status.success(),
    })
}

/// Removes ANSI escape sequences from process output.
pub fn strip_ansi(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1B' && chars.peek() == Some(&'[') {
            chars.next();
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

fn merge_streams(stdout: String, stderr: String) -> String {
    let has_out = !stdout.trim().is_empty();
    let has_err = !stderr.trim().is_empty();

    match (has_out, has_err) {
        (true, true) => format!("{stdout}\n{stderr}"),
        (true, false) => stdout,
        (false, true) => stderr,
        (false, false) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_removes_color_codes() {
        let input = "\x1B[32mGreen Text\x1B[0m";
        assert_eq!(strip_ansi(input), "Green Text");
    }

    #[test]
    fn strip_ansi_preserves_plain_text() {
        let input = "Hello, World!";
        assert_eq!(strip_ansi(input), "Hello, World!");
    }

    #[test]
    fn strip_ansi_handles_empty() {
        assert_eq!(strip_ansi(""), "");
    }

    #[test]
    fn merge_streams_both_empty() {
        assert_eq!(merge_streams(String::new(), String::new()), "");
    }

    #[test]
    fn merge_streams_only_stdout() {
        assert_eq!(merge_streams("out".to_string(), String::new()), "out");
    }

    #[test]
    fn merge_streams_both_present() {
        let result = merge_streams("out".to_string(), "err".to_string());
        assert!(result.contains("out"));
        assert!(result.contains("err"));
    }
}
