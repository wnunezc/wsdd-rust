# Cambios acumulados desde la v2.1.3 publicada

Este documento es el registro de entrada para las notas de la próxima versión
de WSDD. La versión siguiente aún no está definida ni publicada. El punto de
partida es el tag público `v2.1.3` (commit fuente
`c75d1493a1c55560ecdb7645cc825fd7e9e14d81`, publicado el 2026-10-15).
La evidencia y el checksum del MSI público pertenecen a
`2.1.3-validation.md`; **un MSI local posterior con ProductVersion 2.1.3 no
reemplaza ese asset**.

## Inventario posterior al tag (2026-10-19)

| Commit | Estado | Cambio que debe considerarse |
| --- | --- | --- |
| `59bc48076b482370a7cf8f3a5588e1cad74a7662` | En `main` remoto | Sólo documenta y verifica la publicación de v2.1.3; no agrega comportamiento al programa. |
| `0a4fad624a68b5d4b913b79e0690c2016cd6dc30` | En `main` remoto | Guía de credenciales AS-IS y supresión de credenciales conocidas en diagnósticos de Rust, PowerShell, panel, archivos y copia. |
| `7503c5bd1d7fc977a361c95c694ad917236a8b68` | Commit local, sin push | Grupos Compose, guardas de propiedad, etapas de despliegue, seguimiento BuildKit, log por operación y corrección del falso positivo `phpadmin56.wsdd.dock`. |

`origin/main` apunta a `0a4fad6`; el commit local `7503c5b` aún no forma parte
del repositorio remoto. Al preparar una publicación, volver a comparar el tag
`v2.1.3` con el commit elegido para la release e incorporar aquí cualquier
commit posterior. Este inventario no implica que los cambios locales estén
disponibles en el MSI público actual.

## Contenido funcional para las próximas notas de versión

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

## Evidencia disponible y límite de distribución

- El commit `0a4fad6` pasó CI remoto con 211 pruebas. Los cambios locales
  posteriores pasaron formato, `cargo check`, Clippy con advertencias denegadas
  y 218 pruebas del workspace. Los ZIP embebidos coinciden por SHA-256 con sus
  fuentes: 92 archivos Docker y 13 PowerShell.
- Se generó un MSI **local** que combina estos cambios:
  `D:\OpsZone\DevWorkspace\Engines\Rust-Resource\cargo-targets\WSDD\wix\wsdd-2.1.3-x86_64.msi`,
  SHA-256 `35FD5EAC959EF13A2327FBA3D52D74883FCE1119DACA1405B7AE1390ACEF02F1`.
  Sus detalles están en `../maintenance/local-msi-validation-2026-10-19.md`.
  No se instaló ni publicó como nueva release.
- El [issue #15](https://github.com/wnunezc/wsdd-rust/issues/15) se cerró por
  el alcance acordado. Desgloses adicionales, resumen específico de hosts y
  otras mejoras propuestas allí no están planificados para la próxima versión.

## Al preparar la próxima publicación

Comparar de nuevo `v2.1.3..HEAD` del commit de release, actualizar este registro
y trasladar **todos los cambios funcionales anteriores más los posteriores** a
las notas públicas de esa versión. Distinguir código en remoto, cambios sólo
locales y funciones realmente validadas; asignar la versión nueva sólo cuando
se decida publicar. Regenerar MSI/checksum desde ese commit y validar instalación
y despliegue desde WSDD antes de afirmar comportamiento en Docker. Una release
o push requiere su instrucción específica; este documento no los ejecuta.
