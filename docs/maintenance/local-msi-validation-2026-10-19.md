# MSI local de validación — 2026-10-19

## Alcance incluido

- Grupos Compose `wsdd-core`, `wsdd-web` y `wsdd-tools`, con comprobación de
  propiedad de contenedores existentes. Detalle: `compose-groups-2026-10-19.md`.
- Etapas de Deploy PHP/proyecto, avance observado de BuildKit, log completo por
  operación y controles para abrirlo/copiarlo. Detalle:
  `deploy-stage-trace-2026-10-19.md`.
- Filtro diagnóstico Rust/PowerShell corregido para conservar la URL
  `phpadmin56.wsdd.dock` y seguir ocultando el usuario `admin` independiente.

## Evidencia local

`cargo wix-msi --nocapture` terminó con exit code 0. El instalador creado es:

| Propiedad | Valor |
| --- | --- |
| Ruta preservada | `D:\OpsZone\DevWorkspace\Engines\Rust-Resource\cargo-targets\WSDD\wix\wsdd-2.1.3-pre-settings-2026-10-19-x86_64.msi` |
| Tamaño | 26 951 680 bytes |
| SHA-256 | `35FD5EAC959EF13A2327FBA3D52D74883FCE1119DACA1405B7AE1390ACEF02F1` |
| ProductVersion en MSI | `2.1.3` |
| Checksum adjunto | `wsdd-2.1.3-pre-settings-2026-10-19-x86_64.msi.sha256.txt`, cotejado con la copia preservada |

La comparación SHA-256 de cada entrada con su archivo fuente verificó
`docker-structure.zip` (92/92 archivos) y `ps-script.zip` (13/13 archivos),
sin ausencias, extras ni diferencias. Antes de empaquetar pasaron
`cargo fmt --all -- --check`, `cargo check --workspace`,
`cargo clippy --workspace -- -D warnings` y `cargo test --workspace --quiet`
con `WSDD_SKIP_WINDOWS_MANIFEST=1`: 201 pruebas principales, 11 de validación
de release y 6 del launcher.

## Estado y validación manual

Esta compilación anterior fue preservada con otro nombre antes de generar el
MSI local de Settings/Herramientas. Su checksum y tamaño originales se
verificaron antes y después de la copia. Este MSI **no es** el asset publicado de v2.1.3 en GitHub, aunque conserva la
misma versión de producto. Se distingue por el SHA-256 anterior. No se instaló,
no se modificó `C:\WSDD-Environment`, no se ejecutó Docker y no se publicó una
release durante esta validación.

Tras instalarlo, corresponde al usuario iniciar WSDD y verificar que sincroniza
los recursos administrados en `C:\WSDD-Environment`. Desde WSDD, comprobar las
secciones Compose, desplegar un runtime PHP y un proyecto, observar las etapas
y el avance de BuildKit, abrir/copiar el log completo y confirmar que la URL
`phpadmin56.wsdd.dock` aparece intacta. Los grupos reales de Docker, el
comportamiento de la UI durante una build y la instalación sobre el entorno
existente aún requieren esa prueba manual.
