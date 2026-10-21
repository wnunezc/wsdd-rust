# MySQL Core: principal Linux y réplica persistente NTFS

## Estado y alcance

Integrado en el código de `v2.3.0` tras las pruebas locales posteriores a `v2.2.0`. El usuario instaló el MSI local e inició el Core; la validación operativa y el rendimiento medido se registran abajo. WSDD conserva el servicio Compose `wsdd-core/database`, el contenedor `WSDD-MySql-Server`, el puerto público de desarrollo `127.0.0.1:3306`, la red, phpMyAdmin y la ruta de persistencia `C:\WSDD-Environment\Docker-Structure\bin\mysql\data`. No se habilitó VirtioFS.

La nueva imagen fija `mysql:26.7.0` por digest. Ejecuta dos procesos MySQL dentro del **mismo** contenedor: principal en `/opt/wsdd-mysql-primary` (capa Linux efímera, puerto 3306) y réplica en `/var/lib/mysql` (bind NTFS existente, puerto interno 3307). La réplica solo escucha en loopback dentro del contenedor. Ambos usan `lower_case_table_names=1`, GTID y binlog. La réplica guarda su propio binlog para poder servir como fuente de recuperación. `Dockerfile`, `primary.cnf`, `replica.cnf` y `wsdd-mysql` son archivos administrados que WSDD sincroniza; la sincronización no toca `data`. El ZIP embebido contiene los mismos recursos.

## Datos locales preparados

El datadir anterior, inicializado con `lower_case_table_names=2`, se convirtió lógicamente en una fase previa y **no se reutilizó físicamente** en el nuevo formato. La comparación de la conversión registró 14 esquemas de usuario, 1771 tablas InnoDB y 93467 filas; los 1771 checksums de tabla coincidieron. El datadir convertido para MySQL 26.7.0 ya ocupa la ruta persistente original y tiene el marcador `.wsdd-core-import-approved`. El datadir crudo anterior se conservó aislado en `D:\OpsZone\DevWorkspace\Credentials\WSDD\mysql-lctn1-2026-10-20\legacy-datadir-lctn2-original`; el informe, dump ZIP y checksum están en ese mismo directorio protegido. El backup es una salvaguarda de la conversión; **no** debe incluirse en el paquete de instalación ni en un backup general de `C:\WSDD-Environment`.

No se creó ni inició ningún contenedor de producción durante la preparación. Al cerrar esta fase, `docker ps -a` estaba vacío. La primera ejecución de WSDD instalada sincronizará los recursos administrados y generará `init.yml` con sus credenciales; el usuario realizará el despliegue desde WSDD.

## Protocolo del contenedor

1. Si la réplica persistente está vacía, inicializa el principal con el entrypoint oficial MySQL, lo detiene, copia sus datos con ambos procesos parados y configura la réplica. Si la carpeta no está vacía y no contiene un datadir reconocible, falla sin borrarla.
2. Si el principal desapareció por recreación, exige un marcador de preparación, apagado limpio o importación explícita. Recupera la réplica, la detiene, copia el datadir al principal nuevo, retira su `auto.cnf` para obtener otro UUID, arranca ambas instancias y restablece la replicación. No borra la réplica.
3. Antes de ofrecer el principal para escrituras, verifica los hilos IO y SQL de replicación, el testigo de la última validación y, tras `Recreate`, el SHA-256 de un dump SQL completo del estado preparado. Solo entonces quita `read_only` y crea el marcador de salud.
4. `Validate` escribe una matriz SQL y un token en el principal, espera el GTID en la réplica, lee y comprueba ambas copias, compara el estado lógico completo y guarda el testigo. `Recreate` construye primero la nueva imagen, congela escrituras, espera la réplica, compara y guarda el hash, apaga las instancias y recrea **solo** `database`; el nuevo contenedor verifica la restauración antes de declararse saludable.
5. En una parada ordinaria, el script congela el principal, espera el GTID y apaga ambas instancias. Solo después de ambos apagados escribe el marcador de parada limpia. Si falla la barrera o el apagado, no deja un marcador falso de recuperación. El plazo de parada del contenedor es 300 s.

El panel **Docker → MySQL Core** ofrece `Deploy`, `Validate` y `Recreate` con etapas, progreso BuildKit y log abrible/copiable. El despliegue base normal también espera el healthcheck de ambas instancias. Un contenedor legado con el mismo nombre no se inicia ni se reemplaza automáticamente sobre el datadir convertido; hay guardas en el despliegue base, el panel y las operaciones generales de contenedores.

## Backup, restauración y límites

- La réplica NTFS es una **copia operativa**, no un backup independiente: comparte fallos lógicos y borrados propagados. El datadir principal vive en la capa del contenedor; retirarlo sin réplica al día puede perder las transacciones aún no aplicadas.
- El backup completo existente copia archivos. Para el nuevo layout, WSDD lo rechaza mientras MySQL está activo y exige una parada limpia antes de copiar la réplica. No se ha implementado un snapshot caliente consistente.
- La restauración completa existente superpone archivos y no reemplaza atómicamente un datadir. WSDD la rechaza si el entorno actual o el ZIP contiene el layout replicado **antes de detener contenedores o sobrescribir datos**. Recuperar ese ZIP requerirá un procedimiento verificado específico; no presentarlo como botón funcional para este layout.
- El Core de producción ya arrancó, pasó `Validate` por CLI antes y después de la carga descrita abajo y completó un `Recreate` manual desde WSDD. Aún no se ha probado el ciclo ordinario de parada/arranque en producción; phpMyAdmin respondió HTTP 200, pero no se ha probado una aplicación PHP real. El ensayo `mysql-test` tiene documentación separada. El usuario decidió publicar `v2.3.0` como estable con estos límites declarados en las notas; esta decisión no convierte las pruebas pendientes en verificaciones realizadas.

## Verificación tras la instalación del usuario (2026-10-20)

- Contenedores activos: `WSDD-MySql-Server` `healthy`, phpMyAdmin y proxy. Entrypoint del Core `/usr/local/bin/wsdd-mysql`, protocolo `state-v1`; el log muestra recuperación inicial de la réplica persistente y configuración de GTID.
- Consultas SQL independientes: principal MySQL **26.7.0**, `lower_case_table_names=1`, datadir `/opt/wsdd-mysql-primary/`; réplica MySQL **26.7.0**, `lower_case_table_names=1`, datadir `/var/lib/mysql/`, `read_only=1` y `super_read_only=1`. Ambos conservan **14 esquemas de usuario y 1771 tablas base** al excluir la base de validación de WSDD. El esquema temporal del benchmark ya no existe en ninguna instancia.
- `SHOW REPLICA STATUS`: hilos IO y SQL `Yes`, errores IO y SQL `0`. Antes de la carga, `Seconds_Behind_Source=0`. `wsdd-mysql validate` pasó antes de medir (`SQL SHA-256: 54adb7fe17f12735759897df1bd63be14ab39f264b83699e740606dd6a0ab458`) y después de retirar la base de prueba (`9e78ccbc1223031e49926fa3c94f1c8be29d235c105fc5908184641f5d672eed`). Esta acción crea y conserva su propia base testigo `wsdd_core_replication_probe`.
- Puerto `127.0.0.1:3306` accesible desde Windows y `https://pma.wsdd.dock/` respondió HTTP 200. Esto confirma disponibilidad de red y página; no sustituye una prueba de login ni de una aplicación.
- El usuario ejecutó **Recreate** desde WSDD. El log `C:\WSDD-Environment\logs\wsdd-operation-1792540442153-2.log` registra: construcción 2,1 s, espera de réplica y apagado 34,7 s con `READY`, recreación de contenedor 1,2 s y restauración/validación 116,3 s. Esta última etapa incluyó recuperación de la réplica, copia al principal, arranque de ambos MySQL y verificación de hash SQL (`fd30fc8aa54d355f9734704465bf8290e362b73a53347180fa991dd851207373`). Terminó `healthy`; lectura posterior: hilos IO/SQL `Yes`, retraso `0`, errores `0`. **Los 116,3 s no son retraso de replicación**. El plazo de esa etapa en Rust ya es de 600 s; los 60 s del script corresponden solo a esperar los GTID durante la preparación o parada limpia.

### Rendimiento: principal nuevo frente a mediciones anteriores

Se creó **solo** `wsdd_io_probe_core_20261020` en el principal. Tres rondas: una inserción de 50 000 filas × 1024 bytes (51,2 MB) generadas con productos de dígitos; lectura `COUNT(*)` y `SUM(OCTET_LENGTH(payload))` inmediatamente después; 500 `INSERT` individuales mediante un procedimiento. Se verificó `@@autocommit=1`, de modo que cada `INSERT` se confirmó individualmente según la [semántica de MySQL 26.7](https://dev.mysql.com/doc/refman/26.7/en/commit.html). Los tiempos se tomaron con `SYSDATE(6)` dentro de una conexión MySQL; crear tablas y `TRUNCATE` quedaron fuera de las ventanas. La réplica no se incluyó en las cifras de latencia del principal. Tras la carga, la réplica leyó 50 000 filas, 51 200 000 bytes y las 500 inserciones finales. El retraso observado llegó a **45 s**, después volvió a **0 s** con ambos hilos activos y errores `0`. Se eliminó exclusivamente el esquema temporal y se verificó su ausencia en ambas instancias.

| Operación | Core nuevo, tres rondas | Mediana nueva | Mediana MySQL antiguo* | Mediana Test 8.4.11* |
| --- | ---: | ---: | ---: | ---: |
| Inserción de 51,2 MB | 1,437; 1,770; 1,804 s | **1,770 s** | 14,274 s | 1,630 s |
| Lectura inmediata de 51,2 MB | 12,840; 24,376; 11,753 ms | **12,840 ms** | 11,6 ms | 283,9 ms |
| 500 inserciones autocommit | 3,708; 3,692; 3,745 s | **3,708 s** | 3,271 s | 5,591 s |

\* Las dos columnas históricas provienen del [ensayo anterior](mysql-test-single-container-replication-2026-10-20.md). Repetimos volumen y tipo de operación documentados; el SQL exacto del generador anterior no quedó preservado, así que **no son un benchmark idéntico controlado**. El Core nuevo hizo la inserción masiva ~8,1 veces más rápido que el MySQL antiguo y los 500 commits ~13 % más lentos. Las lecturas fueron en caliente, no miden I/O frío. Versiones, buffer, doublewrite, límites de recursos, replicación y carga del equipo también cambian; las cifras no aíslan por sí solas el efecto del filesystem.

## Prueba manual requerida tras instalar el MSI local

MSI local para esta prueba: `D:\OpsZone\DevWorkspace\Engines\Rust-Resource\cargo-targets\WSDD\wix\wsdd-2.2.0-mysql-core-2026-10-20-x86_64.msi` (27 377 664 bytes, SHA-256 `6B3AEB03151596EB2D9CACF98044ED2A78F05609C6EFD3123529912FBE30500E`). El archivo `.sha256.txt` está junto a él. La tabla MSI reporta `ProductVersion=2.2.0` e incluye `wsdd.exe` y `wsdd-launcher.exe`. Es un instalador de prueba local posterior a la release pública `v2.2.0`; no se publicó ni instaló por el agente.

Verificaciones locales: `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings` y 224 pruebas del workspace pasaron; sintaxis del script shell, 100 entradas del ZIP cotejadas por SHA-256 con sus fuentes y `docker compose config --quiet` pasaron. Ninguna de esas verificaciones arrancó MySQL Core ni demuestra el comportamiento de la réplica de producción.

1. Comprobar el backup de conversión y que `C:\WSDD-Environment\Docker-Structure\bin\mysql\data` conserva el datadir convertido. Abrir WSDD desde el MSI nuevo y usar **Docker → MySQL Core → Deploy** (o el asistente normal de arranque). Si falla, conservar contenedor y datos y abrir el log de la operación.
2. Confirmar `healthy`, la conexión de phpMyAdmin y una aplicación existente. Ejecutar `Validate` y guardar su token/hash.
3. Ejecutar `Recreate`: comprobar que cambia el ID del contenedor y que el log confirma restauración y hash. Ejecutar `Validate` otra vez; verificar una muestra de datos reales y la conexión de la aplicación.
4. Detener e iniciar MySQL desde WSDD; confirmar apagado limpio, restauración del servicio y datos. Tomar un backup completo **solo con MySQL detenido y limpio**; no usar todavía el botón de restauración completa para este layout.

Si cualquiera de esos pasos falla, no borrar `data` ni el backup aislado. Conservar el log y corregir a partir de la evidencia.
