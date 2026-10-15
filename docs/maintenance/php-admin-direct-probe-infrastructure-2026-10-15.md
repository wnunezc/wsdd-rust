# PHP-Admin: Direct Apache Probe In WSDD Resources

## Observed PHP 8.4 Release Acceptance

After the user rebuilt PHP 8.4, read-only checks on 2026-10-15 confirmed:

- Container running with restartCount=0, installed Release VERSION 1.0.0 and
  no executable Dev CLI. Managed Host and mounted probe configuration present.
- Agent runs as root from /opt/wsdd-php-admin; runtime:web and the PHP server
  run as www-data from the same root. Port 8989 listens on 127.0.0.1.
- Current startup log reports reconciliation success and readiness verified;
  current agent-status reports ready. Old failures remain in persistent logs
  and must not be mistaken for the result of this new container startup.
- Authenticated direct Apache probe: HTTP 200, app_root=/opt/wsdd-php-admin,
  sapi=apache2handler, nonce matches. Authentication keys were not printed.
- Authenticated runtime-health request through the public Apache vhost
  phpadmin84.wsdd.dock: HTTP 200, root=/opt/wsdd-php-admin, user=www-data,
  startup identity matches the agent. This check was issued inside the container,
  not through the outer Windows/nginx-proxy/browser path.
- Probe without authentication: HTTP 403. Other probe-vhost path (/): HTTP 403.
- Existing default vhost remains php84.wsdd.dock.

These checks did not start/restart services, run reconciliation, submit an
operation, install/remove extensions or edit persistent configuration. HTTP
requests naturally generate service access logs. Only PHP 8.4 Release startup
and the above HTTP paths are now observed working. Dev, other PHP versions,
extension-management operations, delayed-start behavior and external browser/TLS
acceptance remain separate tests. The original implementation-time limitations
below describe the earlier pre-rebuild verification.

## Implemented Scope

Completes the infrastructure missing from the build-time installation of the
official PHP-Admin 1.0.0 package. Applies to PHP 5.6, 7.2, 7.4, 8.1, 8.2, 8.3
and 8.4. Does not change PHP-Admin source, package URL/hash, Dockerfiles, CMD,
official entrypoint, persistent data or the public administration proxy.

| Before | After |
|---|---|
| Missing probe Host; Release basename is invalid | Managed options provide WSDD_PHP_ADMIN_APACHE_PROBE_HOST=phpadmin-runtime.internal |
| Public panel only proxies to 8989 | Separate direct mod_php probe vhost on existing Apache port 80 |
| No direct route to installed public/runtime-probe.php | Probe maps to the same Dev/Release root selected by CMD |
| A Release-only Host/root would conflict with Dev | One Host for both modes; executable CLI conditions select the corresponding probe file |
| Build-time sites-enabled files could be hidden by bind mount | Probe file lives in WSDD's mounted vhost source and is synchronized by Rust |

## Root Selection And HTTP Isolation

The new vhost uses *:80, not 127.0.0.1:80, to keep existing wildcard name-based
vhost selection. Its filename wsdd-php-admin-probe.conf follows vhost.conf, so
the existing PHP vhost remains the default. The internal Host is not added to
VIRTUAL_HOST, Windows hosts, mkcert or an externally published Docker port.

Within this dedicated vhost, mod_rewrite selects only /runtime-probe.php:

1. If the Dev CLI has executable permissions, map directly to that Dev root's
   public/runtime-probe.php and stop rewriting. Missing/incomplete Dev must fail
   there, not fall back to Release after CMD has selected Dev.
2. Otherwise, if /opt/wsdd-php-admin/bin/wsdd-php-admin is executable, map directly
   to /opt/wsdd-php-admin/public/runtime-probe.php.
3. Deny other paths and cases without an executable CLI. Keep the nonce query.

Both target directories use Require local, AllowOverride None and disabled
indexes. There is no ProxyPass or proxy rewrite flag in this vhost. PHP-Admin
retains its own loopback/key/nonce/app_root/apache2handler checks unchanged.
Dev roots are phpadminXX.dock, except PHP 8.3 which keeps phpadmin.dock.

The single environment Host is not tied to the Release root: Apache resolves
the target using the same executable-file priority as CMD. This avoids a
startup wrapper, another agent, symlink management or PHP-Admin package changes.
Changing mounted code or executable permissions after startup can still make
the app's identity check fail; runtime roots must not be switched underneath
running agent/web processes.

## Changed Files And Synchronization

- src/config/environment.rs: shared internal Host constant.
- src/models/project/php_version.rs: version-specific Dev root accessor.
- src/handlers/docker_deploy/templates.rs: versioned probe renderer.
- src/handlers/docker_deploy.rs: writes the separate managed probe file during
  existing resource synchronization, including installations with public proxy
  already present. Does not overwrite vhost.conf or personal project blocks.
  If the new filename already contains a configuration without WSDD's ownership
  marker, synchronization aborts rather than overwriting that custom file.
- src/handlers/yml.rs: inserts/updates the managed probe environment key without
  replacing other variables, project mounts or named-volume declarations.
- Each bin/phpX.Y/options.phpXX.yml: initial environment configuration.
- Each bin/phpX.Y/vhost/wsdd-php-admin-probe.conf: initial probe configuration.
- recursos/docker-structure.zip: seven options entries updated and seven probe
  entries added; 92 source files and ZIP entries match by SHA256.

## Verification And Limits

- All seven Dockerfiles are unchanged relative to safety commit f759cd2;
  CMD and entrypoint semantics are unchanged.
- cargo check, fmt and clippy -D warnings passed.
- 36 focused tests passed: 17 YAML, 9 Docker template, 8 PHP version and 2
  managed-file synchronization tests.
  Includes renderer/source equality across versions, CMD root-condition alignment,
  local-only/no-proxy checks and idempotent options migration preserving projects.
- Apache in the existing PHP 8.4 container accepted the probe via -c directives
  with -t (Syntax OK) and -S. The default remains php84.wsdd.dock and the public
  phpadmin84.wsdd.dock proxy stays separate. These are parse/dump operations only:
  no config was installed in the container, and Apache was not restarted/reloaded.
- An initial attempt to include /dev/stdin for parsing was rejected as not a
  regular config file; CLI directives were then used successfully instead.
- No Docker image build, runtime reconciliation, end-to-end HTTP probe test,
  container recreation or installed-environment synchronization was executed.
  The installation remains unchanged until the user applies the new MSI/deploy.

Startup sequencing remains as contracted: runtime:start in background followed
by Apache foreground. The package's Apache verifier retries for 30 seconds;
agent/web child readiness is a separate later phase. No artificial sleep or
second Apache starter was introduced. An unusually slow Apache startup, or a
graceful reload before Apache exists, remains a manual startup test condition;
this change does not claim to have eliminated that previously identified risk.

## Manual Acceptance

1. Install the new MSI, run WSDD prerequisites, and check that the installed
   options have the Host and mounted vhost directory has the new managed file.
2. Use project Deploy to recreate the chosen PHP runtime with the new environment;
   starting an old container alone does not apply changed Compose environment.
   Existing PHP runtimes are not automatically recreated by base startup recovery.
3. Release: without an executable Dev CLI, verify /opt root, authenticated direct
   mod_php probe, successful reconciliation, agent/web and loopback port 8989.
4. Dev: mount the executable CLI and verify that agent, web and probe all report
   the Dev root while Release files remain installed. Keep the PHP 8.3 exception.
5. Verify that wrong/missing key, non-loopback access and other probe-vhost paths
   are denied. Check normal projects and public admin/cron/Webmin routing still work.
6. Repeat after recreation with existing profiles, then check the other PHP
   versions and delayed-start behavior. Do not delete persistent state for testing.

The latest release draft records the MSI result/hash. Manual runtime validation
remains pending; syntax and source checks are not an end-to-end certification.

## MSI Result

User-authorized cargo wix-msi --nocapture completed with exit 0 and no reported
warnings/errors. The installer includes these resources and their synchronizer.

- Path: D:/OpsZone/DevWorkspace/Engines/Rust-Resource/cargo-targets/WSDD/wix/wsdd-2.1.3-x86_64.msi
- Bytes: 26,730,496.
- SHA256: BA67DA881513745B7FC7BFE7ADE326D75AEC584D0FC385336D793CDE9834438D.

The sibling checksum file was updated. No installation or publication was
performed. The preceding MSI E96FEE6C... does not contain this probe addition.
