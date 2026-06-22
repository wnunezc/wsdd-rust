# WSL Runtime Recovery Design

Fecha: 2026-06-21

## Objetivo

Permitir que los contenedores PHP que dependen de proyectos almacenados en WSL vuelvan a
estar disponibles despues de reiniciar Windows o Docker Desktop, sin abrir WSDD, sin copiar
proyectos y sin instalar software de terceros.

## Decision arquitectonica

Se conserva el contrato actual de WSDD:

- archivos reales dentro de una distro WSL;
- acceso Windows por `\\wsl.localhost\<distro>\...`;
- volumen Docker externo `local`, `type=none`, `o=bind`, `device=UNC`;
- contenedor PHP compartido por version;
- `options.phpXX.yml` referencia volumenes externos.

No se adopta el bind Linux directo. Docker Desktop tambien lo traduce a un relay efimero y,
ademas, ejecutar Compose desde una distro concreta no representa correctamente proyectos
Native que pertenezcan a distintas distros dentro del mismo contenedor PHP.

La solucion es una tarea programada que rehidrata unicamente los volumenes WSL y el contenedor
PHP afectado cuando Docker Desktop pierde sus relays internos.

## Componentes

### Worker

Script no interactivo:

`C:\WSDD-Environment\PS-Script\wsdd-runtime-recovery.ps1`

Responsabilidades:

1. Adquirir el mutex global `Global\WSDD-WslRuntimeRecovery`.
2. Leer `wsdd-config.json` y los proyectos JSON.
3. Descubrir proyectos dependientes de WSL.
4. Despertar las distros requeridas.
5. Esperar a que Docker Desktop responda.
6. Diagnosticar cada runtime PHP compartido.
7. Reparar solamente runtimes rotos.
8. Validar contenedor, volumenes y rutas.
9. Escribir un log sin mostrar ventanas.

### Administrador de tarea

Modulo Rust responsable de generar XML de Task Scheduler y ejecutar `schtasks.exe`.

Nombre de tarea:

`WSDD WSL Runtime Recovery`

La tarea se instala cuando:

```text
Managed Storage habilitado
OR existe al menos un proyecto dependiente de WSL
```

Al deshabilitar Managed Storage:

- se impiden proyectos Managed nuevos;
- se conserva la tarea mientras existan proyectos WSL dependientes;
- nunca se elimina la distro, proyectos, alias, backups, volumenes o contenedores.

### Descubrimiento de targets

Se consideran dependientes de WSL:

- `ManagedNativeLinuxStorage`, usando `managed_storage.distro`, `linux_path` y `windows_unc`;
- `NativeLinuxProject` con ruta `\\wsl.localhost\<distro>\...`;
- `NativeLinuxProject` con ruta `\\wsl$\<distro>\...`;
- `NativeLinuxProject` con ruta Linux absoluta cuando `settings.wsl_distro` define la distro.

No se reparan:

- `ClassicBind`;
- `FastSyncWorkspace`;
- rutas Native absolutas sin distro resoluble.

Una ruta Native no resoluble produce warning y salida normal, no una operacion destructiva.

## Disparadores

Task Scheduler no ofrece un trigger nativo y confiable por inicio del proceso Docker Desktop.
El servicio `com.docker.service` tampoco equivale a engine listo y puede permanecer iniciado
cuando la interfaz se cierra.

La tarea usa:

- inicio de sesion con retraso de 30 segundos;
- desbloqueo de sesion;
- trigger periodico cada 2 minutos mientras el usuario tenga sesion.

La accion ejecuta `wsdd-launcher.exe --recover-wsl-runtime <worker>`. El launcher descubre
PowerShell 7.5+ dinamicamente mediante `PATH/where.exe`, valida la version y crea el proceso con
`CREATE_NO_WINDOW` y `-WindowStyle Hidden`.

El trigger periodico no implica reconstruccion periodica. Cada ejecucion hace primero una
comprobacion idempotente y termina inmediatamente si Docker no esta activo o el runtime esta
sano.

Protecciones de concurrencia:

- `MultipleInstancesPolicy=IgnoreNew`;
- mutex global;
- agrupacion y exclusion por version PHP;
- un runtime sano nunca se recrea.

## Flujo de recuperacion

Por cada version PHP con targets WSL:

1. Si no existe contenedor ni imagen PHP, registrar `not provisioned` y continuar.
2. Si el contenedor esta `running`, validar cada ruta con `docker exec test -d`.
3. Si todo es legible, registrar `healthy` y no modificar nada.
4. Si esta detenido, intentar `docker start` una vez.
5. Si inicia y pasa validacion, terminar sin recrear.
6. Si falla por relay ausente o las rutas no son legibles:
   - validar primero las rutas WSL y UNC;
   - detener y eliminar solamente el contenedor PHP compartido;
   - eliminar y recrear solamente los volumenes WSL de esa version;
   - conservar volumenes Classic y Docker-native;
   - ejecutar Compose con `--no-build --force-recreate`;
   - esperar y validar postcondiciones.

Nunca se ejecutan:

- `docker system prune`;
- eliminacion de imagenes;
- eliminacion de carpetas;
- importacion o eliminacion de distros;
- copia de proyectos;
- build automatico;
- modificacion de Docker Desktop settings.

## Resultados Y Codigos

Salida normal `0`:

- no hay targets;
- Managed Storage esta deshabilitado y no quedan proyectos WSL;
- Docker Desktop no esta iniciado;
- otra instancia posee el mutex;
- runtime sano;
- contenedor o imagen aun no aprovisionados;
- target Native no puede asociarse con seguridad a una distro.

Fallo real `1`:

- ruta declarada desaparecio;
- volumen no pudo recrearse;
- Compose fallo;
- contenedor recreado no alcanza `running`;
- ruta esperada no es legible dentro del contenedor.

Log:

`C:\WSDD-Environment\logs\wsl-runtime-recovery.log`

El log rota al superar 2 MiB y conserva tres generaciones.

## Activacion Desde Settings

Al activar y guardar Managed Storage:

1. Aprovisionar o validar `WSDD-Storage`.
2. Validar ruta UNC y root Linux.
3. Verificar que el script worker existe.
4. Registrar o actualizar la tarea.
5. Consultar la tarea para confirmar su configuracion.
6. Guardar Settings.

Al desactivar:

1. Examinar proyectos guardados.
2. Si quedan targets WSL, conservar la tarea y mostrar una nota informativa.
3. Si no quedan targets WSL, retirar la tarea.
4. Conservar todo el almacenamiento y runtime existente.

Al iniciar WSDD despues de una actualizacion:

- puede reconciliar exclusivamente la existencia/version de la tarea si la configuracion o los
  proyectos indican que es necesaria;
- no diagnostica, inicia ni recrea contenedores;
- la recuperacion sigue perteneciendo al worker programado.

## Estrategia De Pruebas

### Etapa A: Worker en modo diagnostico

No modifica Docker. Verifica descubrimiento, agrupacion, mutex, logs y codigos de salida.

### Etapa B: Probe desechable

Usa una carpeta, volumen y contenedor exclusivos. Se prueba:

- runtime sano;
- relay roto despues de reinicio;
- rehidratacion manual;
- rehidratacion por Task Scheduler;
- dos triggers simultaneos;
- triggers repetidos sin recreacion adicional.

### Etapa C: Runtime WSDD real

Primero una sola version PHP y un proyecto de prueba. Luego:

- Managed;
- Native UNC;
- Managed y Classic en la misma version PHP;
- PHP 8.3 y PHP 8.4 simultaneos;
- reinicio de Docker Desktop;
- reinicio completo de Windows.

## Criterios De Aceptacion

- Los proyectos siguen visibles por UNC y alias Windows.
- Docker Desktop muestra los contenedores.
- WSDD no necesita abrirse.
- El runtime sano no se recrea ante triggers repetidos.
- Un relay roto se rehidrata automaticamente.
- Classic Bind no pierde su volumen.
- Deshabilitar Managed no rompe proyectos WSL existentes.
- No target es una salida normal sin dialogos ni errores.
- Dos ejecuciones simultaneas no operan en paralelo.
