use anyhow::Result;

use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{PsRunner, ScriptRunner};

pub(super) fn handle_av_block(tx: Option<&LogSender>) -> Result<()> {
    let exe_path = std::env::current_exe()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|_| "wsdd.exe".to_string());

    let av_names = detect_installed_av();
    let only_defender = av_names.iter().all(|name| {
        let lower = name.to_lowercase();
        lower.contains("windows defender") || lower.contains("microsoft defender")
    });

    if only_defender && !av_names.is_empty() {
        handle_defender_block(tx, &exe_path);
    } else if let Some(tx) = tx {
        let av_refs: Vec<&str> = av_names.iter().map(|name| name.as_str()).collect();
        emit_mandatory_warnings(tx, &exe_path, &av_refs);
    }

    Err(anyhow::anyhow!(
        "El antivirus está bloqueando la escritura del archivo hosts.\n\
         La instalación no puede completarse hasta que wsdd.exe sea agregado a las excepciones del antivirus.\n\
         Consulta las instrucciones mostradas en pantalla."
    ))
}

fn handle_defender_block(tx: Option<&LogSender>, exe_path: &str) {
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
                emit_mandatory_warnings(tx, exe_path, &[]);
            }
        }
        Err(e) => {
            if let Some(tx) = tx {
                let _ = tx.send(LogLine::warn(format!(
                    "No se pudo agregar exclusión automática: {e}"
                )));
                emit_mandatory_warnings(tx, exe_path, &["Windows Defender"]);
            }
        }
    }
}

fn emit_mandatory_warnings(tx: &LogSender, exe_path: &str, av_names: &[&str]) {
    let _ = tx.send(LogLine::error(
        "✗ El antivirus está bloqueando la escritura del archivo hosts de Windows.",
    ));
    let _ = tx.send(LogLine::warn(
        "────────────────────────────────────────────────────────────",
    ));
    let _ = tx.send(LogLine::warn(
        "TAREA 1 — Agregar wsdd.exe a las excepciones del antivirus:",
    ));
    let _ = tx.send(LogLine::info(format!("  Ruta del ejecutable: {exe_path}")));
    let _ = tx.send(LogLine::info(""));

    if av_names.is_empty() {
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
    } else if lower.contains("defender")
        || lower.contains("microsoft")
        || lower.contains("windows security")
    {
        vec![
            "1. Abre Windows Security → Protección contra virus y amenazas",
            "2. Ve a 'Configuración de protección contra virus y amenazas'",
            "3. Desplázate hasta 'Exclusiones' → 'Agregar o quitar exclusiones'",
            "4. Haz clic en '+ Agregar exclusión' → 'Proceso'",
            "5. Ingresa: wsdd.exe",
            "6. Vuelve a abrir WSDD",
        ]
    } else {
        vec![
            "1. Abre la configuración de tu antivirus",
            "2. Busca la sección 'Exclusiones', 'Lista blanca' o 'Aplicaciones de confianza'",
            "3. Agrega la ruta completa de wsdd.exe como exclusión de proceso",
            "4. Guarda los cambios y vuelve a abrir WSDD",
        ]
    }
}

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
            .map(|line: &str| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect(),
        Err(e) => {
            tracing::warn!("Could not detect antivirus via WMI: {e}");
            Vec::new()
        }
    }
}
