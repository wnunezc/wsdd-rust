# Development Credentials And Diagnostic Suppression

## Authorized Scope

README warning, per-scenario developer documentation, stack relationships, and
suppression of credentials in WSDD logs. WSDD remains an AS-IS development stack.
No vault, rotation, build secrets, new authentication, credential migration or
container lifecycle change. No MSI, release, commit/push or remote issue change
was requested for this implementation. Published v2.1.3 is unchanged.

Subsequent user authorization on 2026-10-17 requests a local safety commit of
this implementation and documentation. It does not authorize a push or release.

## Implementation

- `handlers/diagnostics.rs`: one diagnostic filter, recognized credential fields
  plus values registered from loaded/saved/rendered WSDD settings. Covers MySQL
  users/passwords, per-PHP Webmin credentials and ngrok token. Remembers previous
  values during the process lifetime; handles overlapping values, supported JSON /
  single-quote escaping, UTF-8, multiline parts and ANSI. Poisoned/unavailable
  filter suppresses the diagnostic rather than publishing original text.
- Registers only loaded/saved/rendered credentials, not each keystroke in an
  editable settings draft. The registry exists only in memory; configuration and
  operational compose content are not redacted or changed by it.
- LogLine constructors, process-output channel, deploy log file, headers, loader,
  main/container log channels, Copy and legacy terminal/log readers apply suppression.
- `wsdd-diagnostic-filter.ps1` covers standalone script log sinks. Both
  dd-setting and WSL recovery load it and suppress credentials before writing
  their own files; internal command outputs/recovery decisions are unchanged.
  It reads known values from the selected environment's settings at script start,
  without modifying them; malformed settings suppress diagnostics, not recovery.
  Only these two scripts and the new helper were updated in ps-script.zip.
  The existing PS-Script repair/extraction installs the matching helper.
- The tracing writer buffers fragments through newline or end-of-event, so a
  flush halfway through a value does not publish a partial secret. Oversized
  tracing lines are explicitly suppressed, not emitted as raw fallback.
- Process runner traces use diagnostic arguments, but Command still receives
  the original arguments. ProcOutput retains raw text for internal parsing; its
  Debug representation is redacted. Captured machine data is not a safe log API.
- Bridge preserves output drain and per-operation keys. Write/worker failures
  warn and invalidate the complete-log claim, without converting a successful
  Docker operation into a failure or changing rollback semantics.
- Adds regex as a direct dependency (already present transitively). Cargo.lock
  is ignored/untracked under existing repo policy; no ignore rules were changed.
- README plus Spanish README link `docs/development-credentials.md`, documenting
  stack dependencies, initial vs persisted MySQL state, phpMyAdmin reuse, PHP-wide
  Webmin builds, application-specific credentials, ngrok and diagnostic limits.

## Verification

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace --quiet`: passed.
- `cargo clippy --workspace --quiet -- -D warnings`: passed.
- `WSDD_SKIP_WINDOWS_MANIFEST=1; cargo test --workspace --quiet`: passed,
  211 tests (195 main, 10 release validation, 6 launcher), none failed/ignored.
- 14 added tests cover values/fields, previous values, escaping and overlap,
  fragmented writes, ANSI, marker prefixes, actual tracing formatter, disk/channel
  suppression, Copy text, write errors and preservation of raw parsing data.
- The standalone PowerShell test uses synthetic configuration under tempdir and
  invokes only the two logging function bodies extracted via PowerShell AST.
  It checks log files and malformed-config suppression without running either
  script's Docker/WSL/deployment body.
- Test deploy logs use an isolated temporary directory instead of appending to
  the installed environment. Process tests use synthetic output, not Docker/WSL.
- Additional `cargo clippy --workspace --all-targets --quiet -- -D warnings`
  is blocked by two pre-existing test lints: items_after_test_module in
  `handlers/log_types.rs` and unnecessary_cast in `handlers/scheduled_task.rs`.
  These original constructs are unchanged; no unrelated cleanup was performed.
- `git diff --check`: passed. PS-Script ZIP matches all 13 source files by SHA256.
  Docker-Structure ZIP matches all 92 source files and is unchanged. Dockerfiles,
  compose templates, release workflows and launcher are unchanged.

## Limits

This is source-only, newer than the public 2.1.3 MSI. Fixtures use synthetic
credentials; no real credential values are included in this evidence. No runtime deployment, container restart,
SQL operation or installed environment rewrite was performed. This does not
rewrite old logs, encrypt settings or sanitize data retained directly by Docker,
applications, shells or third parties. Unknown application secrets and unsupported
encodings still require developer review. Generic credential matches may mask
unrelated diagnostic text. Issue #15's operational stage UI remains pending;
the issue was not closed or edited during this implementation.
