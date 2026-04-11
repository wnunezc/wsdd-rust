# WSDD — Migration Map C# to Rust

## Current status

| Version | Path | Status |
|---|---|---|
| C# reference | `Finished\WSDD-CSharp\WebStack Deployer for Docker\` | Archived reference implementation |
| Rust active app | `Projects\Desktop\WSDD\` | Current desktop application (`1.0.0-rc.7`) |

The Rust codebase is now the active implementation of WSDD. The original C# WinForms project
remains as a reference for historical behavior and edge cases.

## Migration summary

The initial desktop migration is functionally in place:

- Windows desktop UI implemented with `egui` / `eframe`
- First-run flow implemented: admin check, resource extraction, settings bootstrap
- Main panel implemented: containers, projects, logs, toolbar, menu actions
- Project management implemented: add project, deploy, remove, toolbox flows
- Settings, WSL settings, About, and Helps views implemented
- Multilingual UI available in `en`, `es`, `fr`, `hi`, and `zh`
- MSI installer flow implemented with WiX / `cargo-wix`

## Major component map

| C# area | Rust area | Notes |
|---|---|---|
| `Program.cs` | `src/main.rs` | Entry point, admin validation, startup bootstrap |
| `HandlerWSDD.cs` | `src/app.rs` | Global application state |
| `PSScript.cs` | `src/handlers/ps_script.rs` | Script runner and process integration |
| `HandlerDocker.cs` | `src/handlers/docker.rs` | Docker operations and container listing |
| `HandlerProject.cs` | `src/handlers/project.rs` | Project persistence and listing |
| `HandlerHosts.cs` | `src/handlers/hosts.rs` | Windows hosts integration |
| `HandlerYml.cs` | `src/handlers/yml.rs` | Compose/options file mutations |
| `HandlerSetting.cs` | `src/handlers/setting.rs` | App settings and persistence |
| `HandlerExternalApp.cs` | `src/handlers/external_app.rs` | Browser, explorer, external app launching |
| `Main.cs` | `src/ui/main_window.rs` | Main workspace, toolbar, menu, logs |
| `Wellcome.cs` | `src/ui/welcome.rs` | Welcome flow and embedded README |
| `Loader.cs` | `src/ui/loader.rs` | Startup requirement checks |
| `AddNewProject.cs` | `src/ui/add_project.rs` | Add-project form and flow |
| `ToolBoxContainer.cs` | `src/ui/toolbox_container.rs` | Container toolbox |
| `ToolBoxProject.cs` | `src/ui/toolbox_project.rs` | Project toolbox |
| `Setting.cs` | `src/ui/settings.rs` | Runtime settings UI |
| `WSLGeneralSetting.cs` | `src/ui/wsl_settings.rs` | WSL tuning UI |
| `About.cs` | `src/ui/about.rs` | About screen |
| `Helps.cs` | `src/ui/helps.rs` | In-app help/manual screen |

## Reference paths

- Active Rust project: `D:\OpsZone\DevWorkspace\Projects\Desktop\WSDD\`
- Archived C# reference: `D:\OpsZone\DevWorkspace\Finished\WSDD-CSharp\WebStack Deployer for Docker\`

## Current Rust structure

```text
src/
├── main.rs
├── app.rs
├── handlers/
├── i18n/
├── models/
├── resources/
└── ui/
```

## Post-migration roadmap status

The current work is no longer “core migration” work. The first follow-up product roadmap is now
implemented in the active Rust line:

- **Block F**: launcher and updater flow — implemented in `1.0.0-rc.4`
- **Block G**: backup and restore for Docker environments and individual projects — implemented in `1.0.0-rc.4`

## Notes

- Public documentation is centered on [README.md](README.md).
- Release packaging currently targets a Windows MSI installer.
- GitHub publication is performed from `clean-main` to avoid leaking unrelated local work from `main`.
