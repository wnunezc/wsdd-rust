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
// Equivalente a Handlers/HandlerHosts.cs
// Modifica C:\Windows\System32\drivers\etc\hosts (requiere admin)

use anyhow::{Context, Result};
use std::fs;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{PsRunner, ScriptRunner};

const HOSTS_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";
const WSDD_MARKER_START: &str = "# WSDD Developer Area Docker";
const WSDD_MARKER_END: &str = "# WSDD End of Area";

/// Crea un backup del archivo hosts antes de modificarlo.
/// Requiere que `C:\WSDD-Environment\` exista (creado por resources::extract).
fn backup_hosts() -> Result<()> {
    let backup = r"C:\WSDD-Environment\hosts.backup";
    fs::copy(HOSTS_PATH, backup).context("No se pudo crear backup de hosts")?;
    Ok(())
}

/// Elimina todas las entradas WSDD del archivo hosts.
pub fn remove_wsdd_entries() -> Result<()> {
    let content = fs::read_to_string(HOSTS_PATH)?;
    let cleaned = remove_wsdd_block(&content);
    backup_hosts()?;
    write_hosts_file(cleaned.as_bytes(), None)?;
    Ok(())
}

/// Actualiza el bloque WSDD del archivo hosts con los dominios de los servicios base.
///
/// Equivalente a `HandlerHosts.UpdateHost()` en la versión C#.
///
/// Dominios por defecto: `pma.wsdd.dock` y `mysql.wsdd.dock`.
/// Si ya existe un bloque WSDD, hace merge: conserva los dominios existentes
/// y añade los que falten. Si no existe, añade el bloque al final del archivo.
///
/// # Parámetros
/// - `extra_domains`: dominios adicionales a incluir además de los por defecto.
/// - `tx`: canal de log para mostrar progreso en la UI.
///
/// Retorna `Err` si no puede leer o escribir el archivo hosts.
/// El caller NO debe guardar `setup_completed = true` si esta función retorna `Err`.
pub fn update_host(extra_domains: Option<&[&str]>, tx: &LogSender) -> Result<()> {
    let _ = tx.send(LogLine::info("Verificando archivo hosts de Windows..."));

    // Diagnóstico: confirmar que el proceso tiene privilegios de administrador
    #[cfg(windows)]
    if !is_elevated() {
        let _ = tx.send(LogLine::warn(
            "Advertencia: el proceso NO está elevado — hosts.rs necesita privilegios de administrador",
        ));
    }

    let default_domains = ["pma.wsdd.dock", "mysql.wsdd.dock"];
    let mut domains: Vec<String> = default_domains.iter().map(|s| s.to_string()).collect();
    if let Some(extra) = extra_domains {
        for d in extra {
            let ds = d.to_string();
            if !domains.contains(&ds) {
                domains.push(ds);
            }
        }
    }

    let content = fs::read_to_string(HOSTS_PATH).context("No se pudo leer el archivo hosts")?;
    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    let start_idx = lines.iter().position(|l| l.trim() == WSDD_MARKER_START);

    if let Some(start) = start_idx {
        // Bloque WSDD ya existe — leer dominios existentes y hacer merge
        let end = lines[start + 1..]
            .iter()
            .position(|l| l.trim() == WSDD_MARKER_END)
            .map(|i| start + 1 + i)
            .unwrap_or(lines.len());

        for line in lines.iter().take(end).skip(start + 1) {
            if let Some(domain) = line.split_whitespace().last() {
                let ds = domain.to_string();
                if !domains.contains(&ds) {
                    domains.push(ds);
                }
            }
        }

        // Reemplazar contenido entre marcadores
        lines.drain((start + 1)..end);
        let entries: Vec<String> = domains.iter().map(|d| format!("127.0.0.1 {d}")).collect();
        for (i, entry) in entries.into_iter().enumerate() {
            lines.insert(start + 1 + i, entry);
        }
    } else {
        // No existe el bloque — añadir al final
        if lines.last().map(|l| !l.trim().is_empty()).unwrap_or(false) {
            lines.push(String::new());
        }
        lines.push(WSDD_MARKER_START.to_string());
        for d in &domains {
            lines.push(format!("127.0.0.1 {d}"));
        }
        lines.push(WSDD_MARKER_END.to_string());
    }

    // Backup — no bloqueante: un fallo de backup no debe impedir actualizar hosts
    if let Err(e) = backup_hosts() {
        let _ = tx.send(LogLine::warn(format!(
            "Advertencia: no se pudo crear backup de hosts: {e:#}"
        )));
    }

    // Usar CRLF — el archivo hosts de Windows usa CRLF; garantizar salto final
    let mut updated = lines.join("\r\n");
    if !updated.ends_with("\r\n") {
        updated.push_str("\r\n");
    }

    write_hosts_file(updated.as_bytes(), Some(tx))?;

    let _ = tx.send(LogLine::success("✓ Archivo hosts actualizado correctamente"));
    for d in &domains {
        let _ = tx.send(LogLine::info(format!("  → http://{d}")));
    }
    Ok(())
}

/// Escribe el contenido al archivo hosts con verificación read-back.
///
/// Estrategias:
/// 1. Escritura directa con `fs::write` + verificación read-back.
/// 2. Si el read-back falla (otro proceso revirtió el archivo), intenta via PsRunner.
/// 3. Si el intento 2 también falla, delega a `handle_av_block` para diagnóstico de AV.
fn write_hosts_file(content: &[u8], tx: Option<&LogSender>) -> Result<()> {
    // ── Intento 1: escritura directa + verificación ───────────────────────────
    if fs::write(HOSTS_PATH, content).is_ok() {
        // Verificar que el write realmente persistió (Docker Desktop / AV pueden revertir)
        match fs::read(HOSTS_PATH) {
            Ok(actual) if actual == content => return Ok(()),
            Ok(_) => {
                tracing::warn!("write_hosts: write() OK pero read-back no coincide — AV revirtió el archivo");
            }
            Err(e) => {
                tracing::warn!("write_hosts: write() OK pero read-back falló: {e}");
            }
        }
    }

    // ── Intento 2: PowerShell via PsRunner (mismo path que Docker/Choco) ──────
    let tmp = r"C:\WSDD-Environment\hosts.tmp";
    fs::write(tmp, content).context("No se pudo escribir archivo hosts temporal")?;

    let runner = PsRunner::new();
    let cmd = format!("Copy-Item -Force '{}' '{}'", tmp, HOSTS_PATH);
    let ps_result = runner
        .run_ps_sync(&cmd, None, None)
        .map_err(|e| anyhow::anyhow!("PowerShell Copy-Item falló al actualizar hosts: {e}"));

    let _ = fs::remove_file(tmp);
    ps_result?;

    // Verificar también el intento 2
    let actual = fs::read(HOSTS_PATH).context("No se pudo leer hosts tras write vía PowerShell")?;
    if actual != content {
        // Ambos intentos fallaron — probablemente AV
        return handle_av_block(tx);
    }

    Ok(())
}

/// Diagnostica el antivirus instalado y muestra instrucciones específicas para agregar
/// wsdd.exe a las excepciones. Retorna siempre `Err` porque la escritura falló.
///
/// Si solo está Windows Defender, intenta agregar la exclusión automáticamente y
/// sugiere reintentar. Para otros antivirus, muestra pasos específicos por producto.
fn handle_av_block(tx: Option<&LogSender>) -> Result<()> {
    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "wsdd.exe".to_string());

    // Detectar antivirus instalados via WMI SecurityCenter2
    let av_names = detect_installed_av();

    // Intentar auto-exclusión si solo está Windows Defender
    let only_defender = av_names.iter().all(|n| {
        let lower = n.to_lowercase();
        lower.contains("windows defender") || lower.contains("microsoft defender")
    });

    if only_defender && !av_names.is_empty() {
        if let Some(tx) = tx {
            let _ = tx.send(LogLine::warn(
                "El antivirus (Windows Defender) está bloqueando la escritura del archivo hosts.",
            ));
            let _ = tx.send(LogLine::info(
                "Intentando agregar exclusión automática en Windows Defender...",
            ));
        }

        let runner = PsRunner::new();
        let cmd = format!(
            "$OutputEncoding = [System.Text.Encoding]::UTF8; \
             [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; \
             Add-MpPreference -ExclusionProcess '{exe_path}'"
        );
        match runner.run_ps_sync(&cmd, None, None) {
            Ok(_) => {
                if let Some(tx) = tx {
                    let _ = tx.send(LogLine::success(
                        "✓ Exclusión agregada en Windows Defender. Reintenta la instalación.",
                    ));
                    emit_mandatory_warnings(tx, &exe_path, &[]);
                }
            }
            Err(e) => {
                if let Some(tx) = tx {
                    let _ = tx.send(LogLine::warn(format!(
                        "No se pudo agregar exclusión automática: {e}"
                    )));
                    emit_mandatory_warnings(tx, &exe_path, &["Windows Defender"]);
                }
            }
        }
    } else {
        // Otros antivirus o sin detección
        let av_refs: Vec<&str> = av_names.iter().map(|s| s.as_str()).collect();
        if let Some(tx) = tx {
            emit_mandatory_warnings(tx, &exe_path, &av_refs);
        }
    }

    Err(anyhow::anyhow!(
        "El antivirus está bloqueando la escritura del archivo hosts.\n\
         La instalación no puede completarse hasta que wsdd.exe sea agregado a las excepciones del antivirus.\n\
         Consulta las instrucciones mostradas en pantalla."
    ))
}

/// Emite los dos mensajes obligatorios:
/// 1. Instrucciones por producto para agregar wsdd.exe a excepciones.
/// 2. Advertencia de que la app no puede funcionar hasta resolver esto.
fn emit_mandatory_warnings(tx: &LogSender, exe_path: &str, av_names: &[&str]) {
    let _ = tx.send(LogLine::error(
        "✗ El antivirus está bloqueando la escritura del archivo hosts de Windows.",
    ));
    let _ = tx.send(LogLine::warn(
        "────────────────────────────────────────────────────────────",
    ));
    let _ = tx.send(LogLine::warn("TAREA 1 — Agregar wsdd.exe a las excepciones del antivirus:"));
    let _ = tx.send(LogLine::info(format!("  Ruta del ejecutable: {exe_path}")));
    let _ = tx.send(LogLine::info(""));

    if av_names.is_empty() {
        // No se detectó ninguno — instrucciones genéricas
        for step in av_specific_steps("generic") {
            let _ = tx.send(LogLine::info(format!("  {step}")));
        }
    } else {
        for av in av_names {
            let _ = tx.send(LogLine::warn(format!("  Antivirus detectado: {av}")));
            for step in av_specific_steps(av) {
                let _ = tx.send(LogLine::info(format!("    {step}")));
            }
            let _ = tx.send(LogLine::info(""));
        }
    }

    let _ = tx.send(LogLine::warn(
        "────────────────────────────────────────────────────────────",
    ));
    let _ = tx.send(LogLine::error(
        "TAREA 2 — IMPORTANTE: La aplicación NO puede completar la instalación",
    ));
    let _ = tx.send(LogLine::error(
        "  hasta que wsdd.exe sea agregado a las excepciones del antivirus.",
    ));
    let _ = tx.send(LogLine::error(
        "  Cada vez que falle la escritura del archivo hosts, la instalación",
    ));
    let _ = tx.send(LogLine::error(
        "  se detendrá con este error. Completa la Tarea 1 y vuelve a abrir WSDD.",
    ));
    let _ = tx.send(LogLine::warn(
        "────────────────────────────────────────────────────────────",
    ));
}

/// Retorna pasos específicos por antivirus para agregar wsdd.exe a excepciones.
fn av_specific_steps(av_name: &str) -> Vec<&'static str> {
    let lower = av_name.to_lowercase();

    if lower.contains("kaspersky") {
        vec![
            "1. Abre Kaspersky → Configuración (ícono de engranaje)",
            "2. Ve a Seguridad → Prevención de intrusiones del host (HIPS)",
            "3. Haz clic en 'Administrar aplicaciones'",
            "4. Busca wsdd.exe en la lista o agrega con 'Agregar aplicación'",
            "5. Establece el grupo de confianza: 'De confianza'",
            "6. Cierra y vuelve a abrir WSDD",
        ]
    } else if lower.contains("eset") {
        vec![
            "1. Abre ESET → Configuración → Protección del equipo",
            "2. Ve a Exclusiones → Agregar",
            "3. Selecciona 'Proceso' e ingresa la ruta de wsdd.exe",
            "4. Confirma y cierra ESET",
            "5. Vuelve a abrir WSDD",
        ]
    } else if lower.contains("avast") || lower.contains("avg") {
        vec![
            "1. Abre Avast/AVG → Menú → Configuración",
            "2. Ve a General → Exclusiones",
            "3. Haz clic en 'Agregar exclusión'",
            "4. Ingresa la ruta completa de wsdd.exe",
            "5. Confirma y vuelve a abrir WSDD",
        ]
    } else if lower.contains("norton") || lower.contains("symantec") {
        vec![
            "1. Abre Norton → Configuración → Antivirus → Exclusiones/Confianza de bajo riesgo",
            "2. En 'Elementos de confianza', haz clic en 'Configurar'",
            "3. Agrega wsdd.exe como elemento de confianza",
            "4. Guarda y vuelve a abrir WSDD",
        ]
    } else if lower.contains("bitdefender") {
        vec![
            "1. Abre Bitdefender → Protección → Antivirus → Configuración",
            "2. Ve a 'Lista de exclusiones' → Agregar",
            "3. Selecciona 'Archivo/Carpeta' e ingresa la ruta de wsdd.exe",
            "4. También agrega exclusión en 'Escudo avanzado de amenazas' si está activo",
            "5. Guarda y vuelve a abrir WSDD",
        ]
    } else if lower.contains("defender") || lower.contains("microsoft") || lower.contains("windows security") {
        vec![
            "1. Abre Windows Security → Protección contra virus y amenazas",
            "2. Ve a 'Configuración de protección contra virus y amenazas'",
            "3. Desplázate hasta 'Exclusiones' → 'Agregar o quitar exclusiones'",
            "4. Haz clic en '+ Agregar exclusión' → 'Proceso'",
            "5. Ingresa: wsdd.exe",
            "6. Vuelve a abrir WSDD",
        ]
    } else {
        // generic
        vec![
            "1. Abre la configuración de tu antivirus",
            "2. Busca la sección 'Exclusiones', 'Lista blanca' o 'Aplicaciones de confianza'",
            "3. Agrega la ruta completa de wsdd.exe como exclusión de proceso",
            "4. Guarda los cambios y vuelve a abrir WSDD",
        ]
    }
}

/// Detecta los antivirus instalados via WMI SecurityCenter2.
/// Retorna una lista de nombres de productos. Lista vacía si no se puede determinar.
fn detect_installed_av() -> Vec<String> {
    let runner = PsRunner::new();
    let cmd = "$OutputEncoding = [System.Text.Encoding]::UTF8; \
               [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; \
               Get-CimInstance -Namespace 'root\\SecurityCenter2' -ClassName AntiVirusProduct \
               | Select-Object -ExpandProperty displayName";

    match runner.run_ps_sync(cmd, None, None) {
        Ok(output) => output
            .text
            .lines()
            .map(|l: &str| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect(),
        Err(e) => {
            tracing::warn!("No se pudo detectar antivirus via WMI: {e}");
            Vec::new()
        }
    }
}

/// Verifica si el proceso actual está corriendo con token de administrador elevado.
///
/// Usa `GetTokenInformation(TokenElevation)` — retorna `false` si no es posible
/// obtener la información o si el token no está elevado.
#[cfg(windows)]
fn is_elevated() -> bool {
    use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token = windows::Win32::Foundation::HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut ret_len = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut TOKEN_ELEVATION as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_len,
        );
        ok.is_ok() && elevation.TokenIsElevated != 0
    }
}

fn remove_wsdd_block(content: &str) -> String {
    let mut result = String::new();
    let mut inside_block = false;
    for line in content.lines() {
        if line.contains(WSDD_MARKER_START) {
            inside_block = true;
            continue;
        }
        if line.contains(WSDD_MARKER_END) {
            inside_block = false;
            continue;
        }
        if !inside_block {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}
