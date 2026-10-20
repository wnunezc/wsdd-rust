# Cambios acumulados desde la v2.1.3 publicada

Este documento conserva el registro de preparación de la versión 2.2.0
de WSDD, publicada el 2026-10-20. La evidencia final está en
`2.2.0-validation.md` y las notas públicas en `2.2.0-draft.md`. El punto de
partida es el tag público `v2.1.3` (commit fuente
`c75d1493a1c55560ecdb7645cc825fd7e9e14d81`, publicado el 2026-10-15).
La evidencia y el checksum del MSI público pertenecen a
`2.1.3-validation.md`; **un MSI local posterior con ProductVersion 2.1.3 no
reemplaza ese asset**.

## Inventario posterior al tag (estado registrado el 2026-10-19)

| Commit | Estado | Cambio que debe considerarse |
| --- | --- | --- |
| `59bc48076b482370a7cf8f3a5588e1cad74a7662` | En `main` remoto | Sólo documenta y verifica la publicación de v2.1.3; no agrega comportamiento al programa. |
| `0a4fad624a68b5d4b913b79e0690c2016cd6dc30` | En `main` remoto | Guía de credenciales AS-IS y supresión de credenciales conocidas en diagnósticos de Rust, PowerShell, panel, archivos y copia. |
| `7503c5bd1d7fc977a361c95c694ad917236a8b68` | Commit local, sin push | Grupos Compose, guardas de propiedad, etapas de despliegue, seguimiento BuildKit, log por operación y corrección del falso positivo `phpadmin56.wsdd.dock`. |
| `a086622` | Commit local, sin push | Crea este registro acumulado de cambios desde v2.1.3. |
| `53226ea` | Commit local, sin push | Reorganiza Settings y Herramientas; agrega trazabilidad y log de despliegue para las utilidades. |
| `9094771` | Commit local, sin push | Retira el mapa histórico `MIGRATION.md` y repara enlaces públicos; no cambia el runtime. |
| `b1fa28b` | Commit local, sin push | Registra la aceptación local de Settings/Herramientas y el cierre del issue #16; no agrega otro cambio de runtime. |
| `0c9f873` | Commit local firmado, sin push | Documenta el alcance del almacenamiento Windows/WSL 2 y la prueba fallida de VirtioFS; no cambia el programa ni agrega un modo de montaje. |

La columna Estado refleja el corte histórico del 2026-10-19. Esos commits ya
se incluyeron en `v2.2.0`; los commits de publicación `ecd5d9f` y `2fb986f`
prepararon la versión y ajustaron la comparación de saltos de línea en una
prueba. El MSI público 2.1.3 sigue correspondiendo a su propio tag.

## Contenido funcional trasladado a las notas de v2.2.0

### Diagnósticos y credenciales

- Se documenta el uso AS-IS del stack LAMP y las relaciones entre credenciales
  MySQL, phpMyAdmin, Webmin, ngrok y aplicaciones. Fuente:
  `../development-credentials.md`.
- Los diagnósticos propios de WSDD ocultan campos y valores conocidos de
  credenciales en Rust, scripts PowerShell, logs y acciones de copia. Se
  conservan los datos originales para comandos y decisiones internas; el filtro
  no modifica la autenticación. Fuente:
  `../maintenance/development-credentials-and-log-suppression-2026-10-17.md`.
- Los nombres de usuario se ocultan como identificadores completos, para no
  alterar dominios como `phpadmin56.wsdd.dock`. Contraseñas y tokens conocidos
  siguen ocultándose donde aparezcan. Esto no sanea logs antiguos ni datos
  retenidos por Docker o aplicaciones externas.

### Organización de contenedores y recursos

- Docker Desktop agrupa proxy/MySQL/phpMyAdmin en `wsdd-core`, runtimes PHP en
  `wsdd-web` y utilidades opcionales en `wsdd-tools`. Cada utilidad se inicia o
  detiene de forma individual; nombres de contenedor, red, puertos y montajes
  conservan su contrato. Fuente: `../maintenance/compose-groups-2026-10-19.md`.
- WSDD comprueba la etiqueta Compose de un contenedor de nombre fijo antes de
  operaciones sensibles y rechaza propietarios incompatibles en lugar de
  adoptarlos o eliminarlos. Se actualizaron plantillas, sincronización de
  recursos administrados y recuperación WSL, junto con ambos ZIP embebidos.

### Experiencia de despliegue

- Deploy PHP y Deploy de proyectos muestran etapas con duración y resultado.
  El panel muestra el último paso de BuildKit observado sin exponer sus
  argumentos ni inventar porcentaje o ETA.
- La salida técnica puede desplegarse en la UI. Cada operación guarda un log
  redactado propio que se puede abrir en Notepad o copiar completo; el log
  diario anterior permanece. Fuente:
  `../maintenance/deploy-stage-trace-2026-10-19.md`.
- El flujo de proyectos mantiene un único paso de imagen/contenedor porque
  Compose ejecuta `up --build` como una sola operación. No cambian la semántica
  de Deploy ni el orden de rollback.

### Configuración y herramientas

- Settings muestra únicamente Idioma. Guardar conserva la configuración previa
  y no reescribe secretos, Compose ni `.wslconfig`.
- ngrok, Redis, Memcached y Mailpit pasan a diálogos propios en Herramientas:
  Guardar persiste opciones sin desplegar, Deploy aplica el servicio elegido y
  Detener para su contenedor sin borrar volúmenes. ngrok comprueba Agent API.
  Deploy muestra etapas medidas, log propio y acciones para abrirlo o copiarlo.
- Los ajustes de almacenamiento y recuperación WSDD pasan a Configuración WSL
  con guardado separado de `.wslconfig`.
- Se retiran de la UI los controles globales PHP/Xdebug, decorativos y matrices
  de credenciales MySQL/Webmin; las claves y los valores previos permanecen
  disponibles para los flujos existentes. Detalle campo por campo:
  `../maintenance/settings-tools-reorganization-2026-10-19.md`.
- Redis, Memcached y Mailpit aún carecen de validación integral en Docker real.
  Estos cambios entraron en el MSI público 2.2.0, no en el 2.1.3.
  La ronda local de formato, compilación, Clippy y 222 pruebas pasó. Se generó
  un MSI local posterior para probar este cambio, detallado en
  `../maintenance/settings-tools-reorganization-2026-10-19.md`.
- El usuario confirmó el 2026-10-19 que probó localmente la reorganización y
  el [issue #16](https://github.com/wnunezc/wsdd-rust/issues/16) se cerró como
  completado por su solicitud.
  Esta aceptación no cubre Redis, Memcached ni Mailpit: si aparece un fallo al
  probarlos, se tratará en un issue específico. La versión 2.2.0 incluye los
  cambios de Settings/Herramientas y la trazabilidad de despliegue de
  utilidades; el cierre del issue no valida esas tres herramientas en Docker real.

## Evidencia local previa a la publicación

- El commit `0a4fad6` pasó CI remoto con 211 pruebas. Los cambios locales
  posteriores pasaron formato, `cargo check`, Clippy con advertencias denegadas
  y 218 pruebas inicialmente; tras Settings/Herramientas pasaron 222 pruebas
  del workspace. Los ZIP embebidos coinciden por SHA-256 con sus
  fuentes: 92 archivos Docker y 13 PowerShell.
- Se generó un MSI **local** que combina estos cambios:
  `D:\OpsZone\DevWorkspace\Engines\Rust-Resource\cargo-targets\WSDD\wix\wsdd-2.1.3-pre-settings-2026-10-19-x86_64.msi`,
  SHA-256 `35FD5EAC959EF13A2327FBA3D52D74883FCE1119DACA1405B7AE1390ACEF02F1`.
  Sus detalles están en `../maintenance/local-msi-validation-2026-10-19.md`.
  No se instaló ni publicó como nueva release.
- Para Settings/Herramientas se generó otro MSI **local**, también con
  ProductVersion 2.1.3, preservado como
  `D:\OpsZone\DevWorkspace\Engines\Rust-Resource\cargo-targets\WSDD\wix\wsdd-2.1.3-settings-tools-pre-trace-2026-10-19-x86_64.msi`,
  SHA-256 `3D2369FC3055E6041843C97174E5D7E0C80183B1F252BB49EECD3E4FB7930532`.
  No incluye las etapas añadidas después para Deploy de herramientas.
- MSI **local de pruebas previo a v2.2.0** con esa trazabilidad:
  `D:\OpsZone\DevWorkspace\Engines\Rust-Resource\cargo-targets\WSDD\wix\wsdd-2.1.3-x86_64.msi`,
  SHA-256 `346A2BF679D581C1EF9DC13210C37854FCD6640E5A29DDA7B45FB5D6331586BC`.
  Se usó para pruebas locales; no es el asset publicado de v2.2.0.
- El [issue #15](https://github.com/wnunezc/wsdd-rust/issues/15) se cerró por
  el alcance acordado. Desgloses adicionales, resumen específico de hosts y
  otras mejoras propuestas allí no están planificados para la próxima versión.

## Publicación de v2.2.0

Las notas `2.2.0-draft.md` se limitaron a los cambios incluidos desde v2.1.3.
El MSI y checksum se generaron desde el tag firmado y se verificaron tras la
publicación; detalles en `2.2.0-validation.md`. La instalación y el despliegue
de Redis, Memcached y Mailpit siguen pendientes de validación real.
