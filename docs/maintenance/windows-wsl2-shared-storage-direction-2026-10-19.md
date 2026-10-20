# Direccion vigente: almacenamiento compartido Windows ↔ WSDD sobre WSL 2

Estado: investigacion abierta. Issue propietario: [#14](https://github.com/wnunezc/wsdd-rust/issues/14). Relacionados: [#10](https://github.com/wnunezc/wsdd-rust/issues/10), [#11](https://github.com/wnunezc/wsdd-rust/issues/11) y [#12](https://github.com/wnunezc/wsdd-rust/issues/12).

Este documento registra la decision de producto y la evidencia local discutidas el 2026-10-19 y 2026-10-20. El codigo sigue siendo la fuente de verdad del comportamiento implementado. No declara implementada ni validada una segunda opcion de montaje.

## Requisitos para esta investigacion

1. Docker Desktop sigue ejecutando contenedores Linux mediante WSL 2. Se conserva la conexion Docker que hoy funciona en la instalacion local.
2. `Classic Bind` permanece como opcion predeterminada, sin conversion automatica ni cambios destructivos sobre proyectos existentes.
3. El directorio activo del proyecto permanece en su ruta NTFS ordinaria de Windows. Windows y contenedor leen y escriben los mismos archivos en tiempo real, sin copia activa paralela ni sincronizacion manual como requisito del flujo diario.
4. Detener, recrear o reconstruir el contenedor no elimina los archivos; WSDD vuelve a conectar el mismo directorio compartido.
5. Investigar como maximo una alternativa adicional que cumpla los requisitos anteriores y mejore de forma medida la lectura desde el contenedor. La UI final para este flujo debe mostrar como maximo `Classic Bind` y esa alternativa validada; no agregar una opcion por su rendimiento teorico ni eliminar prematuramente capacidades existentes.
6. Evaluar todo el stack WSDD: proyectos PHP, datos y logs MySQL, configuracion/SSL del proxy, directorios auxiliares PHP y otros montajes administrados. La estrategia de persistencia puede variar por tipo de dato, pero toda migracion futura debe ser explicita, verificable y recuperable.
7. Medir los mismos archivos y operaciones antes/despues; comprobar lectura, escritura, visibilidad bidireccional, recreacion del contenedor y funcionamiento de un proyecto real. La UI debe explicar las opciones con texto legible y fiel a su comportamiento.

## Evidencia local y alcance

- El volumen `php84-ipandetec.dock` usa `local` con `type=none`, `o=bind` y `device=D:\OpsZone\DevWorkspace\Projects\Web\moodle-ipandetec\moodle`; su nombre de volumen no convierte el origen NTFS en almacenamiento Linux nativo.
- En una prueba de lectura de los mismos 300 archivos pequeños (731.204 bytes en total, rutas relativas y SHA-256 cotejados), desde el contenedor PHP 8.4 el pase caliente tardó aproximadamente 1,09–1,13 s para `filesize` y 2,94–2,99 s para abrir/leer; el PHP de Windows tardó 7,7–9,1 ms y 20,5–20,7 ms, respectivamente. Esto confirma una penalizacion importante al acceder al montaje NTFS desde el contenedor. No cuantifica por si solo la causa completa de la demora de Moodle.
- La instalacion observada tambien monta directamente `C:\WSDD-Environment\Docker-Structure\bin\mysql\data` en MySQL, y monta configuracion/SSL de Nginx y directorios auxiliares PHP desde Windows. Su impacto individual no se ha medido en esta investigacion.
- El codigo actual de `Classic Bind` resuelve la ruta Windows como origen y crea un volumen `local` con `type=none`, `device=<ruta>`, `o=bind` (`src/handlers/deploy/runtime_storage.rs`).

### Prueba de VirtioFS en la instalacion local

- Con WSL 3.0.3.0, Docker Desktop 4.95.0 y `virtiofs=true` en `.wslconfig`, el montaje de `C:` dentro de `docker-desktop` aparecio como `virtiofs`. No se cambio el codigo de WSDD para activar esta prueba.
- En el bind mount de logs MySQL, un proceso con UID/GID `999:999` creo un archivo, pero `stat` mostro `0644 0:0`; el mismo usuario no pudo reabrirlo para escribir. `chown 999:999` como root devolvio exito sin cambiar la propiedad observada. Una prueba aislada en `/mnt/host/d` tambien mantuvo `0:0` tras `chown`. Se retiraron los archivos de prueba.
- El archivo persistente `binlog.index` seguia presente en Windows, pero aparecia como `0640 0:0` en el contenedor. MySQL no podia leerlo como UID/GID `999:999` y entraba en reinicios con `Permission denied`. No se borro ni repoblo el directorio de datos.
- La reproduccion se realizo durante una captura ETW oficial con perfil `storage`, [adjunta al issue microsoft/WSL#41991](https://github.com/microsoft/WSL/issues/41991#issuecomment-6283472190). Los detalles y limites tambien constan en el [comentario de WSDD #14](https://github.com/wnunezc/wsdd-rust/issues/14#issuecomment-6283517251).
- No se midio una mejora de rendimiento con VirtioFS porque no supero la prueba funcional del stack. Tampoco se hizo una comparacion controlada con 9p en la misma instalacion. El fallo no esta atribuido con certeza a WSL, Docker Desktop ni la imagen MySQL; la prueba fuera del contenedor impide atribuirlo solo a esa imagen.
- Se escribio `virtiofs=false` en `.wslconfig` para revertir la prueba, pero el posterior `wsl --shutdown` quedo bloqueado. La aplicacion del cambio y la recuperacion de Docker Desktop requieren verificacion despues de reiniciar Windows. No se debe declarar completada la reversión con la sola edicion del archivo.

## Opciones descartadas para **este objetivo**

| Opcion | Motivo | Estado del codigo existente |
| --- | --- | --- |
| Docker VMM o backend Hyper-V de Docker Desktop | El usuario requiere conservar WSL 2 como motor. VMM cambia el motor global; no es un modo de almacenamiento seleccionable por proyecto. | No se crea el boton `Test feature` para VMM, no se cambia Docker ni se elimina la conexion actual `2375`. No se ha demostrado incompatibilidad de VMM con ese puerto. |
| `Managed Native Linux Storage` como sustituto de `Classic Bind` | Traslada el origen activo a WSL y deja un alias Windows/UNC. No conserva una carpeta NTFS ordinaria como origen activo; #10 documenta diferencias de PHP/Git desde Windows. | La funcionalidad existente no se borra. #11 sigue siendo una necesidad de salida/retorno segura para proyectos Managed existentes; esta decision no lo resuelve. |
| `Native Linux Project` como segunda opcion buscada | Exige que el proyecto activo resida en Linux/WSL y cambia el flujo de trabajo Windows requerido. | Se conserva como capacidad existente; no se propone como solucion de este issue. |
| `Fast Sync Workspace` actual o una copia Linux mantenida por sincronizacion | Ejecuta otra copia y no ofrece la misma visibilidad inmediata bidireccional que el directorio compartido requerido. | Se conserva como capacidad existente; no se presenta como equivalente a un montaje en tiempo real. |
| Docker Desktop Synchronized file shares | La documentacion oficial actual no lo ofrece para el backend WSL 2 de Windows. Requiere cache/sincronizacion y no se ha validado contra la semantica exigida. | No se implementa ni se anuncia como segunda opcion. |
| Aumentar el timeout del proxy como solucion al I/O | No reduce el coste de lectura del montaje. | #12 conserva su diagnostico propio; las demoras de Moodle no se atribuyen exclusivamente al montaje sin medicion causal. |

Estas exclusiones acotan **la busqueda actual**; no piden retirar modos ya publicados ni reescribir historiales de los issues #4 y #8.

## Siguiente decision tecnica

Buscar y probar dentro de Docker Desktop + WSL 2 un mecanismo de montaje que conserve el directorio NTFS Windows como unico origen activo y ofrezca acceso inmediato en ambos sentidos. Compararlo con `Classic Bind` sobre el mismo proyecto y comprobar el resto de montajes del stack. Si ninguna opcion demuestra simultaneamente compatibilidad, persistencia y mejora de I/O, mantener `Classic Bind` como unica opcion recomendada y documentar el limite tecnico; no inventar una segunda opcion.

Esta investigacion no cambia codigo ni recursos empaquetados de WSDD. La prueba y el intento de reversión si cambiaron la configuracion local de WSL indicada arriba; no son una funcionalidad de la proxima release.

Referencias de producto: [Docker VMM](https://docs.docker.com/desktop/features/vmm/) y [disponibilidad de Synchronized file shares](https://docs.docker.com/desktop/settings-and-maintenance/settings/).
