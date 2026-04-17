# Contributing

Thanks for contributing to WSDD.

## Before you start

- Use [Discussions](https://github.com/wnunezc/wsdd-rust/discussions) for questions, ideas, or general support
- Use [Issues](https://github.com/wnunezc/wsdd-rust/issues) for bugs and concrete feature requests
- For security-sensitive reports, follow [SECURITY.md](SECURITY.md) instead of opening a public issue

## Development expectations

WSDD is a Windows-first Rust desktop application built with `egui` / `eframe` and oriented to
local PHP + Docker development workflows.

Please keep changes aligned with the current architecture:

- UI code lives in `src/ui/`
- Business and infrastructure logic lives in `src/handlers/`
- State lives in `src/app.rs`
- Internationalization lives in `src/i18n/`

## Coding guidelines

- Do not use `unwrap()` or `expect()` in production paths
- Keep modules focused on a single responsibility
- Avoid mixing UI logic with infrastructure logic
- Prefer typed errors and explicit handling
- Keep Windows-specific behaviors intentional and well-documented

## Validation before opening a pull request

Run the project checks that apply to your change:

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo clippy -- -D warnings
cargo test
```

Notes:

- `cargo test --workspace` includes unit tests and isolated integration checks
- elevated Docker/WSL/hosts validation is manual; follow `docs/release-validation.md`
- If a validation step cannot run, explain why in the pull request description

## Pull request guidance

- Keep PRs focused and easy to review
- Explain the user-visible change and the technical approach
- Mention any risks, follow-up work, or known limitations
- Include screenshots when the change affects the UI
- Update docs when behavior or workflows change

## Branching

Contributors should branch from the latest `main` and open pull requests back to `main`.

Maintainers may use additional local publication branches for safety, but public contribution
flow is centered on pull requests targeting `main`.
