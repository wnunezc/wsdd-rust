# WSDD — Mapa de Migración C# → Rust

## Estado actual

| Versión | Ruta | Progreso |
|---|---|---|
| C# (referencia) | `Finished\WSDD-CSharp\WebStack Deployer for Docker\` | 100% |
| Rust (activa) | `Projects\Desktop\WSDD\` | ~8% |

---

## Mapa de componentes

### Handlers

| C# | Rust | Estado | Fase |
|---|---|---|---|
| `Program.cs` | `main.rs` | ✅ | — |
| `HandlerWSDD.cs` | `app.rs` + `handlers/requirements.rs` | 🔶 Parcial | 1 |
| `PSScript.cs` | `handlers/ps_script.rs` | ✅ Fase 1 | 1 |
| `Requirement.cs` | `handlers/requirements.rs` | ❌ | 3 |
| `HandlerDocker.cs` | `handlers/docker.rs` | 🔶 Stubs | 3–4 |
| `HandlerChocolatey.cs` | `handlers/chocolatey.rs` | 🔶 Stubs | 3 |
| `HandlerMKCert.cs` | `handlers/mkcert.rs` | ❌ | 3 |
| `HandlerHosts.cs` | `handlers/hosts.rs` | 🔶 Parcial | 4 |
| `HandlerProject.cs` | `handlers/project.rs` | ❌ | 5 |
| `HandlerYml.cs` | `handlers/yml.rs` | ❌ | 5 |
| `HandlerXML.cs` | `handlers/xml.rs` | ❌ | 5 |
| `HandlerSetting.cs` | `handlers/setting.rs` | ✅ | — |
| `HandlerMenu.cs` | `ui/main_window.rs` (inline) | ❌ | 5 |
| `HandlerLogViewer.cs` | `handlers/log_viewer.rs` | ❌ | 5 |
| `HandlerDisplay.cs` | N/A (egui gestiona monitores) | ✅ N/A | — |
| `HandlerExternalApp.cs` | `handlers/external_app.rs` | ❌ | 6 |
| `OutputToTerminal.cs` | `handlers/output.rs` | 🔶 | 3 |

### Formularios / UI

| C# Form | Rust | Estado | Fase |
|---|---|---|---|
| `Main.cs` | `ui/main_window.rs` | 🔶 Esqueleto | 5 |
| `Wellcome.cs` | `ui/welcome.rs` | 🔶 Incompleto | 2 |
| `Loader.cs` | `ui/loader.rs` | 🔶 Esqueleto | 3 |
| `AddNewProject.cs` | `ui/add_project.rs` | ❌ | 6 |
| `ToolBoxContainer.cs` | `ui/toolbox_container.rs` | ❌ | 6 |
| `ToolBoxProject.cs` | `ui/toolbox_project.rs` | ❌ | 6 |
| `Setting.cs` | `ui/settings.rs` | ❌ | 7 |
| `WSLGeneralSetting.cs` | `ui/wsl_settings.rs` | ❌ | 7 |
| `About.cs` | `ui/about.rs` | ❌ | 7 |
| `Helps.cs` | `ui/helps.rs` | ❌ | 7 |
| `DisplaySelector.cs` | N/A | ✅ N/A | — |

### Modelos

| C# | Rust | Notas |
|---|---|---|
| `class Project` | `models/project.rs::Project` | ✅ Actualizado Fase 1 |
| `class AppSettings` | `handlers/setting.rs::AppSettings` | ✅ |
| `class DockerContainer` | `models/project.rs::DockerContainer` | ✅ Fase 1 (struct base — campos runtime en Fase 4) |

---

## Diferencias de modelo — Project

| Campo C# | Campo Rust | Notas |
|---|---|---|
| `Name` | `name` | ✅ |
| `CustomUrl` (e.g. `"myapp.dock"`) | `domain` | ✅ incluye `.dock` |
| `SSL` | `ssl` | ✅ |
| `PhpLabel` (e.g. `"php8.3"`) | `php_version.dir_name()` | ✅ via enum |
| `PhpVersion` (e.g. `"php83"`) | `php_version.compose_tag()` | ✅ via enum |
| `WorkPath` | `work_path` | ✅ |
| `EntryPoint` | `entry_point` | ✅ Fase 1 |
| — | `status` | Mejora Rust: estado en el modelo |

---

## Scripts PowerShell — Inventario

| Script | Qué hace | Keyword de éxito | Usado en |
|---|---|---|---|
| `dd-isinstalled.ps1` | Detecta `Docker Desktop.exe` en Program Files | `"Installed"` | `docker::probe_installed()` |
| `dd-issettingup.ps1` | Verifica `settings.json` Docker con flags WSDD | `"Updated"` | `docker::probe_configured()` |
| `dd-isrunning.ps1` | Ejecuta `docker ps` sin error | `"Running"` | `docker::probe_running()` |
| `dd-setting.ps1` | Parchea `settings.json` + restart Docker completo | `"Continue"` | `docker::apply_settings()` |
| `dd-start.ps1` | Inicia servicio + Desktop + espera pipe + access | `"Continue"` | `docker::start()` |
| `dd-stop.ps1` | Para servicio + mata procesos Docker | — | `docker::stop()` |
| `dd-detector.ps1` | Detección avanzada (versión + running) | `"Running"` | Reservado |
| `dd-fixmysqlpermission.ps1` | `FullControl` en `Docker-Structure/data` | — | Menu: FixMySql |
| `wsl-shutdown.ps1` | `wsl --shutdown` | — | `docker::stop_wsl()` |
| `dd-addproject.ps1` | (vacío — sin implementar en C#) | — | Fase 6 |

> ⚠️ **Bug conocido en C#**: `dd-fixmysqlpermission.ps1` usa `C:\ProgramData\WSDD-Environment\`
> en lugar de `C:\WSDD-Environment\`. En Rust se usará la ruta correcta.

---

## Marcadores del archivo `hosts`

El archivo `C:\Windows\System32\drivers\etc\hosts` usa bloques con marcadores WSDD:

```
# WSDD Developer Area Docker
127.0.0.1 pma.wsdd.dock
127.0.0.1 mysql.wsdd.dock
# WSDD End of Area
```

La versión Rust usa los mismos marcadores para compatibilidad.

---

## Flujo de primer arranque

```
main() → ensure_admin() (UAC)
       → resources::init()       ← extrae recursos.zip → C:\WSDD-Environment\
       → AppSettings::load()     ← lee wsdd-config.json
       → si setup_completed=false:
           WelcomeView:
             - Muestra README
             - Checkbox "Acepto" habilita el botón
             - Al aceptar → LoaderView
           LoaderView (Requirement process):
             1. Docker: probe_installed → probe_configured → probe_running
                └─ Si no configurado: apply_settings() vía dd-setting.ps1
             2. Chocolatey: probe_installed → si no: instalar
             3. MKCert: probe_installed → si no: choco install + mkcert -install
             4. DeployEnvironment:
                ├─ Set DOCKER_HOST=tcp://localhost:2375
                ├─ docker network create wsdd-network
                ├─ docker volume create pma-code
                ├─ docker-compose -f init.yml create --build
                ├─ docker-compose -f init.yml up -d
                ├─ hosts::update(["pma.wsdd.dock", "mysql.wsdd.dock"])
                └─ settings.setup_completed = true → save
       → MainView
```

---

## Evaluación individual de scripts

Cada handler expone funciones `probe_*` que ejecutan un único script
y retornan el resultado tipado. Esto permite validar scripts de forma aislada:

```rust
// Evaluar individualmente sin ejecutar el flujo completo:
let runner = PsRunner::new();
let ok = docker::probe_installed(&runner).await?;
let ok = docker::probe_running(&runner).await?;
let ok = chocolatey::probe_installed(&runner).await?;
```

---

## Mejoras sobre C# implementadas en Rust

| Feature | C# | Rust |
|---|---|---|
| Eliminar proyecto | Código comentado | Implementar completo (Fase 6) |
| WSL General Settings (guardar) | Botón Save vacío | Implementar escritura `.wslconfig` (Fase 7) |
| Settings form | Form vacío | Implementar configuración real (Fase 7) |
| Ruta fix MySQL | Bug: usa ProgramData | Corregida (Fase 5) |
| Modelo Project | Strings en todos lados | Tipos fuertes: `PhpVersion`, `EntryPoint` |
| Output terminal | RichTextBox Windows | egui ScrollArea (cross-platform) |
| Errors | Exceptions no tipadas | `thiserror` jerarquía tipada |
| Logging | Debug.Output | `tracing` estructurado |

---

## Plan de fases

| Fase | Objetivo | Archivos principales |
|---|---|---|
| **1** | Motor PS + modelos + arquitectura | `ps_script.rs`, `errors.rs`, `models/project.rs` |
| **2** | Welcome Wizard completo | `ui/welcome.rs`, `app.rs` |
| **3** | Loader + Requirements | `ui/loader.rs`, `handlers/requirements.rs`, `docker.rs`, `chocolatey.rs`, `mkcert.rs` |
| **4** | Handlers core | `docker.rs` completo, `hosts.rs`, `yml.rs` |
| **5** | Panel principal | `ui/main_window.rs`, `handlers/project.rs`, `log_viewer.rs` |
| **6** | Toolboxes + AddProject | `ui/toolbox_*.rs`, `ui/add_project.rs`, `external_app.rs` |
| **7** | Settings y herramientas | `ui/settings.rs`, `ui/wsl_settings.rs`, `about.rs`, `helps.rs` |

---

## Mejores prácticas adoptadas

Ver `CLAUDE.md` sección "Arquitectura y mejores prácticas".
