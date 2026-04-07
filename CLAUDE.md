# WSDD — WebStack Deployer for Docker (Rust Edition)

## Contexto del proyecto

Migracion de la version C# (WinForms/.NET 8.0) a Rust con egui.
La version C# de referencia esta archivada en:
`D:\OpsZone\DevWorkspace\Finished\WSDD-CSharp\WebStack Deployer for Docker\`

Progreso de migracion: ~0% (esqueleto creado, logica pendiente)

## Stack

- **Lenguaje**: Rust stable (sin nightly)
- **GUI**: egui 0.29 via eframe
- **Async**: tokio
- **Errores**: anyhow (propagacion) + thiserror (tipos propios)
- **Config**: serde_json (reemplaza XML de C#)
- **YAML/XML**: serde_yaml + quick-xml (para docker-compose y config de proyectos)
- **IDE**: RustRover

## Estructura del proyecto

```
src/
├── main.rs           — Punto de entrada, verificacion de admin, init recursos
├── app.rs            — Estado global (equivale a HandlerWSDD.cs)
├── ui/               — Formularios (equivale a Forms/ en C#)
│   ├── mod.rs        — Router de vistas + UiState
│   ├── main_window.rs
│   ├── welcome.rs
│   ├── settings.rs
│   ├── add_project.rs
│   ├── about.rs
│   ├── helps.rs
│   ├── loader.rs
│   ├── toolbox_project.rs
│   ├── toolbox_container.rs
│   ├── display_selector.rs
│   └── wsl_settings.rs
├── handlers/         — Logica de negocio (equivale a Handlers/ en C#)
│   ├── docker.rs     — HandlerDocker.cs
│   ├── hosts.rs      — HandlerHosts.cs
│   ├── project.rs    — HandlerProject.cs
│   ├── mkcert.rs     — HandlerMKCert.cs
│   ├── ps_script.rs  — PSScript.cs
│   ├── setting.rs    — HandlerSetting.cs
│   ├── chocolatey.rs — HandlerChocolatey.cs
│   ├── external_app.rs
│   ├── log_viewer.rs
│   ├── output.rs     — OutputToTerminal.cs
│   ├── xml.rs        — HandlerXML.cs
│   ├── yml.rs        — HandlerYml.cs
│   └── requirements.rs — Requirement.cs
├── models/
│   └── project.rs    — Structs de datos de proyectos
└── resources/
    └── mod.rs        — Descompresion de recursos.zip al primer arranque
```

## PENDIENTE antes de subir a GitHub

> `recursos/recursos.zip` contiene credenciales de dev hardcodeadas en `Docker-Structure/init.yml`
> (`MYSQL_ROOT_PASSWORD`, `MYSQL_PASSWORD`, `PMA_PASSWORD`: `1qazxsw2`) y en los Dockerfiles de
> PHP (`USERNAME="admin" PASSWORD="admin"` para webmin).
> Reemplazar por variables de entorno y regenerar el ZIP antes de hacer el repo publico.

## Recursos embebidos

- El ZIP `recursos/recursos.zip` se embebe en el binario via `include_bytes!` en `src/resources/mod.rs`
- Al primer arranque se extrae a `C:\WSDD-Environment\`
- Los scripts PS1 quedan en `C:\WSDD-Environment\PS-Script\`
- La configuracion de la app se guarda en `C:\WSDD-Environment\wsdd-config.json`

**Para actualizar los scripts PS1:**
1. Editar en `recursos/recursos/PS-Script/`
2. Recomprimir: `cd recursos/recursos && Compress-Archive -Force * ../recursos.zip`
3. Recompilar — el ZIP nuevo queda embebido automaticamente

## Regla obligatoria de fases

**NUNCA marcar una fase como completada (✅) en ningun documento hasta que el usuario lo confirme explicitamente.**
Al terminar la implementacion de una fase, presentar resumen de lo hecho y esperar confirmacion antes de actualizar el estado en `08-AI-Context/wsdd-rust.md` o cualquier otro documento.

## Principio de Responsabilidad Única (SRP) — OBLIGATORIO

Cada archivo `.rs` debe tener una sola responsabilidad clara. Dividir cuando:
- Un archivo comienza a manejar más de una responsabilidad
- Su tamaño crece significativamente (orientativo: >300 líneas es señal de alerta)
- Hay funciones que se pueden agrupar por dominio o propósito distinto
- Se detecta acoplamiento innecesario entre funcionalidades

Ejemplos de splits realizados:
- `handlers/log_types.rs` — tipos de canal separados de `ps_script.rs`
- `handlers/requirements.rs` — orquestación separada de los handlers individuales

Objetivo: archivos pequeños, cohesionados, fáciles de mantener.

## Reglas de desarrollo

0. **OBLIGATORIO — Corregir en lugar de crear**: Si un archivo `.rs` o `.ps1` existente es incorrecto, tiene un bug, o no sigue buenas prácticas, debe corregirse/reconstruirse en el mismo archivo. No crear archivos nuevos como alternativa. Si un PS1 está en `recursos/recursos/PS-Script/`, editar ahí y reempaquetar el ZIP. Si un `.rs` tiene lógica incorrecta, refactorizarlo en el mismo módulo.
1. No usar `unwrap()` ni `expect()` en paths de produccion — usar `?` con el tipo de error apropiado
2. Cada handler es un modulo independiente y stateless — el estado vive en `app.rs`
3. Las operaciones largas (Docker, PS1) deben ejecutarse en `tokio::task::spawn_blocking` para no bloquear la UI
4. `cargo fmt` y `cargo clippy -- -D warnings` deben pasar sin errores antes de commitear
5. Los forms de egui van en `ui/` — no mezclar logica de negocio en la UI
6. Los errores de infraestructura usan `InfraError`; los de negocio usan `DomainError`
7. Cada PS1 tiene una funcion `probe_*` correspondiente para evaluacion individual

## Arquitectura y mejores practicas

### Separacion de capas (dentro del crate unico)

```
src/
├── errors.rs       ← jerarquia de errores tipados (DomainError, InfraError, AppError)
├── models/         ← dominio puro: tipos fuertes, sin I/O (PhpVersion, Project, EntryPoint)
├── handlers/       ← infraestructura: procesos externos, Docker, archivos, red
│   ├── ps_script.rs  ← motor de ejecucion PS1 con trait ScriptRunner
│   └── ...
├── ui/             ← capa de presentacion: solo llama a handlers, sin logica propia
├── app.rs          ← estado global de la aplicacion
└── resources/      ← extraccion de recursos embebidos
```

### Practicas aplicadas (ref: Best-Practice.txt)

| Practica | Implementacion |
|---|---|
| #1 Dominio primero | `models/` sin dependencias de I/O; handlers stateless |
| #5 Traits para puertos | `ScriptRunner` trait — permite mocks en tests |
| #6 Errores tipados | `errors.rs`: DomainError / InfraError / AppError |
| #7 Tipos fuertes | `PhpVersion` enum, `EntryPoint` enum (no strings magicos) |
| #8 Modulos pequenos | Un modulo = una responsabilidad clara |
| #10 fmt + clippy | `rustfmt.toml` presente; clippy -D warnings en CI |
| #12 Observabilidad | `tracing` + `tracing-subscriber` (reemplaza env_logger) |
| #15 Async selectivo | spawn_blocking para PS1; logica de dominio sincrona |

### Evaluacion individual de scripts PS1

Cada handler expone funciones `probe_*` para testear un script en aislamiento:

```powershell
# Con RUST_LOG=debug cargo run se ven los traces de cada script
# Para evaluar individualmente desde codigo:
# docker::probe_installed(&runner).await
# docker::probe_configured(&runner).await
# docker::probe_running(&runner).await
# chocolatey::probe_installed(&runner).await
```

### Patron de comunicacion async → UI

```
tokio::task::spawn_blocking(|| { ... PS1 ... })
    |
    | mpsc::Sender<String>
    v
app.terminal_rx (Option<Receiver<String>>)
    |
    | drenado en cada frame de egui (update())
    v
app.ui.terminal_output: Vec<String>
```

## Plan de fases

| Fase | Estado | Objetivo |
|---|---|---|
| **1** | ✅ En progreso | Motor PS + modelos + arquitectura + documentacion |
| **2** | ❌ | Welcome Wizard completo (README + checkbox) |
| **3** | ❌ | Loader + Requirements (Docker/Choco/MKCert/Deploy) |
| **4** | ❌ | Handlers core (docker completo, hosts, yml) |
| **5** | ❌ | Panel principal (containers table, projects table, log tabs) |
| **6** | ❌ | Toolboxes + AddProject |
| **7** | ❌ | Settings, WSL settings, About, Helps |

Ver `MIGRATION.md` para el mapa detallado C# → Rust.

## Como compilar

```powershell
# Debug
cargo build

# Release (binario optimizado)
cargo build --release

# Verificar antes de commitear (obligatorio)
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test

# Ejecutar con logs de debug
$env:RUST_LOG="wsdd=debug"; cargo run
```

## Requisitos del sistema

- Rust stable (instalar con rustup)
- Windows 10/11 (la app requiere privilegios de administrador)
- Docker Desktop (se instala automaticamente si no esta presente)

## Version C# de referencia

La version C# esta archivada en el DevWorkspace como referencia permanente:
`D:\OpsZone\DevWorkspace\Finished\WSDD-CSharp\WebStack Deployer for Docker\`

Para consultar cualquier logica no migrada, buscar el Handler o Form correspondiente.
Ver `MIGRATION.md` para el estado de cada componente.
