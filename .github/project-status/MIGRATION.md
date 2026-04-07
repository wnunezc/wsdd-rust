# Estado de Migracion C# → Rust

## Progreso general: 5% (estructura creada, logica pendiente)

## Handlers migrados

| C# (Handlers/)          | Rust (src/handlers/)    | Estado        | Notas |
|-------------------------|-------------------------|---------------|-------|
| HandlerDocker.cs        | docker.rs               | 🚧 Esqueleto  | Wrappers basicos sobre PSScript |
| HandlerHosts.cs         | hosts.rs                | 🚧 Esqueleto  | Logica de marcadores implementada |
| HandlerProject.cs       | project.rs              | 📋 Pendiente  | |
| HandlerMKCert.cs        | mkcert.rs               | 🚧 Esqueleto  | |
| PSScript.cs             | ps_script.rs            | ✅ Completo   | |
| HandlerSetting.cs       | setting.rs              | ✅ Completo   | JSON en vez de XML |
| HandlerChocolatey.cs    | chocolatey.rs           | 🚧 Esqueleto  | |
| HandlerExternalApp.cs   | external_app.rs         | ✅ Completo   | |
| HandlerLogViewer.cs     | log_viewer.rs           | 🚧 Esqueleto  | |
| OutputToTerminal.cs     | output.rs               | 🚧 Esqueleto  | |
| HandlerXML.cs           | xml.rs                  | ✅ Completo   | |
| HandlerYml.cs           | yml.rs                  | ✅ Completo   | |
| Requirement.cs          | requirements.rs         | 🚧 Esqueleto  | UAC implementado |
| HandlerDisplay.cs       | —                       | 📋 Pendiente  | egui maneja multi-monitor |
| HandlerMenu.cs          | —                       | 📋 Pendiente  | reemplazado por ui/mod.rs |
| HandlerWSDD.cs          | app.rs                  | 🚧 Esqueleto  | Estado global |

## Forms migrados

| C# (Forms/)             | Rust (src/ui/)          | Estado        |
|-------------------------|-------------------------|---------------|
| Main.cs                 | main_window.rs          | 🚧 Esqueleto  |
| Wellcome.cs             | welcome.rs              | 🚧 Esqueleto  |
| Setting.cs              | settings.rs             | 🚧 Esqueleto  |
| About.cs                | about.rs                | 🚧 Esqueleto  |
| Helps.cs                | helps.rs                | 🚧 Esqueleto  |
| Loader.cs               | loader.rs               | 🚧 Esqueleto  |
| AddNewProject.cs        | add_project.rs          | 🚧 Esqueleto  |
| ToolBoxProject.cs       | toolbox_project.rs      | 🚧 Esqueleto  |
| ToolBoxContainer.cs     | toolbox_container.rs    | 🚧 Esqueleto  |
| DisplaySelector.cs      | display_selector.rs     | 🚧 Esqueleto  |
| WSLGeneralSetting.cs    | wsl_settings.rs         | 🚧 Esqueleto  |

## Orden de migracion recomendado

### Fase 1 — Infraestructura base
1. `requirements.rs` — verificacion completa de Docker, WSL, Chocolatey
2. `resources/mod.rs` — descompresion de recursos.zip verificada
3. `setting.rs` — lectura/escritura de configuracion estable

### Fase 2 — Wizard de bienvenida (Wellcome.cs)
4. `ui/welcome.rs` — pasos del wizard de primer arranque
5. `handlers/docker.rs` — instalacion y configuracion de Docker
6. `handlers/mkcert.rs` — instalacion de mkcert

### Fase 3 — Panel principal (Main.cs)
7. `models/project.rs` — structs completos + lectura desde XML
8. `handlers/project.rs` — listado, creacion, deploy
9. `ui/main_window.rs` — panel de proyectos y contenedores

### Fase 4 — Herramientas
10. `ui/add_project.rs` — formulario de nuevo proyecto
11. `ui/toolbox_project.rs` + `toolbox_container.rs`
12. `handlers/hosts.rs` — verificacion completa
13. `handlers/log_viewer.rs` — streaming de logs

### Fase 5 — Configuracion y detalles
14. `ui/settings.rs` + `wsl_settings.rs`
15. `ui/helps.rs` — contenido de ayuda
16. `ui/display_selector.rs` — multi-monitor

## Leyenda
- ✅ Completo — funcional y testeado
- 🚧 Esqueleto — estructura creada, logica pendiente
- 📋 Pendiente — no iniciado
