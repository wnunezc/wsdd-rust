//! Suppression at diagnostic boundaries only; never changes process inputs or parsing.

use std::io::{self, Write};
use std::sync::{OnceLock, RwLock};

use regex::{Captures, Regex};

use super::setting::{AppSettings, PrereqCredentials, WebminCredentials};

const MASK: &str = "[REDACTED]";
const UNAVAILABLE: &str = "[Diagnostic suppressed: credential filter unavailable]";
const MAX_TRACE_LINE: usize = 1024 * 1024;

#[derive(Default)]
struct Redactor {
    values: Vec<String>,
}

impl Redactor {
    fn remember(&mut self, value: &str) {
        if value.is_empty() {
            return;
        }
        let json = serde_json::to_string(value).unwrap_or_default();
        let variants = [
            value.to_string(),
            value.replace('\'', "''"),
            value.replace('\'', "'\\''"),
            json.get(1..json.len().saturating_sub(1))
                .unwrap_or_default()
                .to_string(),
        ];
        for variant in variants {
            for part in std::iter::once(variant.as_str()).chain(variant.split(['\r', '\n'])) {
                if !part.is_empty() && part != MASK && !self.values.iter().any(|v| v == part) {
                    self.values.push(part.to_string());
                }
            }
        }
        self.values
            .sort_by_key(|value| std::cmp::Reverse(value.len()));
    }

    fn redact(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut rest = input;
        while !rest.is_empty() {
            if let Some(value) = self.values.iter().find(|value| {
                rest.starts_with(value.as_str())
                    && (!rest.starts_with(MASK)
                        || (value.starts_with(MASK) && value.len() > MASK.len()))
            }) {
                result.push_str(MASK);
                rest = &rest[value.len()..];
            } else if rest.starts_with(MASK) {
                result.push_str(MASK);
                rest = &rest[MASK.len()..];
            } else {
                let ch = rest.chars().next().expect("nonempty diagnostic text");
                result.push(ch);
                rest = &rest[ch.len_utf8()..];
            }
        }
        result
    }
}

fn registry() -> &'static RwLock<Redactor> {
    static REGISTRY: OnceLock<RwLock<Redactor>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(Redactor::default()))
}

/// Retain previous values for late process output and copies of existing UI buffers.
pub fn register_credentials(prereq: &PrereqCredentials, webmin: &[WebminCredentials], ngrok: &str) {
    if let Ok(mut redactor) = registry().write() {
        for value in [
            &prereq.mysql_user,
            &prereq.mysql_password,
            &prereq.mysql_root_password,
        ] {
            redactor.remember(value);
        }
        for credentials in webmin {
            redactor.remember(&credentials.username);
            redactor.remember(&credentials.password);
        }
        redactor.remember(ngrok);
    }
}

pub fn register_settings(settings: &AppSettings) {
    register_credentials(
        &settings.prereq_credentials,
        &settings.webmin_credentials,
        &settings.ngrok_authtoken,
    );
}

/// Handles named fields even before settings load, and known values without labels.
pub fn redact(input: &str) -> String {
    static FIELDS: OnceLock<Result<Regex, regex::Error>> = OnceLock::new();
    let fields = FIELDS.get_or_init(|| Regex::new(
        r#"(?ix)(\b(?:WEBMIN_USER|WEBMIN_PASS|USERNAME|PASSWORD|HASHED_PASSWORD|MYSQL_ROOT_PASSWORD|MYSQL_PASSWORD|MYSQL_USER|PMA_PASSWORD|PMA_USER|NGROK_AUTHTOKEN|mysql_root_password|mysql_password|mysql_user|ngrok_authtoken)\b["']?\s*[:=]\s*)(\[REDACTED\]|"(?:\\.|[^"\\])*"|'(?:''|\\.|[^'\\])*'|[^\s,;&}\]]+)"#,
    ));
    let Ok(fields) = fields else {
        return UNAVAILABLE.into();
    };
    let clean = crate::handlers::ps_script::strip_ansi(input);
    let labelled = fields.replace_all(&clean, |captures: &Captures<'_>| {
        let value = &captures[2];
        let quote = if value.starts_with('"') && value.ends_with('"') {
            "\""
        } else if value.starts_with('\'') && value.ends_with('\'') {
            "'"
        } else {
            ""
        };
        format!("{}{quote}{MASK}{quote}", &captures[1])
    });
    match registry().read() {
        Ok(redactor) => redactor.redact(&labelled),
        Err(_) => UNAVAILABLE.into(),
    }
}

/// Buffer tracing fragments so a credential split across writes is never published.
pub struct DiagnosticWriter<W: Write> {
    inner: W,
    pending: Vec<u8>,
    discarding: bool,
}

impl<W: Write> DiagnosticWriter<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            pending: Vec::new(),
            discarding: false,
        }
    }

    fn emit(&mut self) -> io::Result<()> {
        let text = String::from_utf8_lossy(&self.pending);
        let safe = redact(&text);
        self.pending.clear();
        self.inner.write_all(safe.as_bytes())
    }
}

impl<W: Write> Write for DiagnosticWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        for &byte in bytes {
            if self.discarding {
                if byte == b'\n' {
                    self.discarding = false;
                }
                continue;
            }
            self.pending.push(byte);
            if self.pending.len() > MAX_TRACE_LINE {
                self.pending.clear();
                self.discarding = byte != b'\n';
                self.inner
                    .write_all(b"[Diagnostic suppressed: oversized line]\n")?;
            } else if byte == b'\n' {
                self.emit()?;
            }
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // A flush is not a line boundary: it may occur halfway through a secret.
        self.inner.flush()
    }
}

impl<W: Write> Drop for DiagnosticWriter<W> {
    fn drop(&mut self) {
        if !self.pending.is_empty() {
            let _ = self.emit();
        }
        let _ = self.inner.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_values_and_previous_values_remain_suppressed_without_a_label() {
        let prereq = PrereqCredentials {
            mysql_user: "fixture-diagnostic-user-70c1".into(),
            mysql_password: "fixture-diagnostic-password-70c1".into(),
            mysql_root_password: "fixture-diagnostic-root-70c1".into(),
            ..Default::default()
        };
        register_credentials(&prereq, &[], "fixture-diagnostic-token-70c1");
        register_credentials(
            &PrereqCredentials::default(),
            &[],
            "fixture-diagnostic-new-token-70c1",
        );
        for value in [
            &prereq.mysql_user,
            &prereq.mysql_password,
            &prereq.mysql_root_password,
        ] {
            assert_eq!(redact(value), MASK);
        }
        assert_eq!(redact("fixture-diagnostic-token-70c1"), MASK);
        assert_eq!(redact("fixture-diagnostic-new-token-70c1"), MASK);
        assert!(!format!(
            "{:?}",
            crate::handlers::log_types::LogLine::error(&prereq.mysql_password)
        )
        .contains("fixture-diagnostic-password-70c1"));
    }

    #[test]
    fn known_values_overlap_and_escaping_are_suppressed_in_one_pass() {
        let mut redactor = Redactor::default();
        redactor.remember("synth'etic-credential");
        redactor.remember("synth'etic");
        redactor.remember("utf8-\u{e9}-credential");
        redactor.remember("line-one\nline-two");
        let safe = redactor.redact("synth'etic-credential synth''etic-credential synth'\\''etic-credential utf8-\u{e9}-credential line-one line-two");
        assert_eq!(safe, [MASK; 6].join(" "));
        assert_eq!(redactor.redact(&safe), safe);
    }

    #[test]
    fn named_fields_cover_buildkit_shell_json_yaml_and_tokens() {
        for line in [
            "#6 RUN USERNAME=fixture-user && PASSWORD='fixture pass'; command",
            r#"{"mysql_root_password":"fixture-root","ngrok_authtoken":"fixture-token"}"#,
            "PMA_PASSWORD: fixture-pma\nMYSQL_PASSWORD=fixture-db",
            "WEBMIN_PASS=fixture-webmin HASHED_PASSWORD=fixture-hash",
        ] {
            let safe = redact(line);
            assert!(!safe.contains("fixture"), "unsafe diagnostic");
            assert!(safe.contains(MASK));
            assert_eq!(redact(&safe), safe);
        }
        assert_eq!(
            redact("#6 ERROR: build failed; exit code: 56"),
            "#6 ERROR: build failed; exit code: 56"
        );
    }

    #[test]
    fn tracing_writer_handles_fragmented_writes_flushes_and_final_line() {
        let mut bytes = Vec::new();
        {
            let mut writer = DiagnosticWriter::new(&mut bytes);
            writer.write_all(b"PASSWORD=fixture-").unwrap();
            writer.flush().unwrap();
            writer
                .write_all(b"split\nNGROK_AUTHTOKEN=fixture-final")
                .unwrap();
        }
        let safe = String::from_utf8(bytes).unwrap();
        assert!(!safe.contains("fixture"));
        assert_eq!(safe.matches(MASK).count(), 2);
    }

    #[test]
    fn oversized_tracing_line_is_suppressed_without_raw_fallback() {
        let mut bytes = Vec::new();
        {
            let mut writer = DiagnosticWriter::new(&mut bytes);
            writer.write_all(&vec![b'x'; MAX_TRACE_LINE + 10]).unwrap();
            writer.write_all(b"\nWARNING: intact\n").unwrap();
        }
        let safe = String::from_utf8(bytes).unwrap();
        assert_eq!(
            safe,
            "[Diagnostic suppressed: oversized line]\nWARNING: intact\n"
        );
    }

    #[test]
    fn ansi_formatting_cannot_bypass_named_field_suppression() {
        let safe = redact("\x1b[1mPASSWORD\x1b[0m=\x1b[32mfixture-ansi-secret\x1b[0m");
        assert!(!safe.contains("fixture-ansi-secret"));
        assert_eq!(safe, "PASSWORD=[REDACTED]");
    }

    #[test]
    fn marker_prefix_in_a_credential_does_not_bypass_suppression() {
        let mut redactor = Redactor::default();
        redactor.remember("[REDACTED]-fixture-secret");
        redactor.remember("[R");
        assert_eq!(redactor.redact("[REDACTED]-fixture-secret"), MASK);
        assert_eq!(redactor.redact(MASK), MASK);
    }

    #[test]
    fn actual_tracing_formatter_never_publishes_a_sensitive_field() {
        #[derive(Clone)]
        struct Sink(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
        impl Write for Sink {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let bytes = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = Sink(bytes.clone());
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(true)
            .without_time()
            .with_writer(move || DiagnosticWriter::new(sink.clone()))
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            tracing::warn!(
                PASSWORD = "fixture-actual-trace-secret",
                "warning remains visible"
            );
        });
        let output = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
        assert!(!output.contains("fixture-actual-trace-secret"));
        assert!(output.contains(MASK));
        assert!(output.contains("warning remains visible"));
    }

    #[cfg(windows)]
    #[test]
    fn standalone_powershell_logs_use_the_filter_without_running_docker_or_wsl() {
        let dir = tempfile::tempdir().unwrap();
        let fixture = serde_json::json!({
            "prereq_credentials": {
                "mysql_user": "fixture-ps-user",
                "mysql_password": "fixture-ps-db",
                "mysql_root_password": "fixture-ps-root"
            },
            "webmin_credentials": [{"username": "fixture-ps-webmin", "password": "fixture-ps-webmin-pass"}],
            "ngrok_authtoken": "fixture-ps-token"
        });
        std::fs::write(
            dir.path().join("wsdd-secrets.json"),
            serde_json::to_vec(&fixture).unwrap(),
        )
        .unwrap();
        let scripts =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("recursos/recursos/PS-Script");
        let command = r#"
$ErrorActionPreference = 'Stop'
. (Join-Path $env:WSDD_TEST_SCRIPTS 'wsdd-diagnostic-filter.ps1')
Initialize-WsddDiagnosticFilter -EnvironmentRoot $env:WSDD_TEST_ROOT
$text = 'fixture-ps-user fixture-ps-db fixture-ps-root fixture-ps-webmin-pass fixture-ps-token PASSWORD=fixture-unknown'
$safe = Protect-WsddDiagnosticText $text
if ($safe.Contains('fixture-') -or -not $safe.Contains('[REDACTED]')) { throw 'PS credential suppression failed' }
$logFile = Join-Path $env:WSDD_TEST_ROOT 'setting.log'
$script:LogPath = Join-Path $env:WSDD_TEST_ROOT 'recovery.log'
foreach ($pair in @(@('dd-setting.ps1', 'Log'), @('wsdd-runtime-recovery.ps1', 'Write-RecoveryLog'))) {
  $tokens = $null; $errors = $null
  $ast = [Management.Automation.Language.Parser]::ParseFile((Join-Path $env:WSDD_TEST_SCRIPTS $pair[0]), [ref]$tokens, [ref]$errors)
  if ($errors.Count) { throw 'PS syntax error' }
  $name = $pair[1]
  $function = $ast.Find({param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq $name}, $true)
  # Execute only the logging function, never either script's deployment body.
  if ($name -eq 'Log') { & $function.Body.GetScriptBlock() -msg $text | Out-Null }
  else { & $function.Body.GetScriptBlock() -Level ERROR -Message $text }
}
foreach ($path in @($logFile, $script:LogPath)) {
  if ((Get-Content -LiteralPath $path -Raw).Contains('fixture-')) { throw 'PS log file exposed credentials' }
}
'invalid-json' | Set-Content -LiteralPath (Join-Path $env:WSDD_TEST_ROOT 'wsdd-secrets.json')
Initialize-WsddDiagnosticFilter -EnvironmentRoot $env:WSDD_TEST_ROOT
if ((Protect-WsddDiagnosticText $text) -ne '[Diagnostic suppressed: credential filter unavailable]') { throw 'PS filter did not fail closed' }
"#;
        let output = std::process::Command::new("pwsh")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                command,
            ])
            .env("WSDD_TEST_ROOT", dir.path())
            .env("WSDD_TEST_SCRIPTS", scripts)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "isolated PS log test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
