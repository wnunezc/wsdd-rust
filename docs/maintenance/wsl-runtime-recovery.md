# WSL Runtime Recovery

## Proposito

WSDD registra la tarea programada `WSDD WSL Runtime Recovery` cuando Managed Native Linux
Storage esta habilitado o existen proyectos Native/Managed que dependen de una distro WSL.

La tarea no requiere que WSDD este abierto. Ejecuta el launcher instalado:

```text
wsdd-launcher.exe --recover-wsl-runtime C:\WSDD-Environment\PS-Script\wsdd-runtime-recovery.ps1
```

El launcher localiza PowerShell 7.5+ dinamicamente mediante `PATH` y `where.exe`, valida su
version y ejecuta el worker sin ventana de consola.

## Disparadores

- inicio de sesion con retraso de 30 segundos;
- desbloqueo de sesion;
- cambio de estado de `com.docker.service`, como señal temprana;
- cada 2 minutos durante la sesion.

`MultipleInstancesPolicy=IgnoreNew` y el mutex `Global\WSDD-WslRuntimeRecovery` impiden
ejecuciones paralelas. El launcher es una aplicacion GUI y crea PowerShell con
`CREATE_NO_WINDOW` y `-WindowStyle Hidden`. La señal del servicio no se interpreta como engine
listo: el worker espera hasta 180 segundos por una respuesta correcta de `docker info`. El
trigger periodico permanece como respaldo.

## Comportamiento

El worker solo considera:

- `managed_native_linux_storage`;
- `native_linux_project` con ruta UNC WSL;
- `native_linux_project` con ruta Linux absoluta y distro configurada en Settings.

Un runtime sano no se recrea. La reconstruccion solo se habilita cuando Docker informa
explicitamente un relay `docker-desktop-bind-mounts` inexistente. Cuando eso ocurre:

1. valida la ruta Linux y UNC;
2. intenta iniciar el contenedor una vez;
3. elimina solo el contenedor PHP compartido afectado;
4. recrea solo los volumenes WSL de esa version;
5. ejecuta Compose con `--no-build --force-recreate`;
6. valida estado, mounts y Apache.

No elimina datos, imagenes, distros, backups ni volumenes Classic.

Un contenedor detenido por otra causa solo recibe un intento de `docker start`. Si el fallo
posterior no confirma un relay WSL roto, el worker registra `UnhealthyNonRelay` y no elimina ni
recrea contenedores o volumenes.

## Circuito De Seguridad

Los fallos de reconstruccion se guardan por runtime en:

```text
C:\WSDD-Environment\state\wsl-runtime-recovery.json
```

Despues de tres fallos consecutivos con la misma huella de configuracion, el runtime pasa a
`RecoveryBlocked`. Las ejecuciones posteriores siguen diagnosticando, pero no vuelven a
reconstruirlo. El circuito se limpia cuando el runtime queda sano o cambia la configuracion de
sus targets.

Reinicio explicito de soporte:

```powershell
pwsh.exe -NoLogo -NoProfile -NonInteractive `
  -File C:\WSDD-Environment\PS-Script\wsdd-runtime-recovery.ps1 `
  -Mode Diagnose -ResetCircuit
```

## Diagnostico

Modo de solo lectura:

```powershell
pwsh.exe -NoLogo -NoProfile -NonInteractive `
  -File C:\WSDD-Environment\PS-Script\wsdd-runtime-recovery.ps1 `
  -Mode Diagnose
```

Log:

```text
C:\WSDD-Environment\logs\wsl-runtime-recovery.log
```

Consultar tarea:

```powershell
schtasks.exe /Query /TN "WSDD WSL Runtime Recovery" /V /FO LIST
```

Ejecutar manualmente:

```powershell
schtasks.exe /Run /TN "WSDD WSL Runtime Recovery"
```

## Salidas Normales

Codigo `0`:

- no hay proyectos WSL;
- Docker Desktop esta cerrado;
- otra instancia esta activa;
- runtime sano;
- runtime aun no aprovisionado;
- no hay nada que reparar.

Codigo `1`:

- source WSL desaparecido;
- volumen no recreable;
- Compose fallo;
- contenedor no alcanzo postcondiciones;
- runtime bloqueado tras tres fallos consecutivos.

## Relacion Con 2.1.0

Este mecanismo es una compensacion necesaria para una limitacion de lifecycle entre Docker
Desktop y WSL. Docker conserva referencias a relays internos efimeros, pero no garantiza su
rehidratacion despues de todos los reinicios. WSDD conserva la identidad logica de distro y ruta
Linux para reconstruir solo el estado derivado afectado.

Reportes relacionados:

- https://github.com/docker/for-win/issues/14924
- https://github.com/microsoft/WSL/issues/5851
