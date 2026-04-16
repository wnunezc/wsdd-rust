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
//! Motor de ejecución de scripts PowerShell y comandos de sistema.
//!
//! Equivalente a `Handlers/PSScript.cs` en la versión C#.
//!
//! # Arquitectura
//!
//! - [`ScriptRunner`]: trait (contrato) que permite reemplazar la implementación
//!   en tests sin necesidad de PowerShell real.
//! - [`PsRunner`]: implementación concreta para Windows.
//! - Las funciones `async` de módulo (`run_script`, `run_ps_command`, etc.) son
//!   wrappers convenientes que instancian [`PsRunner`] internamente.
//!
//! # Patrón de evaluación individual
//!
//! Cada script PS1 se puede evaluar de forma aislada usando las funciones
//! de los handlers específicos (e.g. `docker::probe_installed()`), que
//! ejecutan un único script y retornan un resultado tipado.
//!
//! # Streaming hacia la UI
//!
//! Las operaciones largas (docker-compose, instalaciones) aceptan un
//! `Option<OutputSender>`. Si se provee, cada línea de output se envía
//! al canal y la UI puede mostrarlas en tiempo real desde su render loop.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::config::environment::{env_config, path_config, DEFAULT_WSDD_ENV};
use crate::errors::InfraError;

// Re-exportar tipos de canal desde log_types como fachada pública.
// Permite que código externo importe desde ps_script sin conocer log_types.
#[allow(unused_imports)]
pub use crate::handlers::log_types::{LogLevel, LogLine, LogSender, OutputSender};

// ─── Constantes ───────────────────────────────────────────────────────────────

/// Directorio raíz del entorno WSDD extraído en el sistema.
pub const WSDD_ENV: &str = DEFAULT_WSDD_ENV;
pub use crate::config::environment::MIN_SUPPORTED_PWSH_VERSION;

/// Flag Win32 para crear proceso sin ventana visible.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

// ─── Tipos públicos ───────────────────────────────────────────────────────────

/// Output completo capturado de un proceso externo.
#[derive(Debug, Clone, Default)]
pub struct ProcOutput {
    /// Texto completo (stdout + stderr combinados y con ANSI limpiado).
    pub text: String,
    /// `true` si el proceso terminó con exit code 0.
    pub success: bool,
}

#[derive(Clone, Debug)]
struct PwshProbe {
    program: String,
    version: String,
    supported: bool,
}

impl ProcOutput {
    /// Verifica si el output contiene un token esperado.
    ///
    /// Usado por los handlers para interpretar resultados de scripts PS1.
    ///
    /// # Ejemplo
    /// ```rust,ignore
    /// let out = run_script("dd-isinstalled.ps1", ...).await?;
    /// if out.contains("Installed") { ... }
    /// ```
    pub fn contains(&self, token: &str) -> bool {
        self.text.contains(token)
    }
}

// OutputSender re-exportado desde log_types — ver arriba.

// ─── Trait ScriptRunner ───────────────────────────────────────────────────────

/// Contrato de ejecución de scripts y comandos de sistema.
///
/// Definir esto como trait permite:
/// 1. Desacoplar los handlers de la implementación concreta.
/// 2. Reemplazar [`PsRunner`] con un `MockRunner` en tests unitarios
///    sin necesidad de PowerShell instalado.
///
/// # Implementaciones disponibles
/// - [`PsRunner`][]: producción (Windows)
pub trait ScriptRunner: Send + Sync {
    /// Ejecuta un script `.ps1` con `-ExecutionPolicy Unrestricted`.
    ///
    /// # Errors
    /// [`InfraError::ScriptFailed`] si PowerShell no puede iniciarse o el
    /// script produce un error.
    fn run_script_sync(
        &self,
        script_name: &str,
        work_dir: Option<&Path>,
        tx: Option<&OutputSender>,
    ) -> Result<ProcOutput, InfraError>;

    /// Ejecuta un comando PowerShell arbitrario.
    ///
    /// # Errors
    /// [`InfraError::Io`] si PowerShell no se puede iniciar.
    fn run_ps_sync(
        &self,
        command: &str,
        work_dir: Option<&Path>,
        tx: Option<&OutputSender>,
    ) -> Result<ProcOutput, InfraError>;

    /// Ejecuta un proceso directo (no a través de PowerShell).
    ///
    /// Útil para docker cuando el comando no necesita el contexto de PS.
    fn run_direct_sync(
        &self,
        program: &str,
        args: &[&str],
        work_dir: Option<&Path>,
        tx: Option<&OutputSender>,
    ) -> Result<ProcOutput, InfraError>;
}

// ─── PsRunner — implementación concreta ──────────────────────────────────────

/// Ejecutor real de scripts PowerShell en Windows.
///
/// Usa `spawn_blocking` internamente para las versiones async,
/// evitando bloquear el runtime de Tokio.
#[derive(Clone, Debug)]
pub struct PsRunner {
    scripts_dir: PathBuf,
}

impl PsRunner {
    /// Crea un runner apuntando al directorio de scripts estándar de WSDD.
    pub fn new() -> Self {
        Self {
            scripts_dir: scripts_dir(),
        }
    }

    /// Crea un runner con un directorio de scripts personalizado.
    ///
    /// Útil para probar scripts en un directorio temporal durante desarrollo.
    pub fn with_scripts_dir(dir: PathBuf) -> Self {
        Self { scripts_dir: dir }
    }
}

impl Default for PsRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptRunner for PsRunner {
    fn run_script_sync(
        &self,
        script_name: &str,
        work_dir: Option<&Path>,
        tx: Option<&OutputSender>,
    ) -> Result<ProcOutput, InfraError> {
        let dir = work_dir.unwrap_or(&self.scripts_dir);
        let path = dir.join(script_name);
        let ps_exe = supported_pwsh_executable().ok_or_else(|| {
            let current = current_pwsh_version().unwrap_or_else(|| "not found".to_string());
            InfraError::PrerequisiteNotMet(format!(
                "PowerShell {MIN_SUPPORTED_PWSH_VERSION}+ is required to run PS1 scripts (found: {current})"
            ))
        })?;

        tracing::debug!(script = script_name, dir = %dir.display(), "Ejecutando script PS1");

        let policy = "Set-ExecutionPolicy -Scope Process -ExecutionPolicy Unrestricted -Force";
        let invoke = format!("& '{}'", path.display());
        let ps_arg = format!(
            "$OutputEncoding = [System.Text.Encoding]::UTF8 ; \
             [Console]::OutputEncoding = [System.Text.Encoding]::UTF8 ; \
             {policy} ; {invoke}"
        );

        exec_direct(
            &ps_exe,
            &[
                "-NoLogo".to_string(),
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-Command".to_string(),
                ps_arg,
            ],
            Some(dir),
            tx,
        )
        .map_err(|e| InfraError::ScriptFailed(script_name.to_string(), e.to_string()))
    }

    fn run_ps_sync(
        &self,
        command: &str,
        work_dir: Option<&Path>,
        tx: Option<&OutputSender>,
    ) -> Result<ProcOutput, InfraError> {
        tracing::debug!(command = %command, "Ejecutando comando PowerShell");

        let policy = "Set-ExecutionPolicy -Scope Process -ExecutionPolicy Unrestricted -Force";
        let ps_arg = format!("{policy} ; {command}");
        exec_powershell(&ps_arg, work_dir, tx)
    }

    fn run_direct_sync(
        &self,
        program: &str,
        args: &[&str],
        work_dir: Option<&Path>,
        tx: Option<&OutputSender>,
    ) -> Result<ProcOutput, InfraError> {
        tracing::debug!(program = program, ?args, "Ejecutando proceso directo");

        let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        exec_direct(program, &args_owned, work_dir, tx)
    }
}

// ─── Rutas del entorno ────────────────────────────────────────────────────────

/// Directorio raíz del entorno WSDD: `C:\WSDD-Environment\`
pub fn env_dir() -> PathBuf {
    path_config().environment_root().to_path_buf()
}

/// Directorio de scripts PS1: `C:\WSDD-Environment\PS-Script\`
pub fn scripts_dir() -> PathBuf {
    path_config().scripts_dir()
}

/// Directorio de la estructura Docker: `C:\WSDD-Environment\Docker-Structure\`
pub fn docker_structure_dir() -> PathBuf {
    path_config().docker_structure_dir()
}

// ─── API async pública ────────────────────────────────────────────────────────

/// Ejecuta un script PS1 del directorio de scripts WSDD de forma asíncrona.
///
/// Equivalente a `PSScript.Script()` en C#.
///
/// # Errors
/// [`InfraError::ScriptFailed`] si PowerShell no puede iniciarse o el script falla.
pub async fn run_script(
    script_name: &str,
    work_dir: Option<PathBuf>,
    tx: Option<OutputSender>,
) -> Result<ProcOutput, InfraError> {
    let name = script_name.to_string();
    let runner = PsRunner::new();

    tokio::task::spawn_blocking(move || {
        runner.run_script_sync(&name, work_dir.as_deref(), tx.as_ref())
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
}

/// Ejecuta un comando PowerShell arbitrario de forma asíncrona.
///
/// Equivalente a `PSScript.Command("powershell", cmd, ...)` en C#.
///
/// # Errors
/// [`InfraError::Io`] si PowerShell no se puede iniciar.
pub async fn run_ps_command(
    command: &str,
    work_dir: Option<PathBuf>,
    tx: Option<OutputSender>,
) -> Result<ProcOutput, InfraError> {
    let cmd = command.to_string();
    let runner = PsRunner::new();

    tokio::task::spawn_blocking(move || runner.run_ps_sync(&cmd, work_dir.as_deref(), tx.as_ref()))
        .await
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
}

/// Ejecuta docker directamente sin pasar por PowerShell.
///
/// Más eficiente para comandos docker simples de consulta.
///
/// # Errors
/// [`InfraError::ProcessNotFound`] si `docker` no está en PATH.
pub async fn run_docker(
    args: Vec<String>,
    tx: Option<OutputSender>,
) -> Result<ProcOutput, InfraError> {
    tokio::task::spawn_blocking(move || {
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let runner = PsRunner::new();
        runner.run_direct_sync(env_config().docker_exe(), &refs, None, tx.as_ref())
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
}

// ─── Lanzamiento sin captura ──────────────────────────────────────────────────

/// Lanza un programa sin capturar output (para abrir Explorer, TTY, etc.)
///
/// Equivalente a `PSScript.InvokeProgram()` en C#.
pub fn launch(program: &str, args: &[&str], work_dir: Option<&str>) {
    let mut cmd = Command::new(program);
    cmd.args(args);
    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }
    if let Err(e) = cmd.spawn() {
        tracing::warn!(program, error = %e, "No se pudo lanzar el programa");
    }
}

/// Abre una URL con el handler predeterminado del sistema sin parpadeo de consola.
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
                tracing::warn!(url, code = result.0 as usize, "No se pudo abrir la URL");
            }
        }
    }

    #[cfg(not(windows))]
    {
        launch("xdg-open", &[url], None);
    }
}

/// Lanza PowerShell con ventana visible y sin salir al terminar (`-NoExit`).
///
/// Equivalente a `PSScript.InvokeShellProgram()` en C#.
/// Usado para abrir sesiones TTY a contenedores Docker.
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
        tracing::warn!(command, error = %e, "No se pudo abrir ventana PowerShell");
    }
}

// ─── Utilidades ───────────────────────────────────────────────────────────────

/// Elimina secuencias de escape ANSI del texto.
///
/// Los outputs de Docker y PowerShell pueden contener códigos de color ANSI
/// (`\x1B[32m`, `\x1B[0m`, etc.) que deben eliminarse antes de mostrar
/// en la terminal de la UI.
pub fn strip_ansi(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        // Detectar secuencia ESC + '['
        if ch == '\x1B' && chars.peek() == Some(&'[') {
            chars.next(); // consumir '['
                          // Consumir hasta el byte de comando (letra ASCII)
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

// ─── Ejecución interna ────────────────────────────────────────────────────────

fn exec_powershell(
    command: &str,
    work_dir: Option<&Path>,
    tx: Option<&OutputSender>,
) -> Result<ProcOutput, InfraError> {
    // Preferir pwsh (PS 7) sobre powershell.exe (PS 5.1).
    // PS 7 tiene manejo de errores consistente y no tiene los bugs de PS 5.1
    // (ConvertFrom-Json con null, Start-Transcript sin consola, etc.)
    let ps_exe = if let Some(ps_exe) = supported_pwsh_executable() {
        ps_exe
    } else if which_pwsh() {
        env_config().pwsh_exe().to_string()
    } else {
        env_config().windows_powershell_exe().to_string()
    };

    // -NoProfile: evita que el perfil del usuario ejecute código ni escriba a stdout.
    //   Sin esto, un perfil que use Write-Host o Write-Output con codificación OEM/CP437
    //   inyecta bytes inválidos en UTF-8 al inicio de stdout, lo que hace que
    //   BufReader::lines() falle antes de leer las líneas propias del script.
    //
    // Forzar UTF-8 explícito en el propio comando como medida de defensa adicional,
    // por si algún comando externo (docker, choco) cambia la codificación mid-stream.
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

pub fn has_supported_pwsh() -> bool {
    supported_pwsh_executable().is_some()
}

pub fn current_pwsh_version() -> Option<String> {
    detect_pwsh_candidates()
        .into_iter()
        .next()
        .map(|probe| probe.version)
}

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
    cmd.creation_flags(CREATE_NO_WINDOW);

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

/// Detecta si `pwsh.exe` (PowerShell 7) está disponible en PATH.
/// El resultado se cachea — la detección solo ocurre una vez por proceso.
fn which_pwsh() -> bool {
    use std::sync::OnceLock;
    static PWSH_AVAILABLE: OnceLock<bool> = OnceLock::new();
    *PWSH_AVAILABLE.get_or_init(|| {
        let mut cmd = std::process::Command::new(env_config().pwsh_exe());
        cmd.arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.status().map(|s| s.success()).unwrap_or(false)
    })
}

fn exec_direct(
    program: &str,
    args: &[String],
    work_dir: Option<&Path>,
    tx: Option<&OutputSender>,
) -> Result<ProcOutput, InfraError> {
    if let Some(sender) = tx {
        return exec_direct_streaming(program, args, work_dir, sender);
    }

    // Modo batch: recoger todo el output al terminar (sin streaming)
    let mut cmd = Command::new(program);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }

    let child = cmd
        .spawn()
        .map_err(|e| InfraError::ProcessNotFound(format!("{program}: {e}")))?;

    // wait_with_output() lee stdout y stderr sin deadlock (buffers del SO)
    let output = child.wait_with_output().map_err(InfraError::Io)?;

    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&output.stderr));
    let text = merge_streams(stdout, stderr);

    tracing::trace!(
        program,
        success = output.status.success(),
        output_len = text.len(),
        "Proceso completado"
    );

    Ok(ProcOutput {
        text,
        success: output.status.success(),
    })
}

/// Modo streaming: lee stdout y stderr byte a byte con BufReader::read_until.
///
/// Usa `read_until(b'\n')` + `from_utf8_lossy` en lugar de `lines()` para tolerar
/// cualquier codificación de salida (OEM, Windows-1252, CP437, etc.).
/// `lines().map_while(Result::ok)` se detiene silenciosamente ante el primer byte
/// inválido en UTF-8, lo que provocaba perder la línea "Continue" del script.
///
/// Lee stdout en el hilo actual y stderr en un hilo separado para evitar
/// deadlock cuando ambos buffers se llenan simultáneamente.
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
    cmd.creation_flags(CREATE_NO_WINDOW);

    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| InfraError::ProcessNotFound(format!("{program}: {e}")))?;

    let stderr_pipe = child.stderr.take().expect("stderr piped");
    let stdout_pipe = child.stdout.take().expect("stdout piped");

    // Leer stderr en un hilo separado para evitar deadlock
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

    // Leer stdout en el hilo actual con read_until para tolerar cualquier encoding
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
        "Proceso streaming completado"
    );

    Ok(ProcOutput {
        text: merge_streams(stdout_text, stderr_text),
        success: status.success(),
    })
}

/// Combina stdout y stderr en un solo string.
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

// ─── Tests ────────────────────────────────────────────────────────────────────

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

    #[test]
    fn proc_output_contains() {
        let out = ProcOutput {
            text: "Docker is Installed".to_string(),
            success: true,
        };
        assert!(out.contains("Installed"));
        assert!(!out.contains("Running"));
    }

    #[test]
    fn ps_runner_default_scripts_dir() {
        let runner = PsRunner::new();
        assert!(runner.scripts_dir.to_string_lossy().contains("PS-Script"));
    }

    #[test]
    fn ps_runner_custom_scripts_dir() {
        let custom = PathBuf::from(r"C:\custom\scripts");
        let runner = PsRunner::with_scripts_dir(custom.clone());
        assert_eq!(runner.scripts_dir, custom);
    }
}
