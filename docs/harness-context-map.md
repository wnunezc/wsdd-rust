# WSDD — mapa de lectura

## Entrada desde este repositorio

1. Leer `AGENTS.md` para reglas locales.
2. Leer `README.md` para producto, versión y requisitos; comprobar `git status` y `Cargo.toml` antes de actuar.
3. Leer solo el documento de dominio que exige la tarea, según la tabla.
4. Consultar el puente breve `Knowledge/Obsidian-Vault/08-AI-Context/wsdd-rust.md` si hace falta contexto transversal del workspace. La reentrada externa se lee únicamente si hay trabajo local sin cerrar.

El estado de producto y las pruebas pertenecen a este repositorio. Las notas de Obsidian son memoria auxiliar, no una segunda fuente de estado vigente.

| Tarea | Documento propietario |
|---|---|
| Uso y entorno | `README.md`, `docs/help/` |
| Arquitectura heredada C# → Rust | `MIGRATION.md`; la antigua carpeta C# bajo `Finished/` no existe actualmente |
| Credenciales de desarrollo y logs | `docs/development-credentials.md`, `docs/maintenance/development-credentials-and-log-suppression-2026-10-17.md` |
| Etapas y log completo de despliegue | `docs/maintenance/deploy-stage-trace-2026-10-19.md` |
| MSI local de validación, grupos Compose y trazabilidad | `docs/maintenance/local-msi-validation-2026-10-19.md` |
| Release o MSI | `docs/release/`, especialmente `docs/release/2.1.3-validation.md` para la publicación vigente |
| Remediación WP-01–WP-10 | `REMEDIATION_PLAN.md` |
| Recursos embebidos y PHP-Admin | `README.md` y el informe concreto de `docs/maintenance/` |
| Infraestructura Rust compartida local | `D:/OpsZone/DevWorkspace/Engines/Rust-Resource/README.md` si el engine existe; su ruta es una dependencia de este workspace, no del código fuente |

## Historia y continuidad

- El historial `wsdd-rust-history.md` y el snapshot `wsdd-rust-context-snapshot-2026-04-30.md` viven en `Knowledge/Obsidian-Vault/99-Archive/ai-context-heavy/`; se consultan solo por necesidad histórica.
- Las versiones extensas anteriores del AI Context y reentry se preservaron en ese mismo archivo con sufijo `pre-p4-2026-10-18`.
- `Knowledge/Obsidian-Vault/08-AI-Context/wsdd-rust-reentry.md` es corto; no cargarlo si el repo está limpio y no hay trabajo abierto.
- Los summaries fechados y los borradores antiguos de release son historia; no se cargan al inicio.
