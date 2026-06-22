# WSL Runtime Recovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Instalar una tarea programada idempotente que recupere automaticamente los runtimes PHP dependientes de WSL despues de reinicios de Docker Desktop o Windows.

**Architecture:** Se conserva el volumen externo UNC actual. Un worker PowerShell independiente descubre proyectos WSL, diagnostica runtimes por version PHP y solo recrea volumenes WSL y el contenedor compartido cuando el relay esta roto. Rust administra la tarea programada durante Save de Settings.

**Tech Stack:** Rust stable, PowerShell 7.5+, Docker Desktop, WSL 2, Task Scheduler XML, `schtasks.exe`.

---

## Alcance De Archivos

**Crear**

- `src/handlers/wsl_runtime_targets.rs`: resolucion pura de proyectos dependientes de WSL.
- `src/handlers/scheduled_task.rs`: XML, registro, consulta y retirada de la tarea.
- `recursos/recursos/PS-Script/wsdd-runtime-recovery.ps1`: worker independiente.
- `docs/maintenance/wsl-runtime-recovery.md`: operacion y diagnostico.

**Modificar**

- `src/handlers/mod.rs`
- `src/config/environment.rs`
- `src/handlers/setting/storage.rs`
- `src/handlers/setting/services.rs`
- `src/ui/settings/actions.rs`
- `src/main.rs`
- `src/ui/settings/sections.rs`
- `src/i18n/locales/{en,es,fr,hi,zh}.json`
- `src/resources/mod.rs`
- `tests/release_validation.rs`
- `recursos/ps-script.zip`
- `docs/maintenance/issue-8-managed-native-linux-storage-plan.md`

**No modificar**

- Docker Desktop settings.
- `dd-setting.ps1`, `dd-detector.ps1`, `dd-issettingup.ps1`.
- `options.phpXX.yml` y su contrato de volumenes externos.
- Datos de proyectos, distros o volumenes durante pruebas unitarias.

## Fase 1: Resolver Targets WSL Sin Tocar Docker

### Task 1: Modelo de target y clasificacion

**Files:**

- Create: `src/handlers/wsl_runtime_targets.rs`
- Modify: `src/handlers/mod.rs`
- Test: `src/handlers/wsl_runtime_targets.rs`

- [ ] **Step 1: Escribir pruebas fallidas**

Cubrir:

```rust
fn project(mode: StorageMode, work_path: &str) -> Project {
    Project {
        name: "demo".to_string(),
        domain: "demo.dock".to_string(),
        php_version: PhpVersion::Php83,
        work_path: work_path.to_string(),
        storage_mode: mode,
        ..Project::default()
    }
}

#[test]
fn managed_project_resolves_metadata_paths() {
    let mut value = project(StorageMode::ManagedNativeLinuxStorage, r"C:\WSDD-Projects\demo");
    value.managed_storage = Some(ManagedNativeStorage {
        distro: "WSDD-Storage".to_string(),
        linux_path: "/home/wsdd/projects/demo".to_string(),
        windows_unc: r"\\wsl.localhost\WSDD-Storage\home\wsdd\projects\demo".to_string(),
        windows_alias: Some(r"C:\WSDD-Projects\demo".to_string()),
    });

    let TargetResolution::Target(target) = resolve_project(&value, None) else {
        panic!("managed project must resolve");
    };
    assert_eq!(target.distro, "WSDD-Storage");
    assert_eq!(target.linux_path, "/home/wsdd/projects/demo");
}

#[test]
fn native_wsl_localhost_unc_is_resolved() {
    let value = project(
        StorageMode::NativeLinuxProject,
        r"\\wsl.localhost\Ubuntu\home\dev\demo",
    );
    let TargetResolution::Target(target) = resolve_project(&value, None) else {
        panic!("native UNC must resolve");
    };
    assert_eq!(target.distro, "Ubuntu");
    assert_eq!(target.linux_path, "/home/dev/demo");
}

#[test]
fn native_wsl_dollar_unc_is_resolved() {
    let value = project(
        StorageMode::NativeLinuxProject,
        r"\\wsl$\Debian\home\dev\demo",
    );
    let TargetResolution::Target(target) = resolve_project(&value, None) else {
        panic!("legacy UNC must resolve");
    };
    assert_eq!(target.distro, "Debian");
}

#[test]
fn native_absolute_path_uses_configured_default_distro() {
    let value = project(StorageMode::NativeLinuxProject, "/home/dev/demo");
    let TargetResolution::Target(target) = resolve_project(&value, Some("Ubuntu")) else {
        panic!("absolute Linux path must use configured distro");
    };
    assert_eq!(target.distro, "Ubuntu");
    assert_eq!(
        target.windows_unc,
        r"\\wsl.localhost\Ubuntu\home\dev\demo"
    );
}

#[test]
fn classic_and_fast_sync_are_ignored() {
    for mode in [StorageMode::ClassicBind, StorageMode::FastSyncWorkspace] {
        assert!(matches!(
            resolve_project(&project(mode, r"C:\projects\demo"), None),
            TargetResolution::NotWslDependent
        ));
    }
}

#[test]
fn unresolved_native_absolute_path_is_reported_without_guessing() {
    let value = project(StorageMode::NativeLinuxProject, "/home/dev/demo");
    assert!(matches!(
        resolve_project(&value, None),
        TargetResolution::UnresolvedNative { .. }
    ));
}
```

- [ ] **Step 2: Ejecutar RED**

```powershell
$env:WSDD_SKIP_WINDOWS_MANIFEST='1'
cargo test wsl_runtime_targets
```

Expected: FAIL porque el modulo y los tipos no existen.

- [ ] **Step 3: Implementar el contrato**

Tipos publicos:

```rust
pub struct WslRuntimeTarget {
    pub project_name: String,
    pub domain: String,
    pub php_version: PhpVersion,
    pub volume_name: String,
    pub distro: String,
    pub linux_path: String,
    pub windows_unc: String,
}

pub enum TargetResolution {
    Target(WslRuntimeTarget),
    NotWslDependent,
    UnresolvedNative { project_name: String, reason: String },
}

pub fn resolve_project(
    project: &Project,
    default_wsl_distro: Option<&str>,
) -> TargetResolution;
```

Reglas:

- Managed usa metadata persistida.
- Native UNC deriva distro y ruta Linux.
- Native absoluta requiere `default_wsl_distro`.
- `volume_name = <compose_tag>-<domain>`.
- No inferir una distro por enumeracion del sistema.

- [ ] **Step 4: Ejecutar GREEN**

```powershell
$env:WSDD_SKIP_WINDOWS_MANIFEST='1'
cargo test wsl_runtime_targets
```

Expected: PASS.

- [ ] **Step 5: Commit checkpoint**

Solo bajo autorizacion explicita:

```powershell
git add src/handlers/wsl_runtime_targets.rs src/handlers/mod.rs
git commit -m "feat: resolve WSL runtime targets"
```

## Fase 2: Worker Recuperador En Modo Diagnostico

### Task 2: Script, parametros, mutex y logging

**Files:**

- Create: `recursos/recursos/PS-Script/wsdd-runtime-recovery.ps1`
- Modify: `tests/release_validation.rs`

- [ ] **Step 1: Agregar validacion fallida del recurso**

La prueba debe abrir `recursos/ps-script.zip` y exigir:

```text
PS-Script/wsdd-runtime-recovery.ps1
```

Tambien debe validar tokens contractuales:

```text
Global\WSDD-WslRuntimeRecovery
-Mode
Diagnose
Recover
docker info
--no-build
--force-recreate
```

- [ ] **Step 2: Ejecutar RED**

```powershell
$env:WSDD_SKIP_WINDOWS_MANIFEST='1'
cargo test --test release_validation wsl_runtime_recovery
```

Expected: FAIL porque el script no existe en el ZIP.

- [ ] **Step 3: Crear worker base**

Interfaz:

```powershell
param(
    [ValidateSet('Diagnose', 'Recover')]
    [string]$Mode = 'Recover',
    [string]$EnvironmentRoot = 'C:\WSDD-Environment',
    [int]$DockerReadyTimeoutSeconds = 180
)
```

Funciones obligatorias:

```powershell
Write-RecoveryLog
Rotate-RecoveryLog
Enter-RecoveryMutex
Read-WsddSettings
Read-WsddProjects
Resolve-WslRuntimeTargets
Wait-DockerReady
Get-RuntimeHealth
Invoke-RuntimeRecovery
```

Semantica:

- mutex ocupado -> log + `exit 0`;
- config/proyectos ausentes -> `exit 0`;
- cero targets -> `exit 0`;
- Docker Desktop no iniciado -> `exit 0`;
- `Diagnose` nunca ejecuta stop/rm/create/compose;
- no usar `Write-Host`, MessageBox ni prompts.

- [ ] **Step 4: Probar diagnostico sin targets**

Usar un `EnvironmentRoot` temporal con config y carpeta `projects` vacia:

```powershell
pwsh.exe -NoLogo -NoProfile -NonInteractive -File `
  .\recursos\recursos\PS-Script\wsdd-runtime-recovery.ps1 `
  -Mode Diagnose `
  -EnvironmentRoot "$env:TEMP\wsdd-recovery-empty"
$LASTEXITCODE
```

Expected: `0`, log contiene `No WSL runtime targets`.

- [ ] **Step 5: Empaquetar ZIP**

Sincronizar `recursos/ps-script.zip` usando la estructura canonica `PS-Script/...`.

- [ ] **Step 6: Ejecutar GREEN**

```powershell
$env:WSDD_SKIP_WINDOWS_MANIFEST='1'
cargo test --test release_validation wsl_runtime_recovery
```

Expected: PASS.

## Fase 3: Diagnostico De Runtime E Idempotencia

### Task 3: Estado saludable, detenido y relay roto

**Files:**

- Modify: `recursos/recursos/PS-Script/wsdd-runtime-recovery.ps1`
- Test: pruebas manuales con fixture desechable

- [ ] **Step 1: Implementar descubrimiento**

El script debe leer JSON sin modificarlo y producir targets equivalentes al resolver Rust.
Debe agrupar:

```powershell
$groups = $targets | Group-Object php_version
```

Cada target contiene:

```text
project_name, domain, php_version, container_name,
volume_name, distro, linux_path, windows_unc
```

- [ ] **Step 2: Implementar health check sin mutacion**

Orden:

1. `wsl.exe -d <distro> -- test -d <linux_path>`.
2. `Test-Path -LiteralPath <windows_unc>`.
3. `docker inspect <container>`.
4. Si running: `docker exec <container> test -d /var/www/html/<domain>`.
5. Clasificar:

```text
Healthy
NotProvisioned
StoppedStartable
BrokenWslRelay
MissingSource
UnknownFailure
```

`NotProvisioned` no es error si no existe imagen compatible.

- [ ] **Step 3: Probar dos invocaciones simultaneas**

Lanzar dos procesos `Diagnose` sobre el mismo fixture. Expected:

- uno adquiere mutex;
- el segundo registra `Another recovery instance is active`;
- ambos terminan `0`;
- no hay comandos Docker mutantes.

- [ ] **Step 4: Probar runtime sano**

Crear un probe desechable con nombres:

```text
wsdd-wsl-recovery-probe
wsdd-wsl-recovery-probe-volume
/home/wsdd/projects/.wsdd-recovery-probe
```

Ejecutar dos veces `Recover`. Expected:

- primera ejecucion diagnostica `Healthy`;
- segunda ejecucion diagnostica `Healthy`;
- `docker inspect .Created` no cambia;
- no aparece `force-recreate` en el log.

## Fase 4: Reparacion Minima Por Version PHP

### Task 4: Rehidratar volumenes y contenedor

**Files:**

- Modify: `recursos/recursos/PS-Script/wsdd-runtime-recovery.ps1`

- [ ] **Step 1: Implementar intento de start**

Para un contenedor detenido:

```powershell
docker start $containerName
```

Si inicia y todas las rutas son legibles, terminar sin recrear.

- [ ] **Step 2: Implementar reparacion solo ante evidencia**

Reparar cuando:

- error OCI contiene `docker-desktop-bind-mounts` y `no such file or directory`;
- `docker start` falla y todos los sources WSL/UNC existen;
- el contenedor running no puede leer uno o mas mounts esperados.

No reparar `UnknownFailure`; registrar para diagnostico.

- [ ] **Step 3: Secuencia de reparacion**

Por version PHP:

```text
validar todos los sources
capturar inspect de volumenes
docker stop contenedor (si aplica)
docker rm contenedor
por cada volumen WSL:
    docker volume rm volumen
    docker volume create --driver local
        --opt type=none
        --opt device=<windows_unc>
        --opt o=bind
        <volume_name>
docker-compose -p wsdd-projects
    -f <webserver.yml>
    -f <options.yml>
    up -d --no-build --force-recreate
```

No tocar volumenes de proyectos Classic/Fast Sync.

- [ ] **Step 4: Postcondiciones**

Esperar hasta 120 segundos:

- contenedor `running`;
- cada volumen existe y su `Options.device` coincide;
- cada `/var/www/html/<domain>` es directorio legible;
- proceso Apache existe.

Solo entonces registrar `Recovered`.

- [ ] **Step 5: Probar relay roto**

Con el probe:

1. Confirmar funcionamiento.
2. Reiniciar Docker Desktop o Windows con autorizacion explicita.
3. Confirmar error de relay.
4. Ejecutar worker manualmente.
5. Confirmar `Recovered`.
6. Ejecutarlo otra vez.
7. Confirmar `Healthy` sin nueva recreacion.

## Fase 5: Administrar Task Scheduler Desde Rust

### Task 5: Paths y XML determinista

**Files:**

- Modify: `src/config/environment.rs`
- Create: `src/handlers/scheduled_task.rs`
- Modify: `src/handlers/mod.rs`
- Test: `src/handlers/scheduled_task.rs`

- [ ] **Step 1: Escribir pruebas RED**

Exigir:

```rust
assert_eq!(paths.runtime_recovery_script(), ...);
assert_eq!(paths.runtime_recovery_log(), ...);
assert!(xml.contains("<MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>"));
assert!(xml.contains("<LogonTrigger>"));
assert!(xml.contains("SessionUnlock"));
assert!(xml.contains("<Interval>PT2M</Interval>"));
assert!(xml.contains("-Mode&quot; &quot;Recover"));
```

- [ ] **Step 2: Implementar paths**

Agregar:

```rust
pub fn runtime_recovery_script(&self) -> PathBuf;
pub fn runtime_recovery_log(&self) -> PathBuf;
pub fn runtime_recovery_task_xml(&self) -> PathBuf;
```

- [ ] **Step 3: Implementar Task Scheduler**

API:

```rust
pub const TASK_NAME: &str = "WSDD WSL Runtime Recovery";

pub fn render_task_xml(pwsh: &Path, script: &Path, user_sid: &str) -> String;
pub fn install_or_update(runner: &PsRunner) -> Result<(), InfraError>;
pub fn remove_if_present(runner: &PsRunner) -> Result<(), InfraError>;
pub fn exists(runner: &PsRunner) -> Result<bool, InfraError>;
pub fn run_now(runner: &PsRunner) -> Result<(), InfraError>;
```

XML:

- `LogonType=InteractiveToken`;
- `RunLevel=LeastPrivilege`;
- hidden;
- `IgnoreNew`;
- limite 10 minutos;
- logon + 30 segundos;
- session unlock;
- calendario con repeticion PT2M;
- ejecutar solo con sesion interactiva;
- no detener por bateria;
- iniciar cuando vuelva a estar disponible.

Comandos:

```powershell
schtasks.exe /Create /TN "WSDD WSL Runtime Recovery" /XML "<xml>" /F
schtasks.exe /Query /TN "WSDD WSL Runtime Recovery" /XML
schtasks.exe /Delete /TN "WSDD WSL Runtime Recovery" /F
```

- [ ] **Step 4: Ejecutar pruebas**

```powershell
$env:WSDD_SKIP_WINDOWS_MANIFEST='1'
cargo test scheduled_task
```

Expected: PASS.

## Fase 6: Ciclo De Vida En Settings

### Task 6: Reconciliar tarea al guardar

**Files:**

- Modify: `src/handlers/setting/storage.rs`
- Modify: `src/handlers/setting/services.rs`
- Modify: `src/ui/settings/actions.rs`
- Modify: `src/ui/settings/sections.rs`
- Modify: `src/i18n/locales/{en,es,fr,hi,zh}.json`
- Test: `src/ui/settings/actions.rs` o helper extraido

- [ ] **Step 1: Extraer decision pura**

```rust
pub enum RecoveryTaskDecision {
    InstallOrUpdate,
    KeepForExistingProjects,
    Remove,
}

pub fn recovery_task_decision(
    managed_enabled: bool,
    projects: &[Project],
    default_wsl_distro: Option<&str>,
) -> RecoveryTaskDecision;
```

Pruebas:

- enabled sin proyectos -> install;
- disabled con Managed -> keep;
- disabled con Native UNC -> keep;
- disabled sin targets -> remove;
- unresolved Native absoluto -> keep y warning.

- [ ] **Step 2: Integrar Save en orden transaccional**

Activacion:

```text
validar settings
provisionar WSDD-Storage
verificar worker
install_or_update task
query task
guardar settings
```

Desactivacion:

```text
listar proyectos
calcular decision
guardar settings
keep o remove task
```

Si falla instalar una tarea requerida, no guardar `enabled=true`.

- [ ] **Step 3: Actualizar UI**

La nota debe indicar:

```text
Activa WSDD-Storage y registra recuperacion automatica de runtimes WSL.
La tarea permanece instalada mientras existan proyectos dependientes de WSL.
```

El campo `WSL distro` deja de ser decorativo: es fallback para proyectos Native con ruta Linux
absoluta.

- [ ] **Step 4: Ejecutar pruebas**

```powershell
$env:WSDD_SKIP_WINDOWS_MANIFEST='1'
cargo test recovery_task_decision
cargo test config_v
```

Expected: PASS.

## Fase 7: Recursos, MSI Y Upgrade

### Task 7: Distribuir worker y reconciliar instalaciones existentes

**Files:**

- Modify: `src/resources/mod.rs`
- Modify: `src/main.rs`
- Modify: `tests/release_validation.rs`
- Modify: `recursos/ps-script.zip`
- Optional modify: `wix/main.wxs`

- [ ] **Step 1: Validar extraccion**

Agregar prueba que demuestre que una instalacion existente reemplaza una version antigua del
worker porque `ps_scripts_need_repair` compara bytes.

- [ ] **Step 2: Decidir ownership**

La tarea no debe ser creada directamente por MSI porque depende del usuario interactivo,
Settings y proyectos existentes. El MSI solo instala binarios; WSDD registra la tarea al
activar Managed o al reconciliar una configuracion existente.

- [ ] **Step 3: Reconciliacion de registro al iniciar WSDD**

Despues de `resources::init()` y de cargar Settings:

```text
si tarea requerida y falta/esta obsoleta -> install_or_update
si tarea no requerida y existe -> remove
```

Esta reconciliacion solo registra o retira Task Scheduler. No ejecuta `Recover`, no consulta
contenedores y no modifica Docker. Si falla, WSDD registra warning y continua; Settings permite
reintentar al guardar.

- [ ] **Step 4: Mostrar estado en Settings**

Al cargar Settings, mostrar:

```text
Installed
Missing
Not required
Needs update
```

- [ ] **Step 5: Release validation**

Exigir worker dentro del ZIP y mantener versiones de paquetes alineadas.

## Fase 8: Validacion Controlada

### Task 8: Matriz sin interfaz del usuario

**Files:**

- Create: `docs/maintenance/wsl-runtime-recovery.md`
- Modify: `docs/maintenance/issue-8-managed-native-linux-storage-plan.md`

- [ ] **Step 1: Prueba manual sin tarea**

Ejecutar worker `Diagnose`, luego `Recover` sobre probe. No usar proyectos reales.

- [ ] **Step 2: Instalar tarea de prueba**

Registrar la tarea y verificar:

```powershell
schtasks.exe /Query /TN "WSDD WSL Runtime Recovery" /V /FO LIST
```

- [ ] **Step 3: Concurrencia**

Disparar manualmente dos veces:

```powershell
schtasks.exe /Run /TN "WSDD WSL Runtime Recovery"
schtasks.exe /Run /TN "WSDD WSL Runtime Recovery"
```

Expected: una instancia opera; la otra es ignorada o sale por mutex.

- [ ] **Step 4: Repeticion saludable**

Esperar tres triggers periodicos. Expected:

- mismo `container id`;
- mismo `Created`;
- log `Healthy`;
- ninguna recreacion.

- [ ] **Step 5: Reinicio Docker Desktop**

Con autorizacion explicita:

1. reiniciar Docker Desktop;
2. no abrir WSDD;
3. esperar trigger;
4. confirmar contenedor recuperado;
5. confirmar proyectos accesibles.

- [ ] **Step 6: Reinicio Windows**

Con tarea instalada:

1. reiniciar equipo;
2. iniciar sesion;
3. abrir Docker Desktop o esperar auto-start;
4. no abrir WSDD;
5. confirmar recuperacion automatica.

- [ ] **Step 7: Compatibilidad**

Matriz:

```text
Managed PHP 8.3
Managed PHP 8.4
Native UNC PHP 8.3
Managed + Classic en PHP 8.4
PHP 8.3 y 8.4 simultaneos
Managed deshabilitado con proyectos existentes
cero targets
Docker Desktop cerrado
```

- [ ] **Step 8: Verificacion Rust**

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace -- -D warnings
$env:WSDD_SKIP_WINDOWS_MANIFEST='1'
cargo test --workspace
```

Expected: todo PASS.

## Fase 9: Cierre

### Task 9: Evidencia y decision de release

- [ ] **Step 1: Registrar evidencia**

Documentar timestamps, triggers, estados antes/despues, IDs de contenedor y logs.

- [ ] **Step 2: Criterio de cierre de issue #8**

No cerrar hasta completar:

- dos reinicios de Docker Desktop;
- dos reinicios completos de Windows;
- cero recreaciones con runtime sano;
- compatibilidad Managed + Classic;
- deshabilitacion segura.

- [ ] **Step 3: Commit checkpoint**

Solo bajo autorizacion explicita y despues de validacion:

```powershell
git add src recursos tests docs
git commit -m "feat: recover WSL runtimes after Docker restart"
```

- [ ] **Step 4: MSI**

No generar MSI hasta orden explicita posterior a la matriz completa.
