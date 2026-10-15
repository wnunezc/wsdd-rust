# WSDD Harness Context Map

## Proposito

Este archivo enruta la documentacion de `WSDD` para mantener el contexto caliente pequeno y dejar la historia larga fuera del arranque normal.

## Secuencia de arranque

1. `D:/OpsZone/DevWorkspace/Projects/Desktop/WSDD/AGENTS.md`
2. `D:/OpsZone/DevWorkspace/Knowledge/Obsidian-Vault/08-AI-Context/wsdd-rust.md`
3. `D:/OpsZone/DevWorkspace/Knowledge/Obsidian-Vault/08-AI-Context/wsdd-rust-reentry.md` si el repo esta sucio o la tarea es retomar trabajo activo
4. summary mas reciente y relevante de `WSDD`
5. este archivo
6. docs `warm` segun la tarea

## Capas documentales

### Hot

- `AGENTS.md` - contrato corto del proyecto
- `Knowledge/Obsidian-Vault/08-AI-Context/wsdd-rust.md` - estado actual, reglas de release y continuidad breve
- `Knowledge/Obsidian-Vault/08-AI-Context/wsdd-rust-reentry.md` - stop point operativo y worktree real
- summary reciente relevante de `WSDD`

### Warm

- `MIGRATION.md` - mapa C# -> Rust
- `REMEDIATION_PLAN.md` - cierre historico de `WP-01` a `WP-10`
- `README.md` - overview tecnico y estructura de entorno
- `docs/release/` - drafts/notas de release cuando la tarea toca RCs o versionado
- `docs/maintenance/php80-php85-and-php-admin-wsdd-desktop-plan-2026-10-11.md` - plan de cambios WSDD Desktop para PHP 8.0, PHP 8.5 y futura integracion WSDD-PHP-Admin
- `docs/maintenance/php-admin-release-build-installation-2026-10-15.md` - instalacion verificada de PHP-Admin 1.0.0 mediante RUN, CMD preservado e impedimento de sonda HTTP Release
- `docs/maintenance/php-admin-direct-probe-infrastructure-2026-10-15.md` - complemento implementado: Host y sonda Apache directa con seleccion Dev/Release, sin cambiar CMD ni entrypoint
- `docs/maintenance/remove-empty-volumes-and-readonly-install-audit-2026-10-15.md` - fix Remove/rollback, contrato volumes y auditoria de instalacion sin escrituras
- `docs/maintenance/base-container-recovery-compose-project-conflict-2026-10-15.md` - recuperacion base sin recrear servicios existentes con proyectos Compose mixtos
- `Knowledge/Obsidian-Vault/08-AI-Context/wsdd-rust-history.md` - historia resumida por fases
- `Knowledge/Obsidian-Vault/08-AI-Context/wsdd-csharp.md` - referencia narrativa C#
- `Finished/WSDD-CSharp/WebStack Deployer for Docker/` - referencia de codigo/flujo legacy cuando haga falta

### Cold

- `Knowledge/Obsidian-Vault/08-AI-Context/wsdd-rust-context-snapshot-2026-04-30.md` - ledger largo congelado antes de compactar
- user guides localizados `docs/help/*.md`
- READMEs localizados `docs/readme/*.md`
- drafts viejos de release que no tocan la tarea actual

## Si tocas X, lee Y

| Area | Leer primero |
|---|---|
| release / RC / MSI / GitHub Release | `08-AI-Context/wsdd-rust.md`, `docs/release/`, `REMEDIATION_PLAN.md` |
| recursos embebidos / Docker-Structure / PS-Script | `README.md`, `08-AI-Context/wsdd-rust-reentry.md` |
| migracion / comportamiento heredado | `MIGRATION.md`, `08-AI-Context/wsdd-csharp.md`, referencia C# archivada |
| historial de remediacion | `REMEDIATION_PLAN.md`, `08-AI-Context/wsdd-rust-history.md` |
| continuidad de trabajo activo | `08-AI-Context/wsdd-rust-reentry.md` |
| ayuda/localizacion | `docs/help/*.md`, `docs/readme/*.md` |

## Regla de carga

- No cargar el snapshot largo ni los user guides al inicio salvo que la tarea lo requiera.
- No usar `wsdd-rust.md` como ledger historico; el detalle largo ya no vive ahi.
- Si el repo esta sucio, `wsdd-rust-reentry.md` es obligatorio antes de interpretar el roadmap o abrir trabajo nuevo.
