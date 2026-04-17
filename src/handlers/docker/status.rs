// WebStack Deployer for Docker
// Copyright (c) 2026 Walter Nunez / Icaros Net S.A
// All Rights Reserved.
//
// This software is provided for development use only.
// Unauthorized commercial use is prohibited.
//
// Redistribution and modification allowed only through
// the official GitHub repository.
//
// This software is provided AS IS, without warranty of any kind.
// The author shall not be liable for any damages.
//
// Contact: wnunez@lh-2.net
//! Docker Desktop status sampling for UI polling.

use super::types::DockerDesktopStatus;
use crate::handlers::ps_script::{PsRunner, ScriptRunner};

/// Returns Docker daemon readiness and a lightweight backend process sample.
pub fn docker_desktop_status_sync(runner: &PsRunner) -> DockerDesktopStatus {
    let daemon_ready = runner
        .run_direct_sync(
            "docker",
            &["info", "--format", "{{.ServerVersion}}"],
            None,
            None,
        )
        .map(|out| out.success && !out.text.trim().is_empty())
        .unwrap_or(false);

    let script = r#"
$procs = @(
    Get-Process -ErrorAction SilentlyContinue |
    Where-Object {
        $_.ProcessName -eq 'com.docker.backend' -or
        $_.ProcessName -eq 'Docker Desktop' -or
        $_.ProcessName -eq 'vmmemWSL' -or
        $_.ProcessName -eq 'dockerd' -or
        $_.ProcessName -eq 'docker' -or
        $_.ProcessName -like '*docker*'
    }
)
if (-not $procs) {
    Write-Output 'backend_running=false'
    Write-Output 'cpu_percent='
    Write-Output 'memory_mb='
    Write-Output 'process_count=0'
    Write-Output 'process_name='
    exit 0
}
$sampleMs = 350
$ids = @($procs | Select-Object -ExpandProperty Id)
$cpu1 = [double](($procs | Measure-Object -Property CPU -Sum).Sum)
Start-Sleep -Milliseconds $sampleMs
$procs2 = @(
    $ids |
    ForEach-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue } |
    Where-Object { $_ -ne $null }
)
if (-not $procs2) {
    Write-Output 'backend_running=false'
    Write-Output 'cpu_percent='
    Write-Output 'memory_mb='
    Write-Output 'process_count=0'
    Write-Output ('process_name=' + (($procs | Select-Object -ExpandProperty ProcessName -Unique) -join ', '))
    exit 0
}
$delta = [double](($procs2 | Measure-Object -Property CPU -Sum).Sum) - $cpu1
$memoryMb = [Math]::Round((($procs2 | Measure-Object -Property WorkingSet64 -Sum).Sum) / 1MB, 0)
$cores = [double][Environment]::ProcessorCount
$percent = [Math]::Round((($delta / ($sampleMs / 1000.0)) / $cores) * 100.0, 1)
if ($percent -lt 0) { $percent = 0 }
Write-Output 'backend_running=true'
Write-Output ('cpu_percent=' + $percent.ToString([System.Globalization.CultureInfo]::InvariantCulture))
Write-Output ('memory_mb=' + $memoryMb)
Write-Output ('process_count=' + $procs2.Count)
Write-Output ('process_name=' + (($procs2 | Select-Object -ExpandProperty ProcessName -Unique) -join ', '))
"#;

    let output = runner
        .run_ps_sync(script, None, None)
        .map(|out| out.text)
        .unwrap_or_default();

    parse_docker_desktop_status(&output, daemon_ready)
}

fn parse_docker_desktop_status(output: &str, daemon_ready: bool) -> DockerDesktopStatus {
    let mut status = DockerDesktopStatus {
        daemon_ready,
        ..DockerDesktopStatus::default()
    };

    for line in output.lines() {
        let mut parts = line.splitn(2, '=');
        let key = parts.next().unwrap_or_default().trim();
        let value = parts.next().unwrap_or_default().trim();

        match key {
            "backend_running" => {
                if value.eq_ignore_ascii_case("false") {
                    status.cpu_percent = None;
                }
            }
            "cpu_percent" => {
                status.cpu_percent = value.parse::<f32>().ok();
            }
            "memory_mb" => {
                status.memory_mb = value.parse::<u64>().ok();
            }
            "process_count" => {
                status.process_count = value.parse::<usize>().unwrap_or(0);
            }
            "process_name" => {
                if !value.is_empty() {
                    status.process_name = Some(value.to_string());
                }
            }
            _ => {}
        }
    }

    status
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_docker_desktop_status_reads_cpu_and_process() {
        let status = parse_docker_desktop_status(
            "backend_running=true\ncpu_percent=7.4\nmemory_mb=1536\nprocess_count=3\nprocess_name=com.docker.backend, vmmemWSL",
            true,
        );
        assert!(status.daemon_ready);
        assert_eq!(status.cpu_percent, Some(7.4));
        assert_eq!(status.memory_mb, Some(1536));
        assert_eq!(status.process_count, 3);
        assert_eq!(
            status.process_name.as_deref(),
            Some("com.docker.backend, vmmemWSL")
        );
    }
}
