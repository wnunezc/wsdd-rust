# Base Container Recovery: Compose Project Conflict

## Confirmed Diagnosis

User startup output showed base recovery recreating MySQL and phpMyAdmin, then
failing to create WSDD-Proxy-Server because the name already existed.

Read-only Docker inspection confirmed:

- Proxy: project label docker-structure, service reverse-proxy, exited.
- MySQL/phpMyAdmin: project label wsdd-projects, services database/phpmyadmin,
  state created after the failed startup.
- PHP 8.4: project wsdd-projects, service webserver84, exited.
- All base config_files labels point to the installed init.yml.

Existing base recovery used Compose up against wsdd-projects whenever any base
service was not running. That command could recreate present services to match
current Compose config and could not adopt the proxy owned by docker-structure.
The orphan warning concerns PHP 8.4 sharing wsdd-projects but absent from init.yml;
it is not a reason to use --remove-orphans or delete the PHP runtime.

MySQL inspect still has the bind from the installed bin/mysql/data to
/var/lib/mysql. This confirms the mount, not database integrity. No service was
started, stopped, recreated, removed or migrated during this investigation.

## Source Correction

- Inventory uses exact canonical base names and machine-readable Docker states;
  unsuccessful Docker commands abort instead of implying absent containers.
- Existing created/exited containers start by name in dependency order:
  MySQL, phpMyAdmin, proxy. Compose ownership labels are not changed or migrated.
- Only missing services use targeted Compose up with --build --no-recreate
  --no-deps. Other services, PHP runtimes, volumes and orphan containers are not
  removed or reconciled by this startup recovery.
- If creating services is necessary, Compose config is validated before actions.
- Running containers are left alone. Restarting is not considered ready and
  is allowed to reach running within the existing bounded wait. Paused/dead/
  removing/unknown states abort without automatic replacement.
- An action failure stops further recovery. Readiness remains a running-state
  check; it does not prove application health or data integrity.

## Verification And Packaging

fmt and clippy -D warnings passed. Six focused tests passed using mocks/pure
planning, including all 27 combinations of absent/running/exited base services,
exact names, daemon failure, stop-on-error and non-destructive Compose flags.
No production log bridge or real recovery operation was executed in these tests.

After investigation the user explicitly authorized MSI compilation. The new
local MSI includes this correction and the earlier Remove/rollback fix:

- cargo wix-msi --nocapture: exit 0, no warnings/errors reported.
- Size: 26,734,592 bytes.
- SHA256: E96FEE6C1DF348170D7C7468628034F99BA39904BA6A703BD4BDE76870559DF5.
- Artifact: D:/OpsZone/DevWorkspace/Engines/Rust-Resource/cargo-targets/WSDD/wix/wsdd-2.1.3-x86_64.msi.

This replaces the previous local MSI 67AEBE1D... . No installation or runtime
recovery was performed; manual testing remains pending.
