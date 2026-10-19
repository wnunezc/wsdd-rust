# Docker Desktop Compose groups (2026-10-19)

## Ownership

| Compose project | Services | Source |
| --- | --- | --- |
| `wsdd-core` | Proxy, MySQL, phpMyAdmin | `Docker-Structure/init.yml` |
| `wsdd-web` | PHP runtimes | `Docker-Structure/bin/php*/webserver.php*.yml` plus live `options.php*.yml` |
| `wsdd-tools` | Redis, Memcached, Mailpit, ngrok | `Docker-Structure/services/*.yml` |

Container names, named volumes, bind mounts, host ports, and the external `wsdd-network` remain unchanged. The top-level Compose `name` values match the explicit `-p` values used by WSDD. Optional deployment supplies all four service files to Compose and targets one service with `up -d --no-deps`; this keeps the project definition complete without activating other utilities. Stop still targets one container.

## Code paths reviewed

| Operation | Code path | Behavior |
| --- | --- | --- |
| Startup | `requirements.rs` → `docker_deploy.rs` → `base/containers.rs` | Sync managed templates; start existing base containers or create only absent services under `wsdd-core` |
| PHP runtime | `deploy/php_runtime.rs`, `deploy/docker_ops.rs` | Build/create one PHP service under `wsdd-web`; project `options` state remains live |
| Project add/remove | `deploy.rs`, `deploy/removal.rs` | Check runtime label before storage or project mutations and before cascaded runtime removal |
| Optional utility | `docker_deploy/optional_services.rs` | Check fixed container name, then start or stop just the selected service under `wsdd-tools` |
| Backup restore | `backup/runtime.rs` | Check known WSDD owners before bulk removal or runtime recreation |
| WSL runtime recovery | `PS-Script/wsdd-runtime-recovery.ps1` | Recreate the PHP runtime with `wsdd-web` |

Rollout sequence: package matching templates and ZIPs; install MSI; launch WSDD so it synchronizes managed files; allow WSDD to create the base group; deploy PHP runtimes and optional utilities through the app as needed. An existing fixed-name container with a legacy Compose label causes an explicit error before the affected operation. Its data and metadata require a user-directed migration if it must be retained.

## Installed environment

The installer carries the executable with the embedded resource ZIPs. On a new environment, WSDD extracts both ZIPs into `C:\WSDD-Environment`. On an existing environment, startup reconciles the managed base, PHP, and optional service Compose files from embedded templates. The existing project `options.php*.yml` files are migrated in place and retain project entries; the full Docker structure is extracted only when absent or structurally damaged. The PowerShell resource is repaired from its ZIP when its contents differ, so WSL recovery also uses `wsdd-web`.

Docker Compose project labels on existing containers cannot be renamed in place. Before base recovery, project deployment/removal, PHP rebuild, or optional service start/stop, WSDD rejects a fixed-name container whose Compose project label differs from the expected group. It does not remove or adopt it. This avoids replacing a legacy container or changing its mounts as a side effect. The user reported an empty Docker container inventory before installing this build; WSDD itself did not remove containers.

## Verification boundary

Source and embedded ZIP entries must agree before build. Rust format, check, clippy, and tests validate the build and ownership guard. MSI compilation proves packaging. A manual run after installation is still needed to verify Docker Desktop grouping and service behavior on the user's machine; no Docker container lifecycle operation is part of the build verification.

The combined local MSI for these Compose groups and the deployment trace was
built on 2026-10-19. Build evidence and its checksum are recorded in
`local-msi-validation-2026-10-19.md`. This local artifact is distinct from the
published v2.1.3 release.
