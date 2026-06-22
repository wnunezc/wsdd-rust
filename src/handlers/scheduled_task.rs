use std::path::Path;

use crate::config::environment::path_config;
use crate::errors::InfraError;
use crate::handlers::project;
use crate::handlers::ps_script::{PsRunner, ScriptRunner};
use crate::handlers::setting::AppSettings;
use crate::handlers::wsl_runtime_targets::has_wsl_dependent_projects;
use crate::models::project::Project;

pub const TASK_NAME: &str = "WSDD WSL Runtime Recovery";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryTaskDecision {
    InstallOrUpdate,
    KeepForExistingProjects,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryTaskStatus {
    Installed,
    Missing,
    NotRequired,
    NeedsUpdate,
}

pub fn recovery_task_decision(
    managed_enabled: bool,
    projects: &[Project],
    default_wsl_distro: Option<&str>,
) -> RecoveryTaskDecision {
    if managed_enabled {
        return RecoveryTaskDecision::InstallOrUpdate;
    }
    if has_wsl_dependent_projects(projects, default_wsl_distro) {
        RecoveryTaskDecision::KeepForExistingProjects
    } else {
        RecoveryTaskDecision::Remove
    }
}

pub fn render_task_xml(launcher: &Path, script: &Path, user_id: &str) -> String {
    let launcher = xml_escape(&launcher.display().to_string());
    let script = xml_escape(&script.display().to_string());
    let user_id = xml_escape(user_id);
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.4" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>Recovers WSDD PHP runtimes that depend on WSL-backed storage.</Description>
    <URI>{task_name}</URI>
  </RegistrationInfo>
  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>
      <Delay>PT30S</Delay>
      <UserId>{user_id}</UserId>
    </LogonTrigger>
    <SessionStateChangeTrigger>
      <Enabled>true</Enabled>
      <StateChange>SessionUnlock</StateChange>
      <UserId>{user_id}</UserId>
    </SessionStateChangeTrigger>
    <EventTrigger>
      <Enabled>true</Enabled>
      <Subscription>&lt;QueryList&gt;&lt;Query Id=&quot;0&quot; Path=&quot;System&quot;&gt;&lt;Select Path=&quot;System&quot;&gt;*[System[Provider[@Name='Service Control Manager'] and EventID=7036] and EventData[Data='com.docker.service' or Data='Docker Desktop Service']]&lt;/Select&gt;&lt;/Query&gt;&lt;/QueryList&gt;</Subscription>
      <Delay>PT5S</Delay>
    </EventTrigger>
    <CalendarTrigger>
      <StartBoundary>2026-01-01T00:00:00</StartBoundary>
      <Enabled>true</Enabled>
      <Repetition>
        <Interval>PT2M</Interval>
        <Duration>P1D</Duration>
        <StopAtDurationEnd>false</StopAtDurationEnd>
      </Repetition>
      <ScheduleByDay>
        <DaysInterval>1</DaysInterval>
      </ScheduleByDay>
    </CalendarTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>{user_id}</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>true</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <IdleSettings>
      <StopOnIdleEnd>false</StopOnIdleEnd>
      <RestartOnIdle>false</RestartOnIdle>
    </IdleSettings>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>true</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <WakeToRun>false</WakeToRun>
    <ExecutionTimeLimit>PT10M</ExecutionTimeLimit>
    <Priority>7</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{launcher}</Command>
      <Arguments>--recover-wsl-runtime &quot;{script}&quot;</Arguments>
      <WorkingDirectory>{working_dir}</WorkingDirectory>
    </Exec>
  </Actions>
</Task>
"#,
        task_name = TASK_NAME,
        working_dir = xml_escape(&path_config().scripts_dir().display().to_string()),
    )
}

pub fn install_or_update(runner: &PsRunner) -> Result<(), InfraError> {
    let paths = path_config();
    let script = paths.runtime_recovery_script();
    if !script.exists() {
        return Err(InfraError::PrerequisiteNotMet(format!(
            "Recovery worker is missing: {}",
            script.display()
        )));
    }
    let launcher = recovery_launcher_path()?;
    let user_id = current_user_id()?;
    let xml = render_task_xml(&launcher, &script, &user_id);
    let xml_path = paths.runtime_recovery_task_xml();
    write_utf16_xml(&xml_path, &xml)?;
    let xml_arg = xml_path.display().to_string();
    let out = runner.run_direct_sync(
        "schtasks.exe",
        &["/Create", "/TN", TASK_NAME, "/XML", &xml_arg, "/F"],
        None,
        None,
    )?;
    if !out.success {
        return Err(InfraError::UnexpectedOutput(
            "schtasks create".to_string(),
            out.text,
        ));
    }
    if !exists(runner)? {
        return Err(InfraError::UnexpectedOutput(
            "schtasks query".to_string(),
            "Task was not found after registration".to_string(),
        ));
    }
    Ok(())
}

pub fn remove_if_present(runner: &PsRunner) -> Result<(), InfraError> {
    if !exists(runner)? {
        return Ok(());
    }
    let out = runner.run_direct_sync(
        "schtasks.exe",
        &["/Delete", "/TN", TASK_NAME, "/F"],
        None,
        None,
    )?;
    if out.success {
        Ok(())
    } else {
        Err(InfraError::UnexpectedOutput(
            "schtasks delete".to_string(),
            out.text,
        ))
    }
}

pub fn exists(runner: &PsRunner) -> Result<bool, InfraError> {
    let out = runner.run_direct_sync("schtasks.exe", &["/Query", "/TN", TASK_NAME], None, None)?;
    Ok(out.success)
}

pub fn status(runner: &PsRunner, required: bool) -> Result<RecoveryTaskStatus, InfraError> {
    if !required {
        return Ok(RecoveryTaskStatus::NotRequired);
    }
    if !exists(runner)? {
        return Ok(RecoveryTaskStatus::Missing);
    }
    let out = runner.run_direct_sync(
        "schtasks.exe",
        &["/Query", "/TN", TASK_NAME, "/XML"],
        None,
        None,
    )?;
    if !out.success {
        return Ok(RecoveryTaskStatus::Missing);
    }
    let script = path_config()
        .runtime_recovery_script()
        .display()
        .to_string();
    let launcher = recovery_launcher_path()?.display().to_string();
    if task_xml_is_current(&out.text, &launcher, &script) {
        Ok(RecoveryTaskStatus::Installed)
    } else {
        Ok(RecoveryTaskStatus::NeedsUpdate)
    }
}

fn task_xml_is_current(xml: &str, launcher: &str, script: &str) -> bool {
    xml.contains(script)
        && xml.contains(launcher)
        && xml.contains("PT2M")
        && xml.contains("--recover-wsl-runtime")
        && xml.contains("<RunLevel>HighestAvailable</RunLevel>")
        && xml.contains("Service Control Manager")
        && xml.contains("com.docker.service")
}

pub fn run_now(runner: &PsRunner) -> Result<(), InfraError> {
    let out = runner.run_direct_sync("schtasks.exe", &["/Run", "/TN", TASK_NAME], None, None)?;
    if out.success {
        Ok(())
    } else {
        Err(InfraError::UnexpectedOutput(
            "schtasks run".to_string(),
            out.text,
        ))
    }
}

pub fn reconcile(
    runner: &PsRunner,
    settings: &AppSettings,
) -> Result<RecoveryTaskDecision, InfraError> {
    let projects = project::list_all()?;
    let decision = recovery_task_decision(
        settings.managed_native_linux_storage_enabled,
        &projects,
        settings.wsl_distro.as_deref(),
    );
    match decision {
        RecoveryTaskDecision::InstallOrUpdate | RecoveryTaskDecision::KeepForExistingProjects => {
            let current = status(runner, true)?;
            if matches!(
                current,
                RecoveryTaskStatus::Missing | RecoveryTaskStatus::NeedsUpdate
            ) {
                install_or_update(runner)?;
            }
        }
        RecoveryTaskDecision::Remove => remove_if_present(runner)?,
    }
    Ok(decision)
}

fn current_user_id() -> Result<String, InfraError> {
    let username = std::env::var("USERNAME").map_err(|error| {
        InfraError::PrerequisiteNotMet(format!("USERNAME is not available: {error}"))
    })?;
    let domain = std::env::var("USERDOMAIN").unwrap_or_default();
    Ok(if domain.trim().is_empty() {
        username
    } else {
        format!(r"{domain}\{username}")
    })
}

fn recovery_launcher_path() -> Result<std::path::PathBuf, InfraError> {
    let current = std::env::current_exe().map_err(InfraError::Io)?;
    let directory = current.parent().ok_or_else(|| {
        InfraError::PrerequisiteNotMet("WSDD executable has no parent directory".to_string())
    })?;
    let launcher = directory.join("wsdd-launcher.exe");
    if launcher.exists() {
        Ok(launcher)
    } else {
        Err(InfraError::PrerequisiteNotMet(format!(
            "WSDD launcher is missing: {}",
            launcher.display()
        )))
    }
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn write_utf16_xml(path: &Path, xml: &str) -> Result<(), InfraError> {
    use std::io::Write;

    let mut file = std::fs::File::create(path).map_err(InfraError::Io)?;
    file.write_all(&[0xFF, 0xFE]).map_err(InfraError::Io)?;
    for unit in xml.encode_utf16() {
        file.write_all(&unit.to_le_bytes())
            .map_err(InfraError::Io)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::project::{ManagedNativeStorage, PhpVersion, StorageMode};

    fn managed_project() -> Project {
        Project {
            name: "demo".to_string(),
            domain: "demo.dock".to_string(),
            php_version: PhpVersion::Php83,
            work_path: r"C:\WSDD-Projects\demo".to_string(),
            storage_mode: StorageMode::ManagedNativeLinuxStorage,
            managed_storage: Some(ManagedNativeStorage {
                distro: "WSDD-Storage".to_string(),
                linux_path: "/home/wsdd/projects/demo".to_string(),
                windows_unc: r"\\wsl.localhost\WSDD-Storage\home\wsdd\projects\demo".to_string(),
                windows_alias: None,
            }),
            ..Project::default()
        }
    }

    #[test]
    fn task_xml_contains_required_scheduling_contract() {
        let xml = render_task_xml(
            Path::new(r"C:\Program Files\WSDD\bin\wsdd-launcher.exe"),
            Path::new(r"C:\WSDD-Environment\PS-Script\wsdd-runtime-recovery.ps1"),
            r"DESKTOP\developer",
        );
        assert!(xml.contains("<MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>"));
        assert!(xml.contains("<LogonTrigger>"));
        assert!(xml.contains("SessionUnlock"));
        assert!(xml.contains("<EventTrigger>"));
        assert!(xml.contains("Service Control Manager"));
        assert!(xml.contains("com.docker.service"));
        assert!(xml.contains("<Interval>PT2M</Interval>"));
        assert!(xml.contains("--recover-wsl-runtime"));
        assert!(xml.contains("wsdd-launcher.exe"));
        assert!(xml.contains("<Hidden>true</Hidden>"));
        assert!(xml.contains("<RunLevel>HighestAvailable</RunLevel>"));
        assert!(!xml.contains("<RunLevel>LeastPrivilege</RunLevel>"));
    }

    #[test]
    fn task_decision_keeps_existing_wsl_projects() {
        assert_eq!(
            recovery_task_decision(false, &[managed_project()], None),
            RecoveryTaskDecision::KeepForExistingProjects
        );
        assert_eq!(
            recovery_task_decision(true, &[], None),
            RecoveryTaskDecision::InstallOrUpdate
        );
        assert_eq!(
            recovery_task_decision(false, &[], None),
            RecoveryTaskDecision::Remove
        );
    }

    #[test]
    fn task_contract_rejects_limited_privilege_xml() {
        let launcher = r"C:\Program Files\WSDD\bin\wsdd-launcher.exe";
        let script = r"C:\WSDD-Environment\PS-Script\wsdd-runtime-recovery.ps1";
        let current = format!(
            "{launcher} {script} PT2M --recover-wsl-runtime \
             <RunLevel>HighestAvailable</RunLevel> \
             Service Control Manager com.docker.service"
        );
        let outdated = current.replace("HighestAvailable", "LeastPrivilege");

        assert!(task_xml_is_current(&current, launcher, script));
        assert!(!task_xml_is_current(&outdated, launcher, script));
    }

    #[test]
    fn task_xml_is_written_as_utf16_le_with_bom() {
        let path = std::env::temp_dir().join("wsdd-task-test.xml");
        write_utf16_xml(&path, "<?xml version=\"1.0\"?><Task />").unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[..2], &[0xFF, 0xFE]);
        assert_eq!(bytes[2], b'<' as u8);
        assert_eq!(bytes[3], 0);
        let _ = std::fs::remove_file(path);
    }
}
