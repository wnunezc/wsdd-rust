# Issue #8 - Managed Native Linux Storage development plan

Fecha: 2026-06-21
Base local: WSDD v2.0.1 (`19a4cef`)
Issue: https://github.com/wnunezc/wsdd-rust/issues/8

## Contexto

WSDD volvió localmente a v2.0.1 porque la implementación posterior de Managed Native Linux Storage introdujo fallos operativos que no existían en la versión estable. Este plan conserva las decisiones útiles tomadas durante las conversaciones del issue #8, pero descarta la implementación fallida y cualquier migración destructiva como primer objetivo.

La meta no es recuperar los commits fallidos. La meta es reconstruir la funcionalidad desde v2.0.1 con fases pequeñas, verificables y reversibles.

## Decisiones que se conservan

- `Classic Bind` sigue siendo el modo default y compatible.
- `Native Linux Project` debe usar storage Linux/WSL real para que Docker/PHP no lea cada archivo desde bind mount Windows.
- Windows debe ver una ruta usable por Explorer/IDE, idealmente la ruta original elegida por el usuario como alias.
- Docker/PHP no debe montar la ruta UNC de Windows; debe montar la ruta Linux real.
- La vista Windows/UNC esperada es:
  `\\wsl.localhost\WSDD-Storage\home\wsdd\projects\<project>`
- La ruta Linux runtime esperada es:
  `/home/wsdd/projects/<project>`
- La ruta dentro del contenedor PHP debe seguir siendo compatible con el contrato actual de WSDD:
  `/var/www/html/<domain>.dock`
- SMB/Samba queda fuera del camino principal. Solo puede considerarse fallback futuro si symlink/reparse point hacia UNC WSL no es confiable.
- Backup/restore debe operar sobre archivos reales, no sobre links/stubs.
- No se debe modificar Docker Desktop settings desde esta funcionalidad.
- Los scripts legacy `dd-setting.ps1`, `dd-detector.ps1` y `dd-issettingup.ps1` no son parte del storage administrado.

## Decisiones que se descartan por ahora

- No implementar migración `Classic Bind -> Native Linux` en la primera fase.
- No detener, eliminar ni recrear contenedores como parte de pruebas tempranas de storage.
- No eliminar volúmenes existentes para probar la idea.
- No guardar metadata Native antes de validar runtime completo.
- No crear MSI/RC de #8 sin una prueba end-to-end real.
- No tratar `docker compose up` como prueba suficiente de disponibilidad.
- No introducir una cola global de comandos. Los flujos deben estar guiados por demanda y disponibilidad de recursos.

## Problemas reales observados

- El intento de migración falló por `os error 5` y privilegios al crear symlink.
- La prueba de symlink falló en sesión no elevada. Eso no prueba que WSDD elevado falle, pero sí prueba que el preflight debe ocurrir antes de tocar runtime o filesystem.
- Se eliminó/recreó runtime demasiado pronto durante pruebas anteriores.
- Hubo confusión entre nombres `PHP83` y `PHP8.3`; el código debe separar etiqueta de metadata y etiqueta real de contenedor.
- Se validó el runtime demasiado pronto, antes de confirmar volumen, montaje y servicios.
- El rollback terminó pareciendo el problema, pero en varios casos solo fue la consecuencia de una transición previa mal validada.
- Residuos de proyectos Native fallidos pueden contaminar `options.phpXX.yml` y provocar errores de volumen externo inexistente.

## Principio de arquitectura

El flujo correcto es por estados confirmados:

1. Un paso solo puede ejecutarse si sus prerequisitos están disponibles.
2. Un comando exitoso no equivale a recurso listo.
3. Cada transición debe tener una postcondición observable.
4. La metadata persistente solo cambia al final del flujo exitoso.
5. El rollback debe partir del último estado confirmado, no de supuestos.

## Fase 0 - Auditoría desde v2.0.1

Objetivo: entender el estado estable antes de agregar #8.

Tareas:

- Mapear el deploy actual desde `src/handlers/deploy.rs`.
- Mapear `runtime_storage` actual para `Classic Bind`, `Fast Sync Workspace` y `Native Linux Project`.
- Confirmar cómo se genera `options.phpXX.yml`.
- Confirmar cómo se crea/usa el volumen por proyecto.
- Confirmar cómo backup/restore decide qué carpeta archivar/restaurar.
- Documentar invariantes del runtime PHP compartido por versión.

Criterio de salida:

- Documento corto con flujo actual y puntos de extensión.
- Sin cambios de comportamiento.

## Fase 1 - Preflight no destructivo de storage administrado

Objetivo: validar si el entorno puede soportar el alias Windows hacia WSL antes de tocar proyectos reales.

Tareas:

- Detectar o crear de forma explícita la distro/storage administrado solo bajo confirmación del usuario.
- Validar acceso a:
  `\\wsl.localhost\WSDD-Storage\home\wsdd\projects`
- Crear un directorio temporal en storage WSL.
- Crear un alias/symlink temporal desde una ruta Windows temporal hacia ese UNC.
- Escribir y leer un archivo por:
  - alias Windows;
  - UNC WSL;
  - ruta Linux dentro de WSL.
- Eliminar solo los artefactos temporales creados por el preflight.

Regla crítica:

Si este preflight falla, no se debe crear backup, copiar proyecto, detener contenedor, eliminar volumen, renombrar carpeta ni guardar metadata.

Criterio de salida:

- Preflight reporta éxito/fallo con causa accionable.
- No modifica proyectos existentes.

## Fase 2 - Crear proyecto nuevo directamente en Native Linux administrado

Objetivo: soportar deploy desde cero usando storage administrado, sin migrar proyectos existentes.

Tareas:

- Extender metadata solo con campos mínimos necesarios:
  - `storage_mode`
  - `windows_alias_path`
  - `wsl_unc_path`
  - `linux_runtime_path`
- Crear el proyecto nuevo en `/home/wsdd/projects/<project>`.
- Crear alias Windows si el preflight ya demostró soporte.
- Generar volumen/mount usando ruta Linux real, no UNC.
- Mantener compatibilidad con el runtime PHP compartido por versión.
- Guardar metadata solo después de validar:
  - volumen existe;
  - contenedor está running;
  - mount exacto existe;
  - ruta de proyecto es legible;
  - vhost responde;
  - servicios esperados del contenedor están disponibles.

Criterio de salida:

- Proyecto nuevo Native administrado funciona end-to-end.
- `Classic Bind` sigue funcionando sin cambios.
- No existe flujo de migración.

## Fase 3 - Backup/restore compatible

Objetivo: evitar que backup/restore archive o restaure solo el alias.

Tareas:

- Para storage administrado, resolver el origen real autoritativo antes de archivar.
- Incluir metadata suficiente para restaurar:
  - modo de storage;
  - alias Windows;
  - UNC WSL;
  - ruta Linux runtime.
- Restaurar primero en backend real y luego recrear alias.
- Validar inventario y hashes cuando aplique.

Criterio de salida:

- Backup de proyecto Classic sigue igual.
- Backup de Native administrado contiene archivos reales.
- Restore no crea stubs sin contenido.

## Fase 4 - Diagnósticos y limpieza segura

Objetivo: hacer visible el estado sin acciones destructivas.

Tareas:

- Diagnóstico de proyectos con metadata inconsistente.
- Diagnóstico de `options.phpXX.yml` con volúmenes externos inexistentes.
- Reporte de contenedores/volúmenes esperados vs reales.
- Botón o acción de reparación solo con confirmación explícita.

Criterio de salida:

- WSDD puede explicar qué está roto antes de intentar repararlo.

## Fase futura - Migración Classic Bind a Native Linux

No implementar hasta que las fases anteriores estén probadas en uso real.

Condiciones mínimas antes de reabrir migración:

- Preflight de alias pasa desde WSDD instalado/elevado.
- Deploy desde cero Native administrado funciona.
- Backup/restore Native administrado funciona.
- Diagnóstico detecta residuos sin romper runtime.
- Existe script/manual de recuperación validado.
- El usuario confirma explícitamente que se reabre migración.

## Matriz mínima de validación

- `Classic Bind` nuevo con PHP 8.3.
- `Classic Bind` nuevo con PHP 8.4.
- `Native Linux administrado` nuevo con PHP 8.3.
- `Native Linux administrado` nuevo con PHP 8.4.
- Dos proyectos en la misma versión PHP sin romper runtime compartido.
- Proyecto con SSL.
- Backup y restore de proyecto Classic.
- Backup y restore de proyecto Native administrado.
- Reinicio de WSDD sin perder metadata.
- Reinicio de Docker Desktop sin residuos de volumen externo inexistente.

## Riesgos abiertos

- Windows puede no permitir symlink a UNC WSL sin elevación o Developer Mode.
- Docker Desktop puede requerir reglas específicas para montar rutas Linux de una distro WSL administrada.
- La ruta UNC puede comportarse distinto en IDEs, Git, watchers y herramientas de build.
- El runtime PHP compartido por versión exige exclusión por recurso, no por cola global.
- Un estado parcial puede contaminar `options.phpXX.yml` y bloquear proyectos sanos.

## Regla de release

No generar MSI/RC para #8 hasta que exista validación end-to-end real de las fases implementadas. Las pruebas `cargo fmt`, `cargo check`, `cargo clippy` y `cargo test` son necesarias, pero no suficientes para declarar lista esta funcionalidad.

## Validacion inicial y correccion posterior

Validado manualmente el 2026-06-21 con `inspinia-theme` y PHP 8.4:

- aprovisionamiento de `WSDD-Storage`;
- preflight de escritura y lectura por alias Windows, UNC y WSL;
- ZIP previo validado;
- copia temporal validada por inventario y SHA256;
- conservación de la carpeta Windows original;
- alias Windows hacia el storage WSL;
- volumen Docker recreado para renovar el relay interno de Docker Desktop;
- runtime PHP, vhost y acceso HTTP funcionales.

Esta validacion no cubrio un reinicio posterior de Docker Desktop. Tras reiniciar el equipo,
los contenedores PHP no pudieron arrancar porque el volumen `local` con `device=UNC` dependia
de un relay efimero bajo `/run/desktop/mnt/host/wsl/docker-desktop-bind-mounts`.

Correccion local en validacion:

- Managed deja de crear volumenes `local` respaldados por UNC.
- `options.phpXX.yml` usa un bind directo desde
  `/home/wsdd/projects/<project>` hacia `/var/www/html/<domain>.dock`.
- Cuando una version PHP contiene al menos un bind Managed, Compose se ejecuta dentro de
  `WSDD-Storage` mediante la integracion WSL de Docker Desktop.
- Los volumenes UNC legados se conservan durante la primera prueba para no romper rollback.
- El modo `NativeLinuxProject` conserva su comportamiento independiente.

Prueba aislada inicial:

- contenedor `wsdd-managed-bind-probe`;
- `restart: always`;
- origen registrado por Docker:
  `/home/wsdd/projects/.wsdd-runtime-probe`;
- lectura/escritura confirmada desde contenedor, WSL y UNC;
- despues de reiniciar el equipo, el probe quedo `exited (127)`, `restartCount=0`;
- Docker conservo el source visible `/home/wsdd/projects/.wsdd-runtime-probe`, pero el error
  OCI revelo que internamente seguia dependiendo de
  `/run/desktop/mnt/host/wsl/docker-desktop-bind-mounts/WSDD-Storage/<hash>`;
- el relay no existia despues del reinicio y Docker no lo reconstruyo automaticamente.

La migración de proyectos WSDD existentes desde `Classic Bind` continúa eliminada y queda fuera
del alcance. La opcion de bind Linux directo mediante la integracion WSL de Docker Desktop queda
descartada como solucion persistente por si sola.

## Recuperacion automatica implementada localmente

La solucion adoptada conserva el contrato de volumenes UNC y agrega una tarea programada
independiente de la interfaz WSDD:

- worker `wsdd-runtime-recovery.ps1`;
- targets Managed y Native WSL;
- mutex global y `IgnoreNew`;
- triggers de logon, desbloqueo, evento de `com.docker.service` y repeticion cada dos minutos;
- espera de hasta 180 segundos por readiness real de Docker;
- circuito persistente que bloquea reconstrucciones tras tres fallos consecutivos;
- reconstruccion permitida solo ante evidencia explicita de relay WSL roto;
- permisos runtime Managed reparados automaticamente despues de levantar PHP;
- accion manual `Fix Permissions` disponible para proyectos Native y Managed;
- propietario Linux preservado, grupo `www-data`, `setgid` y ACL por defecto solo sobre
  directorios runtime conocidos; nunca se aplica `chmod 777` al proyecto completo;
- intento de `docker start` antes de recrear;
- recreacion exclusiva de volumenes WSL y contenedor PHP afectado;
- `--no-build --force-recreate`;
- salida normal cuando no existen targets o Docker esta cerrado.

Validacion inicial del 2026-06-21:

- PHP 8.3 y PHP 8.4 fueron detectados con relay roto;
- ambos runtimes fueron rehidratados correctamente;
- una segunda ejecucion reporto `Healthy`;
- IDs y fechas de creacion no cambiaron;
- dos disparos simultaneos de Task Scheduler no ejecutaron workers paralelos.

## Cierre para 2.1.0

Issue #8 se considera cerrado para `v2.1.0` con estas condiciones:

- Managed Native Linux Storage es una opcion explicita, no el modo por defecto.
- La conversion ocurre durante el alta de un proyecto desde una carpeta Windows; no se ofrece
  migracion de proyectos Classic ya registrados.
- El ZIP validado es el snapshot inmutable usado para poblar WSL.
- Los archivos bloqueados y diferencias de inventario reportan rutas concretas.
- La carpeta Windows original y el ZIP permanecen preservados.
- El relay efimero de Docker Desktop no se considera persistente; la tarea programada lo
  diagnostica y rehidrata.
- El circuito de seguridad detiene reconstrucciones automaticas despues de tres fallos.
- Los permisos runtime se limitan a rutas conocidas, con grupo `www-data`, setgid y ACL.
- `Classic Bind` permanece como opcion de maxima compatibilidad.

Necesidades tecnicas incorporadas para cerrar el issue:

- WSL 2 y una distro administrada Debian 13 slim.
- Docker Desktop con backend WSL2 e integracion de `WSDD-Storage`.
- PowerShell 7.5+ para el worker de recuperacion.
- Task Scheduler con ejecucion elevada y oculta.
- Paquete Linux `acl` dentro de la distro administrada.
- Soporte Windows para symlink de directorio hacia `\\wsl.localhost`.
- Backup ZIP, inventario por SHA256 y rollback conservador.
