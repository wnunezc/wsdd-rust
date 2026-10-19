# Deployment stage trace and complete log

## Scope and execution

`Deploy PHP` reports preparation, image build, container creation, and Apache
verification as separate measured stages. Project deployment reports project
preparation, PHP image/container, and project configuration. The image/container
stage is combined for projects because its existing Compose invocation performs
`up --build --force-recreate` in one process. No Docker command or rollback
order changes were needed for stage reporting.

The later Tools redesign now registers Deploy of ngrok, Redis, Memcached, and
Mailpit through the same operation trace and per-operation log. It reports
service preparation, Compose image download/start, container running, and for
ngrok, Agent API response. These services use published images; their Compose
output may contain pull progress, but there is no PHP-style BuildKit image
construction stage to report. The Tools dialog also shows these measured
stages and offers Open/Copy for its log. Stop and the read-only ngrok API check
remain separate background actions. This addition was not in the earlier MSI
documented below; see `settings-tools-reorganization-2026-10-19.md` for the
newer local build and its acceptance limit.

Each started deployment gets a unique `wsdd-operation-*.log` in the environment
logs directory. The per-operation writer receives the same redacted output
stream as the UI before progress lines are compacted in the on-screen buffer.
The existing daily deployment log remains in place. The panel provides an
operation selector, live stage duration, the latest observed BuildKit step,
an expandable live stream, and actions to open or copy the full operation log.
BuildKit labels show step numbers and status only; command arguments are kept
out of the stage summary. A failed stage and operation are marked in the UI.

## Diagnostic correction

Known passwords and tokens remain masked wherever they occur. Usernames are
masked at identifier boundaries, so the configured username `admin` no longer
changes `phpadmin56.wsdd.dock` into `php[REDACTED]56.wsdd.dock`. The same rule
applies to Rust and the standalone PowerShell diagnostic filter. Both still
mask a standalone `admin` value. The changed PowerShell source was synchronized
with `recursos/ps-script.zip` and its ZIP entry was checked by SHA-256.

## Verification and limits

`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, and
`WSDD_SKIP_WINDOWS_MANIFEST=1 cargo test --workspace --quiet` pass. The tests
include a failed synthetic BuildKit operation with a complete redacted file
and a standalone PowerShell filter check. No Docker container was created or
modified during verification. A live Docker/BuildKit deployment through the UI
remains for user validation. The process abstraction exposes success/failure,
not a numeric exit code; the UI therefore reports the failed stage and retains
the underlying output instead of inventing a code. A log-file write failure is
reported in the main log and disables file actions for that operation.

The combined local MSI for this trace and the Compose group changes was built
on 2026-10-19. Build evidence and its checksum are recorded in
`local-msi-validation-2026-10-19.md`. Installing it and exercising a real
Docker deployment remain user validation steps.
