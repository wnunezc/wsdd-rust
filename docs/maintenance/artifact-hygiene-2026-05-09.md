---
title: Artifact hygiene ledger
date: 2026-05-09
scope: WSDD
---

# WSDD — Artifact Hygiene Ledger (2026-05-09)

## Objetivo

Registrar el saneamiento de artefactos locales bajo la politica del workspace:

- producto: conservar
- evidencia canonica: conservar
- trabajo activo: conservar si sigue vigente
- cache/duplicado: eliminar

## Checkpoint pre-limpieza

### Estado trackeado

- stream activo preservado: templates Apache PHP con proxy WebSocket `/ws`
- `README.md` actualizado para enlazar `docs/harness-context-map.md`
- `docs/harness-context-map.md` incorporado como mapa documental caliente
- `recursos/ps-script.zip` y `recursos/docker-structure.zip` deben quedar sincronizados con sus arboles fuente antes de limpiar `target`

### Politica de retencion

Se conservan:

- `recursos/ps-script.zip`
- `recursos/docker-structure.zip`
- arboles fuente `recursos/recursos/PS-Script/` y `recursos/recursos/Docker-Structure/`
- `target/wix/wsdd-1.0.0-x86_64.msi`
- `target/wix/wsdd-1.0.0-x86_64.msi.sha256.txt`

Se eliminan despues del checkpoint:

- `recursos/recursos.zip`
- `target/validation/stable-1.0.0-final-test`
- `target/validation/stable-1.0.0-final-test-2`
- `target/debug`
- `target/x86_64-pc-windows-msvc`
- `target/release`
- cache auxiliar restante que no pertenezca al MSI/checksum retenidos

### Medicion pre-limpieza

- `target/` antes de la purga: `8159.17 MB` (`7.97 GB`)

### Validacion previa del guardrail

- `cargo check --workspace` -> OK con ZIPs sincronizados
- `cargo check --workspace --target-dir D:\OpsZone\DevWorkspace\tmp-rust-checks\wsdd-desync-should-fail` -> falla esperada al dejar `Docker-Structure` mas nuevo que `docker-structure.zip`
- `cargo check --workspace --target-dir D:\OpsZone\DevWorkspace\tmp-rust-checks\wsdd-final-sync-pass` -> OK tras regenerar el ZIP y normalizar el timestamp de prueba

### Nota operativa

- Git no versiona `target/`; los commits sirven como checkpoints del estado trackeado y de la politica aplicada, no como restauracion byte a byte de cache local.

## Checkpoint post-limpieza

### Resultado

- `target/` despues de la purga: `20.61 MB`
- artefactos retenidos fisicamente:
  - `target/wix/wsdd-1.0.0-x86_64.msi`
  - `target/wix/wsdd-1.0.0-x86_64.msi.sha256.txt`
- artefactos purgados:
  - `target/validation/stable-1.0.0-final-test`
  - `target/validation/stable-1.0.0-final-test-2`
  - `target/debug`
  - `target/x86_64-pc-windows-msvc`
  - `target/release`
  - `target/winget*`
  - `target/.rustc_info.json`

### Delta

- reduccion aproximada: `8138.56 MB`
