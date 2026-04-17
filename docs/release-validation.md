# Release Validation Strategy

This strategy separates checks that can run in CI from checks that require a real elevated Windows environment.

## Validation layers

| Layer | Scope | Command or evidence | Runs in CI |
|---|---|---|---|
| Unit tests | Pure Rust logic and isolated module behavior | `cargo test --workspace` | Yes |
| Isolated integration | Repo metadata, release gates, and package consistency without Docker/admin access | `cargo test --workspace` | Yes |
| Static gates | Formatting, compile, and lints | `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings` | Yes |
| Elevated manual | Docker Desktop, WSL, hosts, mkcert, deploy/remove, backup/restore | Manual checklist below | No |
| Release smoke | MSI creation, checksum, version metadata, local install sanity | `cargo wix-msi --nocapture` plus smoke checklist below | Release workflow/local |

## Local non-elevated gate

Run before preparing an MSI:

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace -- -D warnings
$env:WSDD_SKIP_WINDOWS_MANIFEST = "1"; cargo test --workspace --target-dir .\target\release-validation-test
```

Use a dedicated `--target-dir` when Windows keeps a previous test binary locked.

## Elevated manual checklist

Run from an administrator session on Windows 10/11:

- Fresh launch reaches the loader without panics.
- Requirements page detects PowerShell 7.5+, Chocolatey, mkcert, Docker Desktop, and Docker readiness.
- Docker/WSL lifecycle actions report clear success or failure.
- Base environment deploy is repeatable after a restart.
- Project deploy creates `options.yml`, SSL material, hosts entries, and the PHP container.
- Project remove is repeatable and cleans Docker/hosts state for that project.
- Environment backup and restore complete with readable logs.
- Help, settings, WSL settings, about, logs, containers, and projects remain usable after the deploy flow.

## Release smoke checklist

For each local RC MSI:

- Version is aligned in `Cargo.toml`, `wsdd-launcher/Cargo.toml`, `Cargo.lock`, README/legal text, and MSI legal text.
- `cargo wix-msi --nocapture` finishes successfully.
- `target/wix/wsdd-<version>-x86_64.msi` exists.
- SHA256 is generated beside the MSI.
- Built `wsdd.exe` and `wsdd-launcher.exe` file/product versions match the RC.
- No public GitHub Release is created unless explicitly approved.

## Evidence to record

Record the RC number, MSI path, SHA256, commands run, test counts, skipped checks, and manual findings in the release draft or session summary.
