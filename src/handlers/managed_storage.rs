use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;

use crate::config::environment::{env_config, path_config};
use crate::errors::InfraError;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::managed_storage_transfer::{
    build_inventory, create_validated_zip, ensure_process_success, extract_zip_snapshot,
    inventory_diff, is_reparse_point, ManagedStorageTransition,
};
use crate::handlers::ps_script::{PsRunner, ScriptRunner};
use crate::handlers::wsl_runtime_targets::{resolve_project, TargetResolution};
use crate::models::project::{ManagedNativeStorage, Project, StorageMode};

pub const MANAGED_STORAGE_DISTRO: &str = "WSDD-Storage";
pub const MANAGED_STORAGE_ROOT: &str = "/home/wsdd/projects";
const MANAGED_STORAGE_IMAGE: &str = "debian:13-slim";
const MANAGED_STORAGE_CONTAINER: &str = "wsdd-storage-rootfs";

/// Returns whether the project storage mode can use Linux runtime permission repair.
pub fn supports_wsl_permissions(mode: StorageMode) -> bool {
    matches!(
        mode,
        StorageMode::NativeLinuxProject | StorageMode::ManagedNativeLinuxStorage
    )
}

/// Repairs writable runtime directories for a WSL-backed project and verifies Apache access.
pub fn fix_project_permissions(
    project: &Project,
    default_wsl_distro: Option<&str>,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let target = match resolve_project(project, default_wsl_distro) {
        TargetResolution::Target(target) => target,
        TargetResolution::NotWslDependent => {
            return Err(InfraError::PrerequisiteNotMet(
                "La reparacion de permisos solo aplica a proyectos WSL".to_string(),
            ));
        }
        TargetResolution::UnresolvedNative { reason, .. } => {
            return Err(InfraError::PrerequisiteNotMet(reason));
        }
    };
    let container_name = format!(
        "WSDD-Web-Server-{}",
        project.php_version.runtime_container_tag()
    );
    let group_id = container_www_data_gid(&container_name, runner)?;
    ensure_acl_available(
        &target.distro,
        project.storage_mode == StorageMode::ManagedNativeLinuxStorage,
        runner,
    )?;

    let _ = tx.send(LogLine::info(format!(
        "[Permissions] Reparando directorios runtime de '{}'...",
        project.name
    )));
    let script = render_permissions_script(&target.linux_path, group_id);
    let command = encoded_shell_command(&script);
    let output = runner.run_direct_sync(
        "wsl.exe",
        &[
            "-d",
            &target.distro,
            "-u",
            "root",
            "--",
            "sh",
            "-lc",
            &command,
        ],
        None,
        None,
    )?;
    let output = ensure_process_success("WSL runtime permissions", output)?;
    if output.text.contains("WSDD_NO_RUNTIME_TARGETS") {
        let _ = tx.send(LogLine::warn(format!(
            "[Permissions] No se encontraron directorios runtime conocidos en '{}'",
            project.name
        )));
        return Ok(());
    }

    verify_container_runtime_permissions(project, &container_name, runner)?;
    let _ = tx.send(LogLine::success(format!(
        "[Permissions] Permisos runtime validados para '{}' ✓",
        project.name
    )));
    Ok(())
}

fn container_www_data_gid(container_name: &str, runner: &PsRunner) -> Result<u32, InfraError> {
    let output = runner.run_direct_sync(
        env_config().docker_exe(),
        &[
            "exec",
            container_name,
            "sh",
            "-lc",
            "getent group www-data | cut -d: -f3",
        ],
        None,
        None,
    )?;
    let output = ensure_process_success("resolve www-data group", output)?;
    output.text.trim().parse::<u32>().map_err(|error| {
        InfraError::UnexpectedOutput(
            "resolve www-data group".to_string(),
            format!("GID invalido '{}': {error}", output.text.trim()),
        )
    })
}

fn ensure_acl_available(distro: &str, managed: bool, runner: &PsRunner) -> Result<(), InfraError> {
    let check = runner.run_direct_sync(
        "wsl.exe",
        &["-d", distro, "--", "sh", "-lc", "command -v setfacl"],
        None,
        None,
    )?;
    if check.success {
        return Ok(());
    }
    if !managed {
        return Err(InfraError::PrerequisiteNotMet(format!(
            "La distro WSL '{distro}' necesita el paquete 'acl' para reparar permisos"
        )));
    }
    let install = runner.run_direct_sync(
        "wsl.exe",
        &[
            "-d",
            distro,
            "-u",
            "root",
            "--",
            "sh",
            "-lc",
            "apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq acl",
        ],
        None,
        None,
    )?;
    ensure_process_success("install WSL acl support", install)?;
    Ok(())
}

fn render_permissions_script(linux_path: &str, group_id: u32) -> String {
    let root = sh_single_quote(linux_path);
    format!(
        "set -eu; root={root}; test -d \"$root\"; \
         targets=$(find \"$root\" \
           \\( -path \"$root/.git\" -o -path '*/.git/*' -o -path '*/vendor' -o -path '*/vendor/*' -o -path '*/node_modules' -o -path '*/node_modules/*' \\) -prune -o \
           -type d \\( -name storage -o -name var -o -name cache -o -name logs -o -path '*/bootstrap/cache' \\) -print); \
         if [ -z \"$targets\" ]; then echo WSDD_NO_RUNTIME_TARGETS; exit 0; fi; \
         printf '%s\n' \"$targets\" | while IFS= read -r dir; do \
           chgrp -R {group_id} \"$dir\"; \
           chmod -R g+rwX \"$dir\"; \
           find \"$dir\" -type d -exec chmod g+s {{}} +; \
           setfacl -R -m g:{group_id}:rwX \"$dir\"; \
           setfacl -R -d -m g:{group_id}:rwX \"$dir\"; \
           probe=\"$dir/.wsdd-permission-probe\"; : > \"$probe\"; rm -f \"$probe\"; \
         done"
    )
}

fn encoded_shell_command(script: &str) -> String {
    let payload = base64::engine::general_purpose::STANDARD.encode(script.as_bytes());
    format!("echo {payload} | base64 -d | sh")
}

fn verify_container_runtime_permissions(
    project: &Project,
    container_name: &str,
    runner: &PsRunner,
) -> Result<(), InfraError> {
    let container_root = format!("/var/www/html/{}", project.domain);
    let root = sh_single_quote(&container_root);
    let script = format!(
        "set -eu; root={root}; \
         targets=$(find \"$root\" \
           \\( -path \"$root/.git\" -o -path '*/.git/*' -o -path '*/vendor' -o -path '*/vendor/*' -o -path '*/node_modules' -o -path '*/node_modules/*' \\) -prune -o \
           -type d \\( -name storage -o -name var -o -name cache -o -name logs -o -path '*/bootstrap/cache' \\) -print); \
         [ -n \"$targets\" ] || exit 0; \
         printf '%s\n' \"$targets\" | while IFS= read -r dir; do \
           probe=\"$dir/.wsdd-permission-probe\"; : > \"$probe\"; rm -f \"$probe\"; \
         done"
    );
    let output = runner.run_direct_sync(
        env_config().docker_exe(),
        &[
            "exec",
            "--user",
            "www-data",
            container_name,
            "sh",
            "-lc",
            &script,
        ],
        None,
        None,
    )?;
    ensure_process_success("verify www-data runtime permissions", output)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedStoragePlan {
    pub distro: String,
    pub project_slug: String,
    pub linux_path: String,
    pub windows_unc: String,
    pub windows_alias: Option<String>,
}

impl ManagedStoragePlan {
    pub fn metadata(&self) -> ManagedNativeStorage {
        ManagedNativeStorage {
            distro: self.distro.clone(),
            linux_path: self.linux_path.clone(),
            windows_unc: self.windows_unc.clone(),
            windows_alias: self.windows_alias.clone(),
        }
    }
}

pub fn plan_for_project(project_name: &str, windows_alias: Option<&str>) -> ManagedStoragePlan {
    let project_slug = managed_project_slug(project_name);
    let linux_path = format!("{MANAGED_STORAGE_ROOT}/{project_slug}");
    let windows_unc =
        format!(r"\\wsl.localhost\{MANAGED_STORAGE_DISTRO}\home\wsdd\projects\{project_slug}");
    let windows_alias = windows_alias
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(ToOwned::to_owned);

    ManagedStoragePlan {
        distro: MANAGED_STORAGE_DISTRO.to_string(),
        project_slug,
        linux_path,
        windows_unc,
        windows_alias,
    }
}

pub(super) fn prepare_project_storage(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<ManagedStorageTransition, InfraError> {
    if project.storage_mode != StorageMode::ManagedNativeLinuxStorage {
        return Ok(ManagedStorageTransition::default());
    }

    let metadata = project.managed_storage.as_ref().ok_or_else(|| {
        InfraError::PrerequisiteNotMet(
            "Managed Native Linux Storage requires project metadata".to_string(),
        )
    })?;

    ensure_distro_exists(&metadata.distro, runner)?;
    ensure_linux_project_dir(&metadata.distro, &metadata.linux_path, runner)?;
    ensure_unc_reachable(&metadata.windows_unc)?;

    if let Some(alias) = &metadata.windows_alias {
        let transition = prepare_windows_project(
            project,
            alias,
            &metadata.distro,
            &metadata.linux_path,
            &metadata.windows_unc,
            runner,
            tx,
        )?;
        let _ = tx.send(LogLine::success(format!(
            "[Storage] Alias Windows listo: {alias}"
        )));
        let _ = tx.send(LogLine::success(format!(
            "[Storage] Ruta Linux administrada lista: {}",
            metadata.linux_path
        )));
        return Ok(transition);
    } else {
        let _ = tx.send(LogLine::info(format!(
            "[Storage] Proyecto accesible via {}",
            metadata.windows_unc
        )));
    }

    let _ = tx.send(LogLine::success(format!(
        "[Storage] Ruta Linux administrada lista: {}",
        metadata.linux_path
    )));
    Ok(ManagedStorageTransition::default())
}

fn prepare_windows_project(
    project: &Project,
    alias: &str,
    distro: &str,
    linux_path: &str,
    windows_unc: &str,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<ManagedStorageTransition, InfraError> {
    let alias_path = Path::new(alias);
    if alias_path.exists() {
        let metadata = fs::symlink_metadata(alias_path).map_err(InfraError::Io)?;
        if is_reparse_point(&metadata) {
            validate_alias_inventory(alias_path, Path::new(windows_unc))?;
            return Ok(ManagedStorageTransition::default());
        }
        if !metadata.is_dir() {
            return Err(InfraError::PrerequisiteNotMet(format!(
                "La ruta Windows del proyecto no es un directorio: {alias}"
            )));
        }
    }

    let suffix = unique_suffix();
    preflight_alias(
        alias_path.parent().ok_or_else(|| {
            InfraError::PrerequisiteNotMet(format!(
                "La ruta Windows no tiene directorio padre: {alias}"
            ))
        })?,
        distro,
        &suffix,
        runner,
    )?;
    let _ = tx.send(LogLine::success(
        "[Storage] Preflight de alias Windows/UNC/WSL validado ✓",
    ));

    if !alias_path.exists() {
        ensure_windows_alias(alias, windows_unc, runner)?;
        validate_alias_inventory(alias_path, Path::new(windows_unc))?;
        return Ok(ManagedStorageTransition::default());
    }

    let source_inventory = build_inventory(alias_path)?;
    let backup_path = managed_backup_path(&project.name, &suffix);
    let _ = tx.send(LogLine::info(format!(
        "[Storage] Creando backup previo: {}",
        backup_path.display()
    )));
    let snapshot_inventory = create_validated_zip(alias_path, &backup_path, &source_inventory)?;
    let _ = tx.send(LogLine::success(format!(
        "[Storage] Backup ZIP validado ✓ {}",
        backup_path.display()
    )));

    let target_path = Path::new(windows_unc);
    let target_inventory = build_inventory(target_path)?;
    let mut temporary_linux_path = None;
    if !target_inventory.is_empty() {
        if target_inventory != snapshot_inventory {
            return Err(InfraError::PrerequisiteNotMet(format!(
                "El destino WSL contiene archivos distintos; no se sobrescribira: {windows_unc}\n{}",
                inventory_diff(&snapshot_inventory, &target_inventory, 50)
            )));
        }
        let _ = tx.send(LogLine::info(
            "[Storage] La copia WSL existente coincide con el origen; se reutiliza.",
        ));
    } else {
        let temp_linux = format!("{linux_path}.wsdd-import-{suffix}");
        let temp_unc = format!("{windows_unc}.wsdd-import-{suffix}");
        ensure_linux_project_dir(distro, &temp_linux, runner)?;
        let _ = tx.send(LogLine::info(
            "[Storage] Extrayendo snapshot ZIP hacia almacenamiento WSL temporal...",
        ));
        let copy_result = (|| -> Result<(), InfraError> {
            extract_zip_snapshot(&backup_path, Path::new(&temp_unc))?;
            let copied_inventory = build_inventory(Path::new(&temp_unc))?;
            if copied_inventory != snapshot_inventory {
                return Err(InfraError::UnexpectedOutput(
                    "managed storage copy validation".to_string(),
                    format!(
                        "El inventario SHA256 del destino temporal no coincide con el snapshot ZIP:\n{}",
                        inventory_diff(&snapshot_inventory, &copied_inventory, 50)
                    ),
                ));
            }
            Ok(())
        })();
        if let Err(error) = copy_result {
            cleanup_linux_temp(distro, &temp_linux, runner);
            return Err(error);
        }
        temporary_linux_path = Some(temp_linux);
        let _ = tx.send(LogLine::success(
            "[Storage] Copia WSL validada por inventario y SHA256 ✓",
        ));
    }

    let preserved_path = preserved_original_path(alias_path, &suffix)?;
    fs::rename(alias_path, &preserved_path).map_err(InfraError::Io)?;
    let mut transition = ManagedStorageTransition::default();
    transition.record_original_preserved(alias_path, &preserved_path);
    let _ = tx.send(LogLine::success(format!(
        "[Storage] Carpeta original preservada: {}",
        preserved_path.display()
    )));

    let finish = (|| -> Result<(), InfraError> {
        if let Some(temp_linux) = temporary_linux_path {
            promote_linux_copy(distro, &temp_linux, linux_path, runner)?;
        }
        ensure_windows_alias(alias, windows_unc, runner)?;
        validate_alias_inventory(alias_path, target_path)?;
        let final_inventory = build_inventory(target_path)?;
        if final_inventory != snapshot_inventory {
            return Err(InfraError::UnexpectedOutput(
                "managed storage final validation".to_string(),
                format!(
                    "El inventario SHA256 final no coincide con el snapshot ZIP:\n{}",
                    inventory_diff(&snapshot_inventory, &final_inventory, 50)
                ),
            ));
        }
        Ok(())
    })();

    if let Err(error) = finish {
        if let Err(rollback_error) = transition.rollback() {
            return Err(InfraError::UnexpectedOutput(
                "managed storage rollback".to_string(),
                format!("{error}; rollback adicional fallo: {rollback_error}"),
            ));
        }
        return Err(error);
    }

    Ok(transition)
}

fn preflight_alias(
    alias_parent: &Path,
    distro: &str,
    suffix: &str,
    runner: &PsRunner,
) -> Result<(), InfraError> {
    let linux_path = format!("{MANAGED_STORAGE_ROOT}/.wsdd-preflight-{suffix}");
    let windows_unc =
        format!(r"\\wsl.localhost\{distro}\home\wsdd\projects\.wsdd-preflight-{suffix}");
    let alias = alias_parent.join(format!(".wsdd-preflight-{suffix}"));
    ensure_linux_project_dir(distro, &linux_path, runner)?;

    let result = (|| -> Result<(), InfraError> {
        ensure_windows_alias(&alias.to_string_lossy(), &windows_unc, runner)?;
        let probe = alias.join("probe.txt");
        fs::write(&probe, b"wsdd-storage-preflight").map_err(InfraError::Io)?;
        let unc_probe = Path::new(&windows_unc).join("probe.txt");
        if fs::read(&unc_probe).map_err(InfraError::Io)? != b"wsdd-storage-preflight" {
            return Err(InfraError::UnexpectedOutput(
                "managed storage preflight".to_string(),
                "La lectura por UNC no coincide con la escritura por alias".to_string(),
            ));
        }
        let command = format!(
            "test \"$(cat {}/probe.txt)\" = wsdd-storage-preflight",
            sh_single_quote(&linux_path)
        );
        let output = runner.run_direct_sync(
            "wsl.exe",
            &["-d", distro, "--", "sh", "-lc", &command],
            None,
            None,
        )?;
        ensure_process_success("managed storage WSL preflight", output)?;
        Ok(())
    })();

    if alias.exists() {
        let _ = fs::remove_dir(&alias);
    }
    let cleanup = format!("rm -rf -- {}", sh_single_quote(&linux_path));
    let _ = runner.run_direct_sync(
        "wsl.exe",
        &["-d", distro, "--", "sh", "-lc", &cleanup],
        None,
        None,
    );
    result
}

fn promote_linux_copy(
    distro: &str,
    temporary_linux_path: &str,
    final_linux_path: &str,
    runner: &PsRunner,
) -> Result<(), InfraError> {
    let temp = sh_single_quote(temporary_linux_path);
    let final_path = sh_single_quote(final_linux_path);
    let command = format!(
        "test -d {temp} && \
         if [ -e {final_path} ]; then test -d {final_path} && test -z \"$(ls -A {final_path})\" && rmdir {final_path}; fi && \
         mv {temp} {final_path} && test -d {final_path}"
    );
    let output = runner.run_direct_sync(
        "wsl.exe",
        &["-d", distro, "--", "sh", "-lc", &command],
        None,
        None,
    )?;
    ensure_process_success("managed storage promote", output)?;
    Ok(())
}

fn cleanup_linux_temp(distro: &str, linux_path: &str, runner: &PsRunner) {
    if !linux_path.starts_with(&format!("{MANAGED_STORAGE_ROOT}/"))
        || !linux_path.contains(".wsdd-import-")
    {
        return;
    }
    let command = format!("rm -rf -- {}", sh_single_quote(linux_path));
    let _ = runner.run_direct_sync(
        "wsl.exe",
        &["-d", distro, "--", "sh", "-lc", &command],
        None,
        None,
    );
}

fn validate_alias_inventory(alias: &Path, target: &Path) -> Result<(), InfraError> {
    let alias_inventory = build_inventory(alias)?;
    let target_inventory = build_inventory(target)?;
    if alias_inventory == target_inventory {
        Ok(())
    } else {
        Err(InfraError::UnexpectedOutput(
            "managed storage alias validation".to_string(),
            "El alias Windows y el destino UNC no muestran el mismo inventario".to_string(),
        ))
    }
}

fn managed_backup_path(project_name: &str, suffix: &str) -> PathBuf {
    path_config()
        .environment_root()
        .join("backups")
        .join("managed-storage")
        .join(format!("{project_name}-{suffix}.zip"))
}

fn preserved_original_path(alias: &Path, suffix: &str) -> Result<PathBuf, InfraError> {
    let name = alias.file_name().ok_or_else(|| {
        InfraError::PrerequisiteNotMet(format!(
            "No se puede preservar una ruta sin nombre: {}",
            alias.display()
        ))
    })?;
    let preserved =
        alias.with_file_name(format!("{}.wsdd-original-{suffix}", name.to_string_lossy()));
    if preserved.exists() {
        return Err(InfraError::PrerequisiteNotMet(format!(
            "La ruta de preservacion ya existe: {}",
            preserved.display()
        )));
    }
    Ok(preserved)
}

fn unique_suffix() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{}-{timestamp}", std::process::id())
}

pub fn provision_managed_storage(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    if managed_distro_exists(MANAGED_STORAGE_DISTRO, runner)? {
        let _ = tx.send(LogLine::info(format!(
            "[Storage] Distro WSL administrada ya existe: {MANAGED_STORAGE_DISTRO}"
        )));
    } else {
        import_managed_distro_from_debian_slim(runner, tx)?;
    }

    ensure_linux_project_dir(MANAGED_STORAGE_DISTRO, MANAGED_STORAGE_ROOT, runner)?;
    let root_unc = format!(r"\\wsl.localhost\{MANAGED_STORAGE_DISTRO}\home\wsdd\projects");
    ensure_unc_reachable(&root_unc)?;
    let _ = tx.send(LogLine::success(format!(
        "[Storage] Managed Native Linux Storage listo: {root_unc}"
    )));
    Ok(())
}

fn ensure_distro_exists(distro: &str, runner: &PsRunner) -> Result<(), InfraError> {
    if managed_distro_exists(distro, runner)? {
        return Ok(());
    }

    Err(InfraError::PrerequisiteNotMet(format!(
        "La distro WSL administrada '{distro}' no existe. Crea/importa WSDD-Storage antes de usar Managed Native Linux Storage."
    )))
}

fn managed_distro_exists(distro: &str, runner: &PsRunner) -> Result<bool, InfraError> {
    let out = runner.run_direct_sync("wsl.exe", &["-l", "-q"], None, None)?;
    if wsl_list_contains_distro(&out.text, distro) {
        return Ok(true);
    }

    let probe = runner.run_direct_sync("wsl.exe", &["-d", distro, "--", "true"], None, None)?;
    Ok(probe.success)
}

fn import_managed_distro_from_debian_slim(
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let paths = path_config();
    let cache_dir = paths.environment_root().join("cache");
    let wsl_root = paths.environment_root().join("wsl");
    let install_dir = wsl_root.join(MANAGED_STORAGE_DISTRO);
    let rootfs_tar = cache_dir.join("debian-13-slim-rootfs.tar");

    if install_dir.exists() && install_dir.read_dir()?.next().is_some() {
        return Err(InfraError::PrerequisiteNotMet(format!(
            "La carpeta WSL administrada ya existe y no esta vacia: {}",
            install_dir.display()
        )));
    }

    std::fs::create_dir_all(&cache_dir)?;
    std::fs::create_dir_all(&install_dir)?;
    if rootfs_tar.exists() {
        std::fs::remove_file(&rootfs_tar)?;
    }

    let docker = env_config().docker_exe();
    let rootfs_path = rootfs_tar.display().to_string();
    let install_path = install_dir.display().to_string();

    let _ = tx.send(LogLine::info(format!(
        "[Storage] Descargando rootfs {MANAGED_STORAGE_IMAGE}..."
    )));
    ensure_process_success(
        "docker pull managed storage image",
        runner.run_direct_sync(docker, &["pull", MANAGED_STORAGE_IMAGE], None, None)?,
    )?;

    let _ = runner.run_direct_sync(docker, &["rm", "-f", MANAGED_STORAGE_CONTAINER], None, None);
    ensure_process_success(
        "docker create managed storage rootfs",
        runner.run_direct_sync(
            docker,
            &[
                "create",
                "--name",
                MANAGED_STORAGE_CONTAINER,
                MANAGED_STORAGE_IMAGE,
            ],
            None,
            None,
        )?,
    )?;
    ensure_process_success(
        "docker export managed storage rootfs",
        runner.run_direct_sync(
            docker,
            &["export", "-o", &rootfs_path, MANAGED_STORAGE_CONTAINER],
            None,
            None,
        )?,
    )?;
    ensure_process_success(
        "docker rm managed storage rootfs",
        runner.run_direct_sync(docker, &["rm", MANAGED_STORAGE_CONTAINER], None, None)?,
    )?;

    let _ = tx.send(LogLine::info(format!(
        "[Storage] Importando distro WSL {MANAGED_STORAGE_DISTRO}..."
    )));
    ensure_process_success(
        "wsl import managed storage",
        runner.run_direct_sync(
            "wsl.exe",
            &[
                "--import",
                MANAGED_STORAGE_DISTRO,
                &install_path,
                &rootfs_path,
                "--version",
                "2",
            ],
            None,
            None,
        )?,
    )?;

    let _ = tx.send(LogLine::success(format!(
        "[Storage] Distro WSL creada desde {MANAGED_STORAGE_IMAGE}: {MANAGED_STORAGE_DISTRO}"
    )));
    Ok(())
}

fn ensure_linux_project_dir(
    distro: &str,
    linux_path: &str,
    runner: &PsRunner,
) -> Result<(), InfraError> {
    let quoted_path = sh_single_quote(linux_path);
    let script = format!(
        "mkdir -p {quoted_path} && \
         if command -v useradd >/dev/null 2>&1; then id -u wsdd >/dev/null 2>&1 || useradd -m -s /bin/sh wsdd; chown -R wsdd:wsdd /home/wsdd 2>/dev/null || true; fi && \
         test -d {quoted_path}"
    );
    let out = runner.run_direct_sync(
        "wsl.exe",
        &["-d", distro, "--", "sh", "-lc", &script],
        None,
        None,
    )?;

    if out.success {
        Ok(())
    } else {
        Err(InfraError::UnexpectedOutput(
            "wsl mkdir".to_string(),
            out.text,
        ))
    }
}

fn ensure_unc_reachable(windows_unc: &str) -> Result<(), InfraError> {
    if Path::new(windows_unc).is_dir() {
        Ok(())
    } else {
        Err(InfraError::PrerequisiteNotMet(format!(
            "La ruta UNC administrada no es accesible desde Windows: {windows_unc}"
        )))
    }
}

fn ensure_windows_alias(
    alias: &str,
    target_unc: &str,
    runner: &PsRunner,
) -> Result<(), InfraError> {
    let command = format!(
        "$alias = {alias}; $target = {target}; \
         $parent = Split-Path -Parent -LiteralPath $alias; \
         if ($parent -and -not (Test-Path -LiteralPath $parent)) {{ New-Item -ItemType Directory -Path $parent -Force | Out-Null }}; \
         if (Test-Path -LiteralPath $alias) {{ \
             $item = Get-Item -LiteralPath $alias -Force; \
             if (-not ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {{ throw \"Alias path exists and is not a reparse point: $alias\" }} \
         }} else {{ \
             New-Item -ItemType SymbolicLink -Path $alias -Target $target -Force | Out-Null \
         }}; \
         if (-not (Test-Path -LiteralPath $alias)) {{ throw \"Alias was not created: $alias\" }}",
        alias = ps_single_quote(alias),
        target = ps_single_quote(target_unc)
    );
    let output = runner.run_ps_sync(&command, None, None)?;
    ensure_process_success("managed storage Windows alias", output)?;
    Ok(())
}

pub fn managed_project_slug(name: &str) -> String {
    let mut slug = String::new();
    for ch in name.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if (ch == '-' || ch == '_' || ch.is_whitespace()) && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "project".to_string()
    } else {
        slug
    }
}

fn sh_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

fn ps_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn wsl_list_contains_distro(output: &str, distro: &str) -> bool {
    let normalized = output.replace(['\0', '\u{feff}'], "");
    normalized
        .lines()
        .map(str::trim)
        .any(|line| line.eq_ignore_ascii_case(distro))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_plan_uses_wsdd_storage_contract() {
        let plan = plan_for_project("My App", Some(r"D:\Web\my-app"));

        assert_eq!(plan.distro, "WSDD-Storage");
        assert_eq!(plan.project_slug, "my-app");
        assert_eq!(plan.linux_path, "/home/wsdd/projects/my-app");
        assert_eq!(
            plan.windows_unc,
            r"\\wsl.localhost\WSDD-Storage\home\wsdd\projects\my-app"
        );
        assert_eq!(plan.windows_alias.as_deref(), Some(r"D:\Web\my-app"));
    }

    #[test]
    fn managed_project_slug_is_filesystem_safe() {
        assert_eq!(managed_project_slug(" Test PHP 83 "), "test-php-83");
        assert_eq!(managed_project_slug("CRM_App"), "crm-app");
        assert_eq!(managed_project_slug("!!!"), "project");
    }

    #[test]
    fn quotes_shell_values_safely() {
        assert_eq!(sh_single_quote("/home/wsdd/a'b"), r"'/home/wsdd/a'\''b'");
        assert_eq!(ps_single_quote(r"D:\O'Brien"), r"'D:\O''Brien'");
    }

    #[test]
    fn detects_wsl_distro_from_nul_padded_output() {
        let output = "d\0o\0c\0k\0e\0r\0-\0d\0e\0s\0k\0t\0o\0p\0\n\0\n\0W\0S\0D\0D\0-\0S\0t\0o\0r\0a\0g\0e\0\n\0";

        assert!(wsl_list_contains_distro(output, "WSDD-Storage"));
    }

    #[test]
    fn permissions_are_available_only_for_wsl_projects() {
        assert!(supports_wsl_permissions(StorageMode::NativeLinuxProject));
        assert!(supports_wsl_permissions(
            StorageMode::ManagedNativeLinuxStorage
        ));
        assert!(!supports_wsl_permissions(StorageMode::ClassicBind));
        assert!(!supports_wsl_permissions(StorageMode::FastSyncWorkspace));
    }

    #[test]
    fn permission_script_targets_runtime_directories_and_excludes_dependencies() {
        let script = render_permissions_script("/home/wsdd/projects/demo", 33);

        for token in [
            ".git",
            "vendor",
            "node_modules",
            "bootstrap/cache",
            "-name storage",
            "-name var",
            "-name cache",
            "-name logs",
        ] {
            assert!(script.contains(token), "missing token: {token}");
        }
    }

    #[test]
    fn permission_script_uses_group_setgid_default_acl_and_runtime_probe() {
        let script = render_permissions_script("/home/wsdd/projects/demo", 33);

        assert!(script.contains("chgrp -R 33"));
        assert!(script.contains("chmod -R g+rwX"));
        assert!(script.contains("chmod g+s"));
        assert!(script.contains("setfacl -R -m g:33:rwX"));
        assert!(script.contains("setfacl -R -d -m g:33:rwX"));
        assert!(script.contains(".wsdd-permission-probe"));
    }

    #[test]
    fn encoded_wsl_script_preserves_shell_quotes() {
        let script = "root='/home/wsdd/projects/demo'; find \"$root\" -type d";
        let command = encoded_shell_command(script);

        assert!(command.starts_with("echo "));
        assert!(command.ends_with(" | base64 -d | sh"));
        assert!(!command.contains("\"$root\""));
    }
}
