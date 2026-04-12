# WebStack Deployer for Docker (WSDD)

Windows desktop application that automates the setup of a local web development environment
using Docker. Includes multi-version PHP, local SSL, MySQL, phpMyAdmin, and hosts management.

*Languages: [English](README.md) | [Español](docs/readme/README.es.md) | [Français](docs/readme/README.fr.md) | [हिन्दी](docs/readme/README.hi.md) | [中文](docs/readme/README.zh.md)*

*Quick links: [Migration Map](MIGRATION.md) | [License](LICENSE.md) | [Notice](NOTICE.md) | [Third-Party Licenses](THIRD_PARTY_LICENSES.md) | [Contributing](CONTRIBUTING.md) | [Security](SECURITY.md) | [Discussions](https://github.com/wnunezc/wsdd-rust/discussions) | [Report a Bug](https://github.com/wnunezc/wsdd-rust/issues/new)*

## About WSDD

WSDD is a Windows-first local stack manager for PHP + Docker development. It automates the
first-run environment setup, provisions per-project PHP containers, configures local SSL with
MKCert, updates the Windows `hosts` file, and centralizes container/project operations in a
single desktop app.

- **Current stage**: Release Candidate `1.0.0-rc.11`
- **Primary distribution package**: Windows MSI installer
- **Current UI languages**: English, Spanish, French, Hindi, Chinese
- **Issue reporting**: [GitHub Issues](https://github.com/wnunezc/wsdd-rust/issues/new)

## System Requirements

- **Operating System**: Windows 10 / Windows 11
- **Privileges**: Administrator (required)
- **Docker Desktop**: Automatically installed if not present
- **WSL 2**: Automatically configured
- **Chocolatey**: Automatically installed if not present

## What This Application Does

1. **Verifies and installs dependencies**: Docker Desktop, WSL 2, Chocolatey, MKCert
2. **Configures the Docker stack**: Nginx reverse proxy, MySQL, phpMyAdmin
3. **Manages web projects**: Creates PHP containers per version with Apache + Xdebug
4. **Automatic local SSL**: MKCert certificates per domain, no browser warnings
5. **Automatic hosts**: Modifies `C:\Windows\System32\drivers\etc\hosts` for you

## Docker Stack Containers

### Base Services (always active)
- **WSDD-Proxy-Server** — Nginx reverse proxy (ports 80 / 443)
- **WSDD-MySql-Server** — MySQL 8 (port 3306)
- **WSDD-phpMyAdmin-Server** — phpMyAdmin

### PHP Containers (one per version used)
Available versions: 5.6 - 7.2 - 7.4 - 8.1 - 8.2 - 8.3 - 8.4

For each activated version, the following development URLs are created:
- `php{version}.wsdd.dock` — Main PHP environment
- `cron{version}.wsdd.dock` — Cron jobs manager
- `wm{version}.wsdd.dock` — Webmin (server administration)

## Disk Environment Structure

The application creates and manages the `C:\WSDD-Environment\` directory:

```
C:\WSDD-Environment\
├── PS-Script\          — PowerShell automation scripts
├── Docker-Structure\   — docker-compose and PHP images
├── certs\              — SSL certificates per domain
└── wsdd-config.json    — Application configuration
```

## First Launch — Automatic Process

1. The application verifies it has administrator privileges
2. Extracts embedded resources to `C:\WSDD-Environment\`
3. Checks Chocolatey → installs if missing
4. Checks Docker Desktop → installs if missing (requires restart)
5. Checks MKCert → installs and configures local CA
6. Starts the base Docker stack
7. Shows the main panel

> **Note**: Docker Desktop installation may require a system restart.
> The application will automatically resume after restart.

## Usage After First Launch

### Adding a Project
1. Click "Add Project"
2. Choose the local domain (e.g., `myproject.wsdd.dock`)
3. Select the PHP version
4. The application creates the container, SSL certificate, and hosts entry

### Managing Containers
- Start / stop individual containers from the main panel
- Open real-time logs with one click
- Restart the complete stack from the menu

## Technical Information

- **Version**: 1.0.0-rc.11 (Rust edition)
- **GUI**: egui / eframe (immediate-mode)
- **Async**: tokio
- **Configuration**: JSON at `C:\WSDD-Environment\wsdd-config.json`
- **Logs**: Environment variable `RUST_LOG=wsdd=debug` for detailed logs

## License

Proprietary. See [LICENSE.md](LICENSE.md) for terms,
[NOTICE.md](NOTICE.md) for component attribution, and
[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) for referenced license families.
Copyright (c) 2026 Walter Nunez / Icaros Net S.A. All Rights Reserved.
