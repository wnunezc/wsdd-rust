# Remove: Empty Volumes And Read-Only Installation Audit

## Subsequent Authorized Repair

After the read-only audit, the user explicitly authorized correcting the installed
findings and generating a new MSI. The audit sections below describe the earlier
observation; the installed-state results in this section supersede them.

- Backups of all seven edited files and a SHA256 manifest were saved outside
  installation at C:/Users/wnune/AppData/Local/Temp/wsdd-install-repair-20261015T194115Z.
- Removed only the empty root volumes key from installed options for PHP 5.6,
  7.2, 7.4, 8.1, 8.2 and 8.3. Service mounts/persistence were preserved.
- Changed only the installed MySQL Dockerfile's bulk_insert_buffer_size and
  innodb_buffer_pool_size from 4G to 1G. This does not alter the active MySQL
  runtime: applying image settings requires a separately authorized rebuild.
- All seven installed PHP Compose configurations now pass config --quiet.
- Comparing 206 static file hashes before/after found exactly the intended
  seven edited files; the remaining 199 were unchanged. Content comparison
  confirmed no semantic edits beyond the empty keys and the two MySQL values.
- Source Docker resources and embedded ZIP remain synchronized (85 files).
- No containers were created, removed, rebuilt or restarted, no project records
  were edited, and no persistent data was modified by the repair commands.
- A new local MSI was requested; see current release draft and reentry for its
  final build result and checksum.

## Scope

Authorized: fix Remove in WSDD source and evaluate C:/WSDD-Environment without
changing its contents. No installation, repair, synchronization, recreation,
build of Docker images or service restart was executed against that environment.
Evidence/logs were saved only under TEMP; this report lives in the repository.
No new MSI was requested or generated for this fix.

## Confirmed Defects And Source Fix

1. Removing the last named project volume left a root-level `volumes:` with a
   null value. Compose rejects it with `volumes must be a mapping`. Remove now
   parses the resulting YAML with serde_yaml and removes only that empty root
   key before writing. Other named volumes, service mounts, persistence and
   unrelated root sections are retained. Invalid root volume types abort the
   write instead of claiming success.
2. Shared PHP recreation previously called stop/rm before Compose, then marked
   the rollback flag only after successful up. Compose failure could leave the
   runtime removed without a restoration attempt. Recreation now validates the
   complete Compose configuration first, does not manually stop/remove the
   container, and tracks the mutation attempt before Compose up can fail.
   Both project Deploy and Remove pass that flag to their existing rollback.
   Rollback remains best-effort and reports restoration errors; this is not an
   atomic transaction and does not guarantee recovery from external failures.

The root `volumes` mapping declares named volumes. The separate service-level
`volumes` list mounts project data and runtime resources; it is not removed.
`add_project_to_options_yml` already handles an absent root key: its else branch
appends the key and the project's external named-volume declaration. Therefore
the literal root header is not required when there are no named volumes.

Regression tests cover adding two projects, removing the first, removing the
last, repeated removal, and adding a third across all seven PHP versions.
They verify valid mappings, preservation of runtime persistence and networks,
preexisting empty-root cleanup, rejection without writes, preflight failure
without up, and tracking a failed recreation for rollback.

## Installed Environment Findings

| Area | Read-only finding |
|---|---|
| Inventory | 7,330 files observed; one MySQL socket reparse point skipped |
| PHP 5.6, 7.2, 7.4, 8.1, 8.2, 8.3 | options files end in an empty root `volumes:`; Compose validation fails |
| PHP 8.4 | Compose validation passes |
| Base stack and optional services | init, Redis, Memcached, Mailpit and ngrok Compose validation passes |
| Projects | 15 JSON records, all PHP 8.4; no duplicate domains; expected mounts, vhosts, SSL files and Docker volumes present |
| PHP 8.4 named volumes | 15 project declarations, no orphan project declarations relative to those records |
| JSON | All 800 observed JSON files parse with System.Text.Json; an initial PowerShell ConvertFrom-Json rejection of a vendor file was not reproduced by the JSON parser and is not evidence of corrupt JSON |
| PowerShell | All 12 installed scripts parse without syntax errors |
| Certificates | 45 CRT files readable and unexpired at observation; private-key matching and TLS handshakes not tested |
| Resource baseline | 97 corresponding source resources checked: 70 byte-identical, 27 different, none missing |
| PHP Dockerfiles | All seven match current source by SHA256 |
| PHP compose files | All seven match source after normalizing rendered Webmin/Xdebug build arguments and line endings |
| Vhost managed prefixes | PHP 8.3 matches; other six differ in the newline separating the closing VirtualHost tag and the managed marker; installed copies have the separate lines |
| MySQL Dockerfile | Installed file still contains 4G for both innodb_buffer_pool_size and bulk_insert_buffer_size; current source contains 1G. Confirmed configuration drift, not proof of data corruption |

Other differing files include rendered init/optional-service configurations and
live project options/vhosts; differences from templates alone do not establish
corruption. Credentials were neither printed nor included in this report.

For each of the six invalid options files, its text was read into memory and
only the empty root key removed from that in-memory copy. Passing the result
via stdin to `docker compose ... config --quiet` with the original project
directory succeeded for all six. No corrected copy was written to installation.

At observation Docker listed PHP 8.4, proxy, MySQL, phpMyAdmin and ngrok running;
there were no PHP 5.6/7.2/7.4/8.1/8.2/8.3 containers in that listing. Invalid
Compose files block future create/recreate operations; they do not by themselves
stop an already running container.

## Protection And Verification

- All 97 corresponding installed resource hashes remained unchanged between
  the initial comparison and the final comparison.
- A second set of 206 static-file hashes, including configuration, project
  records, certificates and scripts, remained unchanged through verification.
- Comparing inventory size/mtime later found changes in 12 files: Apache's PHP
  8.4 error log and MySQL temporary tablespace files. These are live-runtime
  paths; no write operation was issued to them by this task. An active directory
  cannot be claimed globally frozen based on this audit.
- fmt, cargo check and clippy -D warnings passed. Targeted tests: 16 YAML tests
  and 5 Docker lifecycle tests passed. Full workspace tests were deliberately
  not run here: other existing tests instantiate log bridges that use the
  installed logs directory. Tests exercised in this task use TEMP/mocks only.
- The latest existing MSI predates these fixes. Source resources/Dockerfiles
  and their embedded ZIP were not changed by this fix.

## Remaining Limits

The six invalid installed files are intentionally still untouched. A source fix
does not repair that local state automatically during this audit. A separate
explicitly authorized repair or deployment is required before treating those
installed configurations as usable.

This was a configuration, syntax and consistency review, not a forensic
integrity certificate. No MySQL physical data-page checks, filesystem repair,
PHP-Admin state migration/reconciliation, Apache configuration execution inside
containers, or external project-source audit was performed. Those data cannot
be declared free of corruption based on these checks.
