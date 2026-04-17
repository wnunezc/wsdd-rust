use anyhow::{Context, Result};
use std::fs;

use crate::config::environment::{path_config, path_to_string};
use crate::handlers::log_types::LogSender;
use crate::handlers::ps_script::{PsRunner, ScriptRunner};

use super::antivirus;

pub(super) fn backup_hosts() -> Result<()> {
    let paths = path_config();
    fs::copy(paths.hosts_file(), paths.hosts_backup_file())
        .context("No se pudo crear backup de hosts")?;
    Ok(())
}

pub(super) fn capture_snapshot() -> Result<Vec<u8>> {
    fs::read(path_config().hosts_file()).context("No se pudo capturar snapshot del archivo hosts")
}

pub(super) fn write_hosts_file(content: &[u8], tx: Option<&LogSender>) -> Result<()> {
    let paths = path_config();
    if fs::write(paths.hosts_file(), content).is_ok() {
        match fs::read(paths.hosts_file()) {
            Ok(actual) if actual == content => return Ok(()),
            Ok(_) => {
                tracing::warn!(
                    "write_hosts: write() OK but read-back did not match; file may have been reverted"
                );
            }
            Err(e) => {
                tracing::warn!("write_hosts: write() OK but read-back failed: {e}");
            }
        }
    }

    let tmp = paths.hosts_temp_file();
    fs::write(&tmp, content).context("No se pudo escribir archivo hosts temporal")?;

    let runner = PsRunner::new();
    let tmp_display = path_to_string(&tmp);
    let hosts_display = path_to_string(paths.hosts_file());
    let cmd = format!("Copy-Item -Force '{}' '{}'", tmp_display, hosts_display);
    let ps_result = runner
        .run_ps_sync(&cmd, None, None)
        .map_err(|e| anyhow::anyhow!("PowerShell Copy-Item falló al actualizar hosts: {e}"));

    let _ = fs::remove_file(tmp);
    ps_result?;

    let actual =
        fs::read(paths.hosts_file()).context("No se pudo leer hosts tras write vía PowerShell")?;
    if actual != content {
        return antivirus::handle_av_block(tx);
    }

    Ok(())
}
