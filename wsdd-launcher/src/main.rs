#![windows_subsystem = "windows"]

use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use semver::Version;
use serde::Deserialize;

const RELEASES_LATEST_URL: &str = "https://api.github.com/repos/wnunezc/wsdd-rust/releases/latest";
const USER_AGENT: &str = "wsdd-launcher";
const MSIEXT: &str = ".msi";
const APPLY_UPDATE_ARG: &str = "--apply-update";

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if let Some(request) = apply_update_request(&args) {
        return run_updater_mode(request);
    }

    let install_dir = install_dir()?;
    let local_wsdd = wsdd_exe_path(&install_dir);

    if !local_wsdd.exists() {
        show_error(
            "WSDD Launcher",
            "No se encontro wsdd.exe junto al launcher.",
        );
        return Err(anyhow!(
            "wsdd.exe no encontrado en {}",
            local_wsdd.display()
        ));
    }

    match check_for_update() {
        Ok(Some(update)) => {
            let body = update_prompt_text(&update);
            let answer = MessageDialog::new()
                .set_level(MessageLevel::Info)
                .set_title("WSDD Update Available")
                .set_description(&body)
                .set_buttons(MessageButtons::YesNo)
                .show();

            if answer == MessageDialogResult::Yes {
                match prepare_update(&update) {
                    Ok(msi_path) => {
                        spawn_temp_updater(&msi_path, &install_dir)?;
                        return Ok(());
                    }
                    Err(e) => {
                        show_error(
                            "WSDD Launcher",
                            &format!("No se pudo preparar la actualizacion.\n\n{e}"),
                        );
                    }
                }
            }

            launch_local_wsdd(&local_wsdd)?;
        }
        Ok(None) => {
            launch_local_wsdd(&local_wsdd)?;
        }
        Err(e) => {
            // Si falla la consulta remota, no bloquear la app principal.
            eprintln!("update check failed: {e}");
            launch_local_wsdd(&local_wsdd)?;
        }
    }

    Ok(())
}

fn apply_update_request(args: &[String]) -> Option<UpdateRequest> {
    if args.len() >= 3 && args[1] == APPLY_UPDATE_ARG {
        return Some(UpdateRequest {
            msi_path: PathBuf::from(&args[2]),
            install_dir: args.get(3).map(PathBuf::from),
        });
    }
    None
}

fn run_updater_mode(request: UpdateRequest) -> Result<()> {
    if !request.msi_path.exists() {
        return Err(anyhow!("MSI no encontrado: {}", request.msi_path.display()));
    }

    let status = Command::new("msiexec.exe")
        .args([
            "/i",
            &request.msi_path.to_string_lossy(),
            "/qn",
            "/norestart",
        ])
        .status()
        .context("No se pudo lanzar msiexec")?;

    if !status.success() {
        let exit_code = status
            .code()
            .map(|code| code.to_string())
            .unwrap_or_else(|| "desconocido".to_string());
        show_error(
            "WSDD Launcher",
            &format!(
                "La instalacion silenciosa del MSI fallo. WSDD no fue actualizado.\n\nCodigo de salida: {exit_code}"
            ),
        );
        return Err(anyhow!("msiexec devolvio exit code {exit_code}"));
    }

    let Some(local_wsdd) = resolve_installed_wsdd_path(request.install_dir.as_deref()) else {
        show_error(
            "WSDD Launcher",
            "La actualizacion termino pero wsdd.exe no fue encontrado en la ubicacion instalada despues del MSI.",
        );
        return Err(anyhow!(
            "wsdd.exe no encontrado despues de instalar {}",
            request.msi_path.display()
        ));
    };

    launch_local_wsdd(&local_wsdd)?;
    Ok(())
}

fn check_for_update() -> Result<Option<ReleaseInfo>> {
    let client = Client::builder()
        .build()
        .context("No se pudo crear cliente HTTP")?;
    let release: GitHubRelease = client
        .get(RELEASES_LATEST_URL)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .send()
        .context("No se pudo consultar GitHub Releases")?
        .error_for_status()
        .context("GitHub Releases respondio con error")?
        .json()
        .context("No se pudo parsear la respuesta de GitHub Releases")?;

    let current = Version::parse(env!("CARGO_PKG_VERSION"))
        .context("No se pudo parsear la version local del launcher")?;
    let latest = Version::parse(normalize_tag(&release.tag_name))
        .with_context(|| format!("Tag de release invalido: {}", release.tag_name))?;

    if latest <= current {
        return Ok(None);
    }

    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name.to_ascii_lowercase().ends_with(MSIEXT))
        .ok_or_else(|| anyhow!("La release no contiene un asset MSI"))?;

    Ok(Some(ReleaseInfo {
        version: latest.to_string(),
        changelog: release.body,
        msi_url: asset.browser_download_url.clone(),
    }))
}

fn prepare_update(update: &ReleaseInfo) -> Result<PathBuf> {
    let temp_dir = env::temp_dir().join("wsdd-launcher");
    fs::create_dir_all(&temp_dir).context("No se pudo crear directorio temporal del launcher")?;

    let msi_path = temp_dir.join(format!("wsdd-{}.msi", update.version));
    download_file(&update.msi_url, &msi_path)?;
    Ok(msi_path)
}

fn download_file(url: &str, destination: &Path) -> Result<()> {
    let client = Client::builder()
        .build()
        .context("No se pudo crear cliente HTTP")?;
    let mut response = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .with_context(|| format!("No se pudo descargar {url}"))?
        .error_for_status()
        .with_context(|| format!("Descarga MSI fallo desde {url}"))?;

    let mut file = File::create(destination)
        .with_context(|| format!("No se pudo crear {}", destination.display()))?;
    response
        .copy_to(&mut file)
        .with_context(|| format!("No se pudo escribir {}", destination.display()))?;

    Ok(())
}

fn spawn_temp_updater(msi_path: &Path, install_dir: &Path) -> Result<()> {
    let current = env::current_exe().context("No se pudo resolver current_exe del launcher")?;
    let temp_launcher = env::temp_dir().join("wsdd-launcher-updater.exe");
    let msi_arg = msi_path.to_string_lossy().into_owned();
    let install_arg = install_dir.to_string_lossy().into_owned();

    fs::copy(&current, &temp_launcher).with_context(|| {
        format!(
            "No se pudo copiar el launcher temporal a {}",
            temp_launcher.display()
        )
    })?;

    Command::new(&temp_launcher)
        .args([APPLY_UPDATE_ARG, &msi_arg, &install_arg])
        .spawn()
        .with_context(|| format!("No se pudo lanzar {}", temp_launcher.display()))?;

    Ok(())
}

fn launch_local_wsdd(wsdd_path: &Path) -> Result<()> {
    let install_dir = wsdd_path
        .parent()
        .ok_or_else(|| anyhow!("wsdd.exe no tiene directorio padre"))?;

    Command::new(wsdd_path)
        .current_dir(install_dir)
        .spawn()
        .with_context(|| format!("No se pudo lanzar {}", wsdd_path.display()))?;

    Ok(())
}

fn install_dir() -> Result<PathBuf> {
    let exe = env::current_exe().context("No se pudo resolver current_exe")?;
    exe.parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow!("El launcher no tiene directorio padre"))
}

fn wsdd_exe_path(install_dir: &Path) -> PathBuf {
    install_dir.join("wsdd.exe")
}

fn resolve_installed_wsdd_path(preferred_install_dir: Option<&Path>) -> Option<PathBuf> {
    let current_exe_dir = install_dir().ok();
    let fallback_dirs = install_dir_candidates_from_sources(
        preferred_install_dir,
        current_exe_dir.as_deref(),
        &program_files_install_dirs(),
    );

    fallback_dirs
        .into_iter()
        .map(|dir| wsdd_exe_path(&dir))
        .find(|wsdd_path| wsdd_path.exists())
}

fn program_files_install_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for key in ["ProgramW6432", "ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(base) = env::var_os(key) {
            push_unique_path(
                &mut dirs,
                Some(PathBuf::from(base).join("WSDD").join("bin")),
            );
        }
    }
    dirs
}

fn install_dir_candidates_from_sources(
    preferred_install_dir: Option<&Path>,
    current_exe_dir: Option<&Path>,
    program_files_dirs: &[PathBuf],
) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    push_unique_path(&mut dirs, preferred_install_dir.map(Path::to_path_buf));
    push_unique_path(&mut dirs, current_exe_dir.map(Path::to_path_buf));
    for dir in program_files_dirs {
        push_unique_path(&mut dirs, Some(dir.clone()));
    }
    dirs
}

fn push_unique_path(paths: &mut Vec<PathBuf>, candidate: Option<PathBuf>) {
    if let Some(candidate) = candidate {
        if !paths.iter().any(|existing| existing == &candidate) {
            paths.push(candidate);
        }
    }
}

fn normalize_tag(tag: &str) -> &str {
    tag.trim_start_matches('v')
}

fn update_prompt_text(update: &ReleaseInfo) -> String {
    let mut text = format!(
        "Hay una actualizacion disponible para WSDD.\n\nVersion remota: {}\nVersion local: {}\n\n",
        update.version,
        env!("CARGO_PKG_VERSION")
    );

    let changelog = update.changelog.trim();
    if changelog.is_empty() {
        text.push_str("No se recibio changelog para esta release.\n\n");
    } else {
        text.push_str("Changelog:\n");
        text.push_str(&truncate_changelog(changelog, 900));
        text.push_str("\n\n");
    }

    text.push_str("Deseas descargar el MSI e instalar la actualizacion ahora?");
    text
}

fn truncate_changelog(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    let mut truncated = input.chars().take(max_chars).collect::<String>();
    truncated.push_str("\n...");
    truncated
}

fn show_error(title: &str, description: &str) {
    let _ = MessageDialog::new()
        .set_level(MessageLevel::Error)
        .set_title(title)
        .set_description(description)
        .set_buttons(MessageButtons::Ok)
        .show();
}

#[derive(Debug, Clone)]
struct ReleaseInfo {
    version: String,
    changelog: String,
    msi_url: String,
}

#[derive(Debug, Clone)]
struct UpdateRequest {
    msi_path: PathBuf,
    install_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_update_request_accepts_install_dir_argument() {
        let args = vec![
            "wsdd-launcher.exe".to_string(),
            APPLY_UPDATE_ARG.to_string(),
            r"C:\Temp\wsdd.msi".to_string(),
            r"C:\Program Files\WSDD\bin".to_string(),
        ];

        let request = apply_update_request(&args).expect("request should parse");
        assert_eq!(request.msi_path, PathBuf::from(r"C:\Temp\wsdd.msi"));
        assert_eq!(
            request.install_dir,
            Some(PathBuf::from(r"C:\Program Files\WSDD\bin"))
        );
    }

    #[test]
    fn apply_update_request_keeps_backward_compatibility() {
        let args = vec![
            "wsdd-launcher.exe".to_string(),
            APPLY_UPDATE_ARG.to_string(),
            r"C:\Temp\wsdd.msi".to_string(),
        ];

        let request = apply_update_request(&args).expect("request should parse");
        assert_eq!(request.msi_path, PathBuf::from(r"C:\Temp\wsdd.msi"));
        assert_eq!(request.install_dir, None);
    }

    #[test]
    fn install_dir_candidates_prioritize_explicit_install_dir() {
        let program_files_dirs = vec![
            PathBuf::from(r"C:\Program Files\WSDD\bin"),
            PathBuf::from(r"C:\Program Files (x86)\WSDD\bin"),
            PathBuf::from(r"C:\Program Files\WSDD\bin"),
        ];

        let dirs = install_dir_candidates_from_sources(
            Some(Path::new(r"D:\Apps\WSDD\bin")),
            Some(Path::new(r"C:\Users\user\AppData\Local\Temp")),
            &program_files_dirs,
        );

        assert_eq!(dirs[0], PathBuf::from(r"D:\Apps\WSDD\bin"));
        assert_eq!(dirs[1], PathBuf::from(r"C:\Users\user\AppData\Local\Temp"));
        assert_eq!(dirs[2], PathBuf::from(r"C:\Program Files\WSDD\bin"));
        assert_eq!(dirs[3], PathBuf::from(r"C:\Program Files (x86)\WSDD\bin"));
        assert_eq!(dirs.len(), 4);
    }
}
