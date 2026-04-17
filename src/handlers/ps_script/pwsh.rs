use std::process::{Command, Stdio};
use std::sync::OnceLock;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::config::environment::env_config;

#[derive(Clone, Debug)]
struct PwshProbe {
    program: String,
    version: String,
    supported: bool,
}

/// Returns true when a supported PowerShell 7 executable is available.
pub fn has_supported_pwsh() -> bool {
    supported_pwsh_executable().is_some()
}

/// Returns the highest detected PowerShell version, if any candidate is found.
pub fn current_pwsh_version() -> Option<String> {
    detect_pwsh_candidates()
        .into_iter()
        .next()
        .map(|probe| probe.version)
}

/// Returns the executable path/name for the highest supported PowerShell 7 candidate.
pub fn supported_pwsh_executable() -> Option<String> {
    detect_pwsh_candidates()
        .into_iter()
        .find(|probe| probe.supported)
        .map(|probe| probe.program)
}

fn detect_pwsh_candidates() -> Vec<PwshProbe> {
    let mut probes = Vec::new();

    for candidate in env_config().pwsh_candidates() {
        if let Some(probe) = probe_pwsh(&candidate) {
            if probes
                .iter()
                .any(|existing: &PwshProbe| existing.program == probe.program)
            {
                continue;
            }
            probes.push(probe);
        }
    }

    probes.sort_by(|a, b| version_key(&b.version).cmp(&version_key(&a.version)));
    probes
}

fn probe_pwsh(program: &str) -> Option<PwshProbe> {
    let mut cmd = Command::new(program);
    cmd.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "$PSVersionTable.PSVersion.ToString()",
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::null());

    #[cfg(windows)]
    cmd.creation_flags(super::CREATE_NO_WINDOW);

    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        return None;
    }

    Some(PwshProbe {
        program: program.to_string(),
        version: version.clone(),
        supported: is_supported_pwsh_version(&version),
    })
}

fn is_supported_pwsh_version(version: &str) -> bool {
    let (major, minor, _) = version_key(version);
    major > 7 || (major == 7 && minor >= 5)
}

fn version_key(version: &str) -> (u32, u32, u32) {
    let mut parts = version
        .split('.')
        .map(|part| part.trim().parse::<u32>().unwrap_or(0));

    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

pub(super) fn which_pwsh() -> bool {
    static PWSH_AVAILABLE: OnceLock<bool> = OnceLock::new();
    *PWSH_AVAILABLE.get_or_init(|| {
        let mut cmd = Command::new(env_config().pwsh_exe());
        cmd.arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        cmd.creation_flags(super::CREATE_NO_WINDOW);
        cmd.status().map(|s| s.success()).unwrap_or(false)
    })
}
