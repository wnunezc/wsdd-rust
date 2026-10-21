# WebStack Deployer for Docker (WSDD)

Windows desktop application that automates the setup of a local web development environment
using Docker. Includes multi-version PHP, local SSL, MySQL, phpMyAdmin, hosts management,
Xdebug, and optional Redis/Memcached/Mailpit services.

*Languages: [English](README.md) | [Español](docs/readme/README.es.md) | [Français](docs/readme/README.fr.md) | [हिन्दी](docs/readme/README.hi.md) | [中文](docs/readme/README.zh.md)*

*Quick links: [User Guide](docs/help/user-guide.en.md) | [Harness Context Map](docs/harness-context-map.md) | [License](LICENSE.md) | [Notice](NOTICE.md) | [Third-Party Licenses](THIRD_PARTY_LICENSES.md) | [Contributing](CONTRIBUTING.md) | [Security](SECURITY.md) | [Discussions](https://github.com/wnunezc/wsdd-rust/discussions) | [Report a Bug](https://github.com/wnunezc/wsdd-rust/issues/new)*

## About WSDD

WSDD is a Windows-first local stack manager for PHP + Docker development. It automates the
first-run environment setup, provisions per-project PHP containers, configures local SSL with
MKCert, updates the Windows `hosts` file, and centralizes container/project operations in a
single desktop app.

- **Current stage**: `2.3.0` stable release line
- **Primary distribution package**: Windows MSI installer
- **Current UI languages**: English, Spanish, French, Hindi, Chinese
- **Language fallback**: English is the fallback for any missing localized UI/help content
- **Issue reporting**: [GitHub Issues](https://github.com/wnunezc/wsdd-rust/issues/new)

## Development Credentials And Responsibility

WSDD provides an **AS-IS local development stack**, not a production-hardened LAMP
platform. Generic development credentials may be used; legacy Webmin configuration
falls back to `admin` / `admin`. MySQL credentials are supplied during setup and
ngrok uses your own token. Do not treat these as production security defaults.

Use disposable/non-sensitive development data where possible. Change generic or
reused credentials before using confidential data, sharing access, or exposing a
service through ngrok or a network. Changing credentials alone does not guarantee
that data cannot be compromised. The developer is responsible for configuring
each container and the applications that use it.

See [Development credentials and stack relationships](docs/development-credentials.md)
for first setup, existing MySQL data, phpMyAdmin dependencies, per-PHP Webmin,
application credentials, and ngrok. WSDD does not add automatic password rotation,
a vault, or a new authentication mechanism.

The source implementation suppresses known WSDD credentials and recognized
credential fields in WSDD diagnostics, including copied logs. It does not encrypt
configuration, sanitize Docker's own retained logs/image metadata, or rewrite old
logs. These safeguards are included in version 2.2.0; installing new sources
does not change an already installed release. Review diagnostics before
sharing them, especially output from applications with secrets unknown to WSDD.

## System Requirements

- **Operating System**: Windows 10 / Windows 11
- **Privileges**: Administrator (required)
- **Graphics**: OpenGL 2.0-compatible graphics driver required by the desktop UI
- **Docker Desktop**: Must be installed by the user before first run
- **WSL 2**: Required by Docker Desktop
- **Chocolatey**: Automatically installed if not present
- **PowerShell**: 7.5+ (automatically installed/upgraded if missing)
- **Microsoft Visual C++ Runtime**: No separate VC++ redistributable is required for the shipped MSI because the Windows binaries are built with a static MSVC runtime

## What This Application Does

1. **Verifies and prepares dependencies**: Chocolatey, PowerShell 7.5+, Docker Desktop, MKCert
2. **Configures the Docker stack**: Nginx reverse proxy, MySQL, phpMyAdmin
3. **Manages web projects**: Creates PHP containers per version with Apache + Xdebug
4. **Automatic local SSL**: MKCert certificates per domain, no browser warnings
5. **Automatic hosts**: Modifies `C:\Windows\System32\drivers\etc\hosts` for you
6. **Optional developer services**: Configure ngrok, Redis, Memcached, and Mailpit in their separate Tools dialogs. Save stores options; Deploy applies the selected service through Docker. Redis, Memcached, and Mailpit still need end-to-end validation on a live WSDD installation.
7. **Linux-native project storage**: Optional managed WSL storage provides Linux filesystem performance while preserving a Windows-accessible project alias
8. **WSL runtime recovery**: A hidden scheduled task repairs Docker Desktop WSL mount relays after Windows or Docker restarts

## Docker Stack Containers

### Base Services (always active)
- **WSDD-Proxy-Server** — Nginx reverse proxy (ports 80 / 443)
- **WSDD-MySql-Server** — MySQL 26.7.0, primary and persistent replica (port 3306)
- **WSDD-phpMyAdmin-Server** — phpMyAdmin

The MySQL primary uses the container's Linux filesystem; the replica keeps the
existing Windows bind-mounted `bin/mysql/data` directory. Existing datadirs
initialized with `lower_case_table_names=2` require a separate, verified logical
migration before this Core can use them. The MSI does not migrate database data.
Full backup requires a clean MySQL stop; full restore is currently blocked for
this replicated layout. See the [MySQL Core report](docs/maintenance/mysql-core-replicated-datadir-cutover-2026-10-20.md).

### PHP Containers (one per version used)
Available versions: 5.6 - 7.2 - 7.4 - 8.1 - 8.2 - 8.3 - 8.4

For each activated version, the following development URLs are created:
- `php{version}.wsdd.dock` — Main PHP environment
- `cron{version}.wsdd.dock` — Cron jobs manager
- `wm{version}.wsdd.dock` — Webmin (server administration)
- `phpadmin{version}.wsdd.dock` — WSDD-PHP-Admin, proxied to internal port 8989

PHP-Admin is installed from its verified official package during image build.
A separate local-only Apache probe selects the same Dev/Release root as the
existing startup command. PHP 8.3 keeps the Dev root `phpadmin.dock`; other
versions use `phpadminXX.dock`. See the
[probe integration report](docs/maintenance/php-admin-direct-probe-infrastructure-2026-10-15.md)
for configuration and manual acceptance checks.

Webmin is installed inside PHP containers from the official Webmin APT repository during image build, so WSDD no longer pins a package version in application settings.

### Optional Services (disabled by default)
- **WSDD-Redis-Server** — Redis cache/queues/sessions (`redis:7.4.8-alpine`)
- **WSDD-Memcached-Server** — Memcached legacy cache (`memcached:1.6.39-alpine`)
- **WSDD-Mailpit-Server** — Local SMTP capture and web UI (`axllent/mailpit:v1.29.7`)

Optional services use Compose files under `Docker-Structure/services/` and the shared
`wsdd-tools` Compose project. WSDD starts only the selected service; each utility can be
started or stopped independently. The base stack uses `wsdd-core`, while PHP runtimes use
`wsdd-web`. All three groups share `wsdd-network`.

## Disk Environment Structure

The application creates and manages the `C:\WSDD-Environment\` directory:

```
C:\WSDD-Environment\
├── PS-Script\          — PowerShell automation scripts
├── Docker-Structure\   — docker-compose, PHP images, services, and SSL assets
├── wsdd-config.json    — Application settings
└── wsdd-secrets.json   — Managed secrets for containers
```

## First Launch — Automatic Process

1. The application verifies it has administrator privileges
2. Extracts embedded resources to `C:\WSDD-Environment\`
3. Checks Chocolatey → installs if missing
4. Checks PowerShell 7.5+ → installs/upgrades if missing
5. Checks Docker Desktop → blocks if not installed/configured
6. Checks MKCert → installs and configures local CA
7. Starts the base Docker stack
8. Shows the main panel

## Usage After First Launch

### Adding a Project
1. Click "Add Project"
2. Choose the local domain (e.g., `myproject.wsdd.dock`)
3. Select the PHP version
4. Select the storage mode:
   - `Classic Bind` for maximum compatibility with Windows folders
   - `Native Linux Project` for projects already stored in WSL
   - `Managed Native Linux` to convert a Windows folder into WSDD-managed WSL storage
   - `Fast Sync Workspace` for an initial Windows-to-Docker-volume snapshot
5. The application creates the container, SSL certificate, and hosts entry

Managed Native Linux Storage creates a validated ZIP snapshot before changing the source
folder, preserves the original Windows directory, stores project files under
`/home/wsdd/projects` in `WSDD-Storage`, and exposes the project through a Windows symbolic
link. Runtime permissions can be repaired from the project toolbox.

### Managing Containers
Use **Deploy PHP**, between **CMD Terminal** and **Add Project** in the toolbar,
to select and deploy a PHP version without adding a project. If Webmin credentials
are not stored for that version, enter them in the dialog. Existing containers
are not replaced by this action; use their toolbox instead.

Deployment builds the selected service before creating its container, preserves
existing project registrations and volumes, and reports progress in the main log.
Success requires a running container and an Apache HTTP response. A startup failure
leaves the container available for diagnostics; it does not delete its data.
This check does not certify PHP-Admin agent/reconciliation readiness.

The lower log panel shows measured deployment stages for **Deploy PHP** and
project deployment. During an image build it shows the latest observed BuildKit
step; **Show live output** expands the detailed stream. **Open full log** opens
that operation's redacted log in Notepad, and **Copy full log** copies the file
instead of the compacted on-screen buffer. Operation logs are stored under
`C:\WSDD-Environment\logs\wsdd-operation-*.log`.

- Start / stop individual containers from the main panel
- Open real-time logs with one click
- Restart the complete stack from the menu

## Technical Information

- **Version**: 2.3.0 (Rust edition)
- **GUI**: egui / eframe (immediate-mode)
- **Async**: tokio
- **Configuration**: `C:\WSDD-Environment\wsdd-config.json`
- **Secrets**: `C:\WSDD-Environment\wsdd-secrets.json`
- **Logs**: Environment variable `RUST_LOG=wsdd=debug` for detailed logs

## License

Proprietary. See [LICENSE.md](LICENSE.md) for terms,
[NOTICE.md](NOTICE.md) for component attribution, and
[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) for referenced license families.
Copyright (c) 2026 Walter Nunez / Icaros Net S.A. All Rights Reserved.
