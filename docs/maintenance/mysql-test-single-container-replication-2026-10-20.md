# MySQL Test: principal efímero y réplica persistente en un contenedor

## Alcance

Prueba **manual y aislada** para verificar si un principal MySQL que escribe en el filesystem Linux de un contenedor puede replicar sus transacciones a una segunda instancia MySQL cuyos datos permanecen en un bind mount NTFS de Windows. No sustituye `WSDD-MySql-Server`, no participa en `wsdd-core`, no expone puertos y no se despliega durante el arranque normal de WSDD. No requiere VirtioFS.

| Elemento | Ubicación / identidad |
|---|---|
| Contenedor | `WSDD-MySql-Server-Test` |
| Proyecto Compose | `wsdd-mysql-test`, archivo `C:\WSDD-Environment\Docker-Structure\bin\mysql-test\compose.yml` |
| Principal | `/opt/wsdd-mysql-primary`, capa escribible del contenedor; se pierde al recrear |
| Réplica | `/var/lib/mysql`, bind a `C:\WSDD-Environment\Docker-Structure\bin\mysql-test\data` |
| Red | red privada de este proyecto Compose, sin puertos publicados; ambas instancias escuchan solo en `127.0.0.1` dentro del contenedor |
| Imagen | `mysql:8.4.11` para ambas instancias; se omite el entrypoint de una sola instancia de la imagen |

La etiqueta `VOLUME /var/lib/mysql` de la imagen queda cubierta por el bind explícito **solo para la réplica**. El principal no usa un volumen Docker. Los archivos fuente del experimento se guardan en `recursos/recursos/Docker-Structure/bin/mysql-test/` y en `recursos/docker-structure.zip`. Al pulsar una acción, WSDD sincroniza exclusivamente estos cinco archivos administrados en la instalación existente; nunca copia ni borra `data` y no edita `init.yml` ni el MySQL principal.

## Protocolo implementado

1. **Deploy inicial:** si `data` está vacío, MySQL inicializa el principal en Linux con `lower_case_table_names=1`. Con el servidor apagado copia el datadir a la réplica, elimina `auto.cnf` en esa copia para obtener UUID distinto y arranca ambas instancias. Crea un usuario de replicación con contraseña aleatoria guardada en un archivo del directorio persistente; configura GTID y `GET_SOURCE_PUBLIC_KEY=1` para `caching_sha2_password`. Espera que los hilos IO y SQL estén activos. Si encuentra datos no reconocibles, se detiene sin borrarlos.
2. **Validate:** comprueba estado de ambas instancias, escribe una fila con token aleatorio en el principal, espera la aplicación de su GTID en la réplica y lee esa fila en la réplica. Guarda el token como testigo persistente. En validaciones posteriores exige que el testigo anterior siga en ambos servidores.
3. **Recreate:** exige el contenedor activo y la replicación saludable. Primero construye la imagen de reemplazo **sin parar MySQL**. Después pone el principal en `super_read_only`, espera a que la réplica aplique todos los GTID ejecutados, detiene replicación y apaga ambas instancias. Solo tras la confirmación `READY` ejecuta `docker compose up --force-recreate --no-build` para este único servicio. El nuevo contenedor arranca la réplica persistente, la recupera y la apaga limpiamente; copia su datadir al principal nuevo, cambia su UUID, arranca ambos, restablece replicación y verifica el testigo guardado. Si no hay confirmación, WSDD no continúa a la recreación. Si Compose falla después de apagar, `data` permanece intacto y **Deploy** puede reactivar el contenedor preparado o crearlo otra vez si fue retirado.
4. **Deploy de contenedor ya existente:** si está activo y saludable, verifica sin recrearlo; si quedó preparado después de una recreación interrumpida, lo reinicia; si está detenido, lo inicia conservando su capa escribible y la configuración de replicación existente. El script corregido detecta una réplica que aún no tiene fuente y completa la configuración sin vaciar el datadir. Si el contenedor fue eliminado, crea uno nuevo y restaura desde la réplica persistente. En caso de parada inesperada, la copia recuperable alcanza las transacciones **ya aplicadas** por la réplica; las que aún no llegaron pueden perderse.

La configuración usa binlogs con nombres relativos para permitir copiar el datadir entre rutas Linux distintas. Ambas instancias fijan `lower_case_table_names=1` **antes** de inicializar, porque ese ajuste no puede cambiarse después y el bind NTFS puede ser insensible a mayúsculas. La réplica conserva binlog de las transacciones recibidas para que pueda actuar como fuente del principal restaurado.

## Prueba manual en la aplicación

1. Abrir **Tools → MySQL Test · réplica NTFS**. Comprobar que `WSDD-MySql-Server-Test` aparece ausente y que el MySQL principal conserva su estado.
2. Pulsar **Deploy**. Esperar las etapas y `healthy`. Abrir el log si falla; inspeccionar el log del contenedor desde el panel de contenedores si es necesario.
3. Pulsar **Validate**. Debe informar `Replication verified` y guardar el testigo en `data/.wsdd-test-last-probe`.
4. Confirmar que los archivos de réplica existen en la ruta Windows indicada por el diálogo. No modificar esos archivos mientras MySQL está activo.
5. Pulsar **Recreate**. Deben completarse la espera de GTID, el apagado, la recreación y la verificación de salud. Se debe ver un ID de contenedor distinto y el testigo previo debe seguir presente.
6. Pulsar **Validate** de nuevo. Debe comprobar el testigo anterior y replicar una fila nueva.
7. Comprobar que `WSDD-MySql-Server`, su datadir `bin/mysql/data`, proxy y PHP no se alteraron.

El diálogo muestra etapas, progreso de BuildKit, botón para abrir el log y botón para copiarlo. Cada acción es explícita; ninguna despliega el experimento al abrir WSDD.

## MSI local para la prueba manual

El MSI inicial `wsdd-2.2.0-mysql-test-2026-10-20-x86_64.msi` (SHA-256 `4FB1436C1EEA020B423BF547416151542FBFDE2D32A733B26043676279457E25`) produjo el fallo descrito abajo y queda **descartado para nuevas pruebas**. Los instaladores tienen nombres y checksums distintos. Son compilaciones locales de prueba de la línea 2.2.0, no releases publicadas.

MSI `fix1` (superado): `D:\OpsZone\DevWorkspace\Engines\Rust-Resource\cargo-targets\WSDD\wix\wsdd-2.2.0-mysql-test-fix1-2026-10-20-x86_64.msi`. Tabla MSI: `ProductVersion=2.2.0`, `wsdd.exe` y `wsdd-launcher.exe`; tamaño 27 205 632 bytes; SHA-256 `432FB1604C82338B4A673967D976C24CD87DA0FFBD007CCB67C8E2B12E192B83`. Manifiesto `.sha256` junto al instalador. El ejecutable release contiene el script corregido y su SHA-256 es `CD563E6F420848B9EC5F4966F4C430DEEB5F734F87FD106EBD709DE96F33D00D`.

El MSI `fix1` quedó superado por el fallo de comprobación descrito abajo y **no debe utilizarse para nuevas pruebas**. El MSI `fix2` está en `D:\OpsZone\DevWorkspace\Engines\Rust-Resource\cargo-targets\WSDD\wix\wsdd-2.2.0-mysql-test-fix2-2026-10-20-x86_64.msi` (27 205 632 bytes, SHA-256 `BC0BD76366CC9FD279C0BF1E6B3D32C6A0A6021B4A229AD12DB1B797E172BF98`). Su manifiesto `.sha256` está junto al instalador y se verificó. La tabla MSI contiene `ProductVersion=2.2.0`, `wsdd.exe` y `wsdd-launcher.exe`. El ejecutable release tiene SHA-256 `DC82E7F1C3EE35295867682BCBF1A290F78D65CE505AFB70EE93D62E7493AC50` y contiene `sql_replica_status` y `--vertical --execute`.

**Archivo de artefactos (2026-10-20):** los MSI locales y sus checksums se movieron a `D:\OpsZone\DevWorkspace\Trash\WSDD-MSI-2026-10-20\`. Las rutas de WiX mencionadas arriba registran dónde se generaron originalmente; el inventario y SHA-256 de cada archivo están en `manifest.json` dentro de Trash. Los recursos fuente y el ZIP embebido permanecen en el proyecto, por lo que el MSI puede regenerarse.

## Primer Deploy y corrección

La primera prueba manual del usuario construyó la imagen y arrancó ambas instancias MySQL 8.4.11, pero el contenedor salió con código 1 durante «Verificación de ambas instancias». MySQL rechazó `CHANGE REPLICATION SOURCE TO` con `ERROR 3056`: el script generó una contraseña de 48 caracteres con `openssl rand -hex 24`, mientras que `SOURCE_PASSWORD` admite como máximo 32. La réplica conservó un datadir inicializado y el archivo de contraseña de 48 caracteres; no se creó el testigo de validación. El segundo proceso también registró conflictos de puerto y socket de MySQL X Plugin, aunque alcanzó `ready for connections` en el protocolo clásico.

La corrección genera 32 caracteres (`openssl rand -hex 16`), reemplaza de forma localizada el secreto de 48 caracteres del primer intento y distingue una réplica sin fuente configurada de una que ya replica. Desactiva X Plugin en ambas instancias, porque este experimento solo usa el protocolo clásico y dos servidores en el mismo contenedor comparten el puerto X predeterminado. Se actualizó el ZIP embebido con los tres archivos fuente modificados y se verificaron sus 97 entradas contra el árbol de recursos.

El usuario retiró **solo el contenedor de prueba** manualmente y conservó `C:\WSDD-Environment\Docker-Structure\bin\mysql-test\data`. El siguiente Deploy recuperó el principal desde esa réplica, sustituyó el secreto inválido y conectó la replicación. Un contenedor existente conserva el script de su imagen; actualizar archivos en Windows por sí solo no lo corrige.

## Segundo Deploy y corrección

El Deploy con `fix1` reconstruyó la imagen, recuperó el principal desde el datadir persistente y arrancó ambas instancias. El log confirmó `Replica receiver thread ... connected to source`, pero terminó `Replication did not become healthy` tras 77,1 s. Durante una repetición controlada por el usuario, una lectura SQL sin modificar Docker ni las bases devolvió `Replica_IO_Running: Yes`, `Replica_SQL_Running: Yes`, `Last_IO_Errno: 0`, `Last_SQL_Errno: 0` y `Executed_Gtid_Set: ...:1-3`. La replicación estaba activa; falló la comprobación de salud.

La causa está en `wsdd-mysql-test`: `status()` buscaba los campos `Replica_IO_Running` y `Replica_SQL_Running` en `SHOW REPLICA STATUS`, pero la consulta usaba `--skip-column-names`. El cliente MySQL elimina esos nombres también en salida vertical. Se añadió una consulta de estado sin esa opción y se usa tanto para reconocer una fuente existente como para verificar ambos hilos. Si se agota la espera, ahora muestra estados y códigos de error, sin imprimir credenciales. La prueba aislada del script confirmó que `status()` acepta ambos `Yes` con los nombres visibles. El resultado del Deploy con `fix2` se documenta abajo; siguen pendientes **Validate → Recreate → Validate**.

## Deploy con `fix2`

El usuario inició otra prueba manual. El contenedor nuevo `98a0ea4dd9bb` contiene `sql_replica_status()` y alcanzó `healthy` después de recuperar el principal desde la réplica persistente. Una lectura SQL posterior mostró que el principal no tiene fuente de replicación; la réplica informó `Replica_IO_Running: Yes`, `Replica_SQL_Running: Yes` y ambos códigos de error en `0`. El log inicial del principal mostró errores al cargar metadatos de relay heredados de la copia; el proceso continuó, `RESET REPLICA ALL` dejó al principal sin fuente y la réplica se conectó. Falta verificar **Validate → Recreate → Validate** y comprobar si esos errores de arranque requieren una limpieza previa en el protocolo de recuperación.

## Validate inicial y Recreate

El usuario ejecutó **Validate** antes de recrear: `primary=ready replica=ready replication=running` y `Replication verified: ad7ef5ddd2479e25b709b543d69c0a89` en 0,5 s. El testigo quedó en el bind NTFS. Después, **Recreate** construyó la imagen sin detener MySQL, esperó a que la réplica alcanzara los GTID, recibió `READY` tras apagar ambas instancias y sustituyó únicamente el contenedor de prueba (`98a0ea4dd9bb` → `c6358270d1d6`). La nueva instancia alcanzó `healthy` y WSDD completó «Validar restauración y replicación».

Verificación independiente de solo lectura: el archivo `.wsdd-test-last-probe` sigue en el directorio Windows con el mismo token; una consulta SQL encontró esa fila en principal y réplica (`COUNT(*)=1` en ambos); la réplica informó `Replica_IO_Running: Yes`, `Replica_SQL_Running: Yes`, `Last_IO_Errno: 0`, `Last_SQL_Errno: 0`; el principal no tiene fuente de replicación. Queda pendiente que el usuario pulse **Validate** nuevamente para comprobar una escritura y replicación posteriores a la recreación.

**Defecto observado en el arranque restaurado:** el nuevo principal registró errores `MY-010544`, `MY-011059`, `MY-010426` y `MY-010529` al cargar relay logs. El script copia el datadir completo de la réplica al principal, incluyendo metadatos de réplica, y ejecuta `RESET REPLICA ALL` solo después de iniciar el principal. El servidor pudo arrancar, el `RESET` dejó su `SHOW REPLICA STATUS` vacío y el testigo sobrevivió; estos errores todavía deben eliminarse del protocolo de recuperación antes de considerar la prueba robusta.

## Revisión local posterior: validación de estado antes y después de Recreate

La primera ampliación local de la validación tomaba un hash SQL **antes** de bloquear escrituras y escribía la matriz de prueba **antes** de revisar el testigo anterior. Se retiró esa ampliación sin empaquetarla. El orden actual del script es:

1. `Validate` comprueba salud y testigo previo; luego escribe una matriz aislada con Unicode, binario, decimal, `NULL`, actualización, eliminación y clave foránea. Espera los GTID en la réplica, relee la matriz en ambas instancias, compara las bases de usuario mediante exportación lógica con SHA-256 y solo entonces guarda el nuevo testigo.
2. `Recreate` construye primero la imagen sin detener MySQL. `prepare` exige un testigo, activa `super_read_only` en el principal, espera todos los GTID, compara las bases de usuario **después** de inmovilizar las escrituras y guarda el SHA-256 esperado en el bind persistente. Si falla cualquiera de esas comprobaciones, restablece la escritura y no autoriza la recreación.
3. El contenedor nuevo restaura el principal desde la réplica, reconecta la replicación y compara el estado lógico de ambas instancias con el SHA-256 persistido. Solo entonces retira el marcador de preparación y declara `healthy`. Una discrepancia deja el contenedor sin estado listo y conserva el marcador para diagnóstico.

La comparación incluye todas las bases de usuario con nombres ASCII compatibles con el protocolo de prueba; excluye los esquemas de sistema. No es un backup independiente ni prueba todas las posibles cargas concurrentes. La configuración del principal ahora fija el nombre del relay log y activa su recuperación para leer coherentemente los metadatos que llegan con la copia; **debe verificarse en el siguiente Recreate manual** que desaparezcan los errores `MY-010544`, `MY-011059`, `MY-010426` y `MY-010529`.

El script embebido identifica este protocolo como `wsdd-mysql-test:state-v2`. WSDD rechaza `Deploy`, `Validate` y `Recreate` de un contenedor de prueba anterior que siga ejecutándose, con una indicación para retirarlo conservando `bin/mysql-test/data` y volver a desplegar. WSDD no retira ni reemplaza ese contenedor automáticamente. Las 97 entradas del ZIP embebido fueron comparadas con las fuentes tras el cambio. El código pasó comprobación de sintaxis shell, `cargo fmt` y `cargo check`; la prueba manual con el MSI nuevo aún está pendiente.

**Orden de la siguiente prueba manual con el MSI nuevo:** retirar únicamente `WSDD-MySql-Server-Test` desde Docker, **conservar** `C:\WSDD-Environment\Docker-Structure\bin\mysql-test\data`, instalar el MSI, abrir WSDD y pulsar **Deploy → Validate → Recreate → Validate**. Guardar el log completo y comprobar que cambió el ID del contenedor, que aparecen los hashes de estado iguales y que el principal no registra errores de relay al arrancar. No ejecutar estas acciones sobre `WSDD-MySql-Server`.

**Resultado manual recibido (20-10-2026):** el usuario ejecutó `Validate` y luego `Recreate` con el MSI `state-v2`. El log adjunto de `Recreate` confirma el marcador `wsdd-mysql-test:state-v2`, imagen construida, réplica alcanzada y ambas instancias apagadas antes de `docker compose up --force-recreate`. La restauración emitió `Restored SQL state verified: 3328ec568beb3dc234d7e4188df9abdacc08710fb038f76392ca316063e76793` y terminó `healthy`. Lecturas posteriores sin modificar Docker: contenedor `1258797ef1ff` saludable; bind exclusivo `C:\WSDD-Environment\Docker-Structure\bin\mysql-test\data` → `/var/lib/mysql`; testigo persistente `25099a9289ce699c675e79ca68010e0b` presente exactamente una vez en principal y réplica; tablas de la matriz con 2 filas padre y 1 hija en ambos; GTID ejecutados iguales; hilos IO y SQL en `Yes` y errores `0`; principal sin canal de réplica; marcador `.wsdd-test-prepared` retirado. No aparecieron los errores de relay `MY-010544`, `MY-011059`, `MY-010426` ni `MY-010529`. Sí apareció `MY-010539`, aviso de recuperación de posición de relay, y los avisos `MY-010235` y `MY-010897`; no impidieron el arranque. **Falta el Validate posterior desde la UI** para completar la secuencia integrada; una escritura y su replicación posteriores se probaron directamente en la medición siguiente.

## Medición puntual de escritura y lectura tras Recreate

El usuario autorizó una prueba de escritura y lectura en ambos contenedores. Se creó en cada uno una base aislada `wsdd_io_probe_*`, sin tocar bases existentes. La carga, ejecutada primero en el principal experimental y luego en el MySQL habitual, fue idéntica: tres rondas de inserción de 10 000 filas × 1024 bytes (10 240 000 bytes), lectura SQL de todas las cargas con `COUNT(*)` y `SUM(OCTET_LENGTH(payload))`, y 100 inserciones de 1024 bytes con autocommit por ronda. Los tiempos provienen de `SYSDATE(6)` dentro de la misma conexión MySQL; no incluyen el arranque de `docker exec`. `TRUNCATE` y creación de tablas quedaron fuera del intervalo medido. Cada lectura devolvió 10 000 filas y 10 240 000 bytes.

| Operación | Principal experimental, tres rondas | MySQL habitual, tres rondas | Mediana experimental | Mediana habitual |
|---|---:|---:|---:|---:|
| Insertar 10 MB en una sentencia | 592, 582, 743 ms | 3085, 2964, 2950 ms | 592 ms | 2964 ms |
| Leer 10 MB con `SUM(OCTET_LENGTH)` | 2,1; 3,2; 2,7 ms | 2,5; 2,1; 3,1 ms | 2,7 ms | 2,5 ms |
| 100 inserciones con autocommit | 999, 1050, 1216 ms | 677, 587, 662 ms | 1050 ms | 662 ms |

La inserción masiva del principal experimental fue unas **5,0 veces más rápida** en esta carga. Las 100 transacciones pequeñas fueron unas **1,6 veces más lentas** en el experimental. Las lecturas fueron similares y se hicieron inmediatamente después de insertar: son **lecturas en caliente**, por lo que no miden lectura fría del filesystem. El contraste solicitado compara exclusivamente las operaciones SQL del **principal experimental** con las del **MySQL habitual**; no incluye tiempos de la réplica.

No es un experimento que aísle exclusivamente el montaje. El experimental ejecuta MySQL **8.4.11**, buffer InnoDB de **128 MB**, `innodb_doublewrite=ON` y no tiene límite de CPU/memoria de contenedor. El habitual ejecuta MySQL **26.7.0**, buffer de **1 GB**, `innodb_doublewrite=OFF`, límite de **2 CPU/2 GB** y escribe directamente en el bind NTFS. Ambos tenían `innodb_flush_log_at_trx_commit=1`, `sync_binlog=1` y binlog activo. Las cifras describen **estos dos despliegues actuales**, no la ganancia causal del filesystem por sí solo. Al finalizar se eliminaron únicamente las bases `wsdd_io_probe_*` de la prueba; se verificó que no quedaron esquemas de prueba. Ambos contenedores siguieron activos.

### Repetición con cinco veces más datos

A solicitud del usuario se repitió **solo la comparación entre los dos principales**, con el mismo SQL y orden de ejecución: tres rondas de **50 000 filas × 1024 bytes** (51 200 000 bytes) en una sentencia, lectura completa con `COUNT(*)` y `SUM(OCTET_LENGTH(payload))`, y **500 inserciones con autocommit** por ronda. Se midió con `SYSDATE(6)` dentro de cada servidor. Creación de tablas y `TRUNCATE` quedaron fuera de los intervalos. Antes de iniciar el MySQL habitual se comprobó que había terminado la actividad causada por la primera carga, para evitar solaparla con la segunda. No se incluyen mediciones de réplica.

| Operación | Principal experimental, tres rondas | MySQL habitual, tres rondas | Mediana experimental | Mediana habitual |
|---|---:|---:|---:|---:|
| Insertar 51,2 MB en una sentencia | 1,261; 1,887; 1,630 s | 14,780; 13,764; 14,274 s | **1,630 s** | 14,274 s |
| Leer 51,2 MB después de insertarlos | 283,9; 160,3; 329,5 ms | 11,7; 11,6; 11,0 ms | 283,9 ms | **11,6 ms** |
| 500 inserciones con autocommit | 4,971; 5,614; 5,591 s | 3,236; 3,461; 3,271 s | 5,591 s | **3,271 s** |

En esta carga la inserción masiva del principal experimental fue **8,76 veces más rápida**, pero la lectura SQL posterior fue **24,48 veces más lenta** y los commits individuales **1,71 veces más lentos**. Las lecturas se hicieron inmediatamente después de insertar y no representan lectura fría controlada del disco. Las diferencias de versión, buffer, `doublewrite` y límites de recursos siguen impidiendo atribuir los resultados a un único factor. Se retiraron las bases temporales `wsdd_io_probe5x_*`; se verificó que no quedaron en los principales ni en sus directorios de datos. Ambos contenedores permanecieron activos y el experimental `healthy`.

MSI local para esta prueba: `D:\OpsZone\DevWorkspace\Engines\Rust-Resource\cargo-targets\WSDD\wix\wsdd-2.2.0-mysql-test-state-v2-2026-10-20-x86_64.msi` (27 213 824 bytes, SHA-256 `D0B9096014DCF46EA2ED45B8107530CDED38C94EDCD526274ADEE4B43589BC5B`). El checksum está junto al archivo. Se verificó `ProductVersion=2.2.0`, la presencia de `wsdd.exe` y `wsdd-launcher.exe` y el marcador `state-v2` en el ejecutable. El paquete no se instaló ni se probó contra Docker durante esta revisión.

## Límites de esta prueba

- La réplica es una **base de datos activa**, no un respaldo independiente ni una instantánea consistente de archivos mientras MySQL escribe. Un backup exportable requiere un procedimiento separado, con copia consistente y prueba de restauración.
- Replicación asíncrona: la recreación solicitada por WSDD exige `GTID` al día; una caída abrupta puede perder las últimas transacciones del principal que la réplica no había aplicado.
- `fix2` completó Deploy, Validate inicial y Recreate; el testigo continuó en ambas instancias. La revisión local posterior espera prueba manual completa con el MSI nuevo, confirmación de los relay logs y medición de rendimiento frente al MySQL actual.
- El backup completo de WSDD copia el entorno mientras está activo; **no** debe tomarse como snapshot consistente de la réplica. Una restauración completa puede detener y retirar el contenedor de prueba; el bind `data` permanece en el entorno restaurado si existe en ese backup.
- Esta prueba no migra datos del MySQL existente ni conecta PHP/phpMyAdmin al nuevo contenedor. Su propósito es validar el mecanismo antes de decidir si se integra en el flujo normal.

## Evidencia técnica consultada

- [MySQL 8.4: `RESET REPLICA`](https://dev.mysql.com/doc/refman/8.4/en/reset-replica.html): conserva historial GTID pero elimina relay logs, incluso los no aplicados. Por eso la recreación espera primero a que la réplica alcance al principal.
- [MySQL 8.4: incorporar una réplica copiando el directorio de datos](https://dev.mysql.com/doc/refman/8.4/en/replication-howto-additionalslaves.html): indica conservar los relay logs y fijar un índice compatible al arrancar una copia con UUID nuevo.
- [MySQL 8.4: `mysqldump`](https://dev.mysql.com/doc/refman/8.4/en/mysqldump.html): documenta `--single-transaction` y `--order-by-primary`, usados en la comparación lógica de este experimento.
- [MySQL 8.4: configurar la fuente](https://dev.mysql.com/doc/refman/8.4/en/replication-howto-slaveinit.html): para `caching_sha2_password` sin TLS exige `GET_SOURCE_PUBLIC_KEY` o una clave pública local.
- [MySQL 8.4: sensibilidad de identificadores](https://dev.mysql.com/doc/refman/8.4/en/identifier-case-sensitivity.html): `lower_case_table_names` se decide al inicializar y el valor 1 permite trasladar datos entre filesystems con distinta sensibilidad a mayúsculas.
- [MySQL 8.4: UUID de servidor](https://dev.mysql.com/doc/refman/8.4/en/replication-options.html): sin `auto.cnf` se genera un UUID nuevo; fuente y réplica requieren UUID distintos.
- [MySQL 8.4: contraseñas para replicación](https://dev.mysql.com/doc/refman/8.4/en/assigning-passwords.html): `CHANGE REPLICATION SOURCE TO` limita efectivamente `SOURCE_PASSWORD` a 32 caracteres.
- [MySQL 8.4: opciones de X Plugin](https://dev.mysql.com/doc/refman/8.4/en/x-plugin-options-system-variables.html): X Plugin viene activado por defecto y `mysqlx=OFF` lo desactiva cuando no se usa.
- [MySQL Bug #47147](https://bugs.mysql.com/bug.php?id=47147): `--skip-column-names` suprime también los nombres de campo en salida vertical; es la causa del falso negativo observado.
