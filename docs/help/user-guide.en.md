# WSDD Help

<!-- Canonical source for the in-app help rendered by src/ui/helps.rs. -->

## System requirements

Operating system:
  - Windows 10 (build 2004+) or Windows 11
  - x64 architecture (AMD64)
  - 8 GB RAM minimum — 16 GB recommended for PHP stacks
  - 20 GB free disk space

Privileges:
  - WSDD must run as Administrator (UAC required)
  - Needed to modify C:\Windows\System32\drivers\etc\hosts

Software installed automatically by WSDD:
  - Chocolatey — Windows package manager
  - PowerShell 7.5+ (pwsh.exe) — compatible automation runtime
  - mkcert — local SSL certificate generation

Software that must be pre-installed:
  - Docker Desktop — main container engine
  - WSL 2 — Linux subsystem required by Docker Desktop

## Installation and first launch

1. Run wsdd.exe as Administrator (right-click → Run as administrator).

2. On first launch the Welcome Wizard appears:
   - Read and check the requirements checkbox.
   - Click 'Next' to start environment verification.

3. Before the base environment is deployed for the first time, WSDD opens a
   dialog to capture MySQL and phpMyAdmin credentials.
   - These credentials are stored in wsdd-secrets.json.
   - They are requested only when missing from configuration.

4. The Loader verifies and automatically installs:
   - Chocolatey (package manager)
   - PowerShell 7.5+
   - mkcert (local SSL)
   - Network configuration and root certificate

5. If Docker Desktop was not installed:
   - The Loader stops and asks you to install it manually.
   - After installing Docker Desktop, open WSDD again.

6. When the Loader completes, the Main Panel appears.

WSDD data location:
  C:\WSDD-Environment\
  ├── PS-Script\         PowerShell automation scripts
  ├── Docker-Structure\  Container, project, and SSL configuration
  └── wsdd-config.json  Application settings

## Main panel — overview

The main panel has three areas:

1. MENU BAR (top):
   - File → Add Project, Exit
   - Docker → Refresh list, Reload Docker Desktop, Clear logs
   - Tools → WSL Settings, Settings
   - Help → Help, About...

2. TOOLBAR (below menu):
   - ⬡ phpMyAdmin   — Open phpMyAdmin in browser (http://pma.wsdd.dock)
   - ⚡ PS Terminal  — Open PowerShell 7 in C:\WSDD-Environment
   - ⬛ CMD Terminal — Open CMD in C:\WSDD-Environment
   - + Add          — New project form
   - ↺ Refresh      — Update containers and projects
   - Theme selector (right) — Switch between Dark Neutral, Dark Blue, Dark Warm, Light

3. CENTER TABS:
   - Containers   — List of active/inactive Docker containers
   - Projects     — List of registered WSDD projects

4. LOG PANEL (bottom):
   - Shows operation history with color-coded levels.
   - Has more usable height and can be resized for longer output.
   - 'Copy' button — copies log to clipboard.
   - 'Clear' button — clears visible log.

5. STATUS BAR (bottom):
   - Shows detected containers, running containers, projects and logs.
   - Shows Docker state and visible CPU/RAM usage.

## Docker container management

The 'Containers' tab shows all WSDD containers with their current state.

COLUMNS:
  - Name       — Docker container name
  - Status     — Running / Exited / ...
  - Image      — Base Docker image
  - Toolbox (⚙)  — Advanced container actions

ACTIONS PER CONTAINER (action columns):
  - ▶ Start   — Start the container
  - ■ Stop    — Stop the container
  - ↺ Restart — Restart the container

CONTAINER TOOLBOX (⚙ button):
  - View real-time container logs
  - Open interactive TTY terminal inside container
  - View exposed URLs and ports
  - Detailed info (image, ports, volumes)

AUTO POLLING:
  Containers update automatically every 3 seconds.
  Manual refresh not required (though ↺ button exists to force it).

PRESENTATION:
  - The main view leaves more breathing room around the grid.
  - The goal is to improve readability and avoid text touching the edge.

NOTE: Only WSDD stack containers are shown (prefix 'wsdd-' or related).
To see all Docker containers, use Docker Desktop or CLI.

## Project management

The 'Projects' tab shows web projects registered in WSDD.

COLUMNS:
  - Name       — Project name
  - Domain     — Local domain (e.g., myapp.wsdd.dock)
  - PHP        — Assigned PHP version (5.6 — 8.4)
  - Status     — Deployed / Not Deployed
  - Actions    — Deploy, Remove, Toolbox

ACTIONS:
  - ⬆ Deploy    — Deploy the project (creates containers, SSL, hosts)
  - ⬇ Remove    — Remove deployment (does NOT delete source files)
  - ⚙ Toolbox   — Advanced project actions

PROJECT TOOLBOX:
  - Open project folder in Windows Explorer
  - Open project in browser
  - View detailed info (paths, domain, entrypoint)

DELETE PROJECT:
  - Clicking Remove prompts for confirmation.
  - Removed: containers, SSL, hosts entry, JSON record.
  - Source code files are NOT deleted.

## Adding a project

To add a new web project:

1. Click '+ Add' in toolbar or go to File → Add Project.

2. Fill in the form:

   Name:
     Project identifier (no spaces, only letters/numbers/hyphens).
     Example: my-project

   Domain:
     Local subdomain. The '.wsdd.dock' suffix is added automatically.
     Example: type 'myapp' → final domain: myapp.wsdd.dock

   PHP Version:
     Select from PHP 5.6, 7.2, 7.4, 8.1, 8.2, 8.3, 8.4.

   Working Directory:
     Project root directory on local disk.
     Use 'Browse...' button to select folder.
     This directory is mounted as a volume in the Docker container.

   Entry Point:
     Main application file.
     Options: index.php, index.html, index.htm, custom (specify).

   SSL:
     Checkbox to generate local SSL certificate with mkcert.
     Recommended: enabled. Generates HTTPS automatically.

3. Click 'Deploy' to create the project.
   - If that PHP version does not yet have saved Webmin credentials and its
     container does not exist, WSDD opens a dialog to capture them.
   - Those credentials are stored per PHP version and reused for future
     projects on the same version.

4. Deploy process:
   a) Saves project to JSON
   b) Creates Docker volume
   c) Configures options.yml for PHP version
   d) Syncs Docker/Webmin resources for that PHP version
   e) Stops and removes previous PHP container
   f) Rebuilds PHP container with new project
   g) Generates vhost.conf for web server
   h) Generates SSL certificate (if enabled)
   i) Updates C:\Windows\System32\drivers\etc\hosts

## Deploy and Remove — detailed flow

DEPLOY FLOW:
  1. handlers::project::save()    → saves {name}.json in Docker-Structure/projects/
  2. docker volume create         → creates volume with project code
  3. handlers::yml::add_project_to_options_yml()
                                  → registers project in options.php{XX}.yml
  4. Syncs Dockerfile + webserver.yml + Webmin credentials for that PHP version
  5. docker stop + docker rm      → stops and removes previous PHP container
  6. docker create --build + up -d → rebuilds and starts PHP container
  7. Generate vhost.conf          → configures web server virtual host
  8. mkcert + SSL                 → generates certificate if SSL enabled
  9. Proxy restart                → restarts proxy to apply vhost
  10. handlers::hosts::update_host() → adds domain to /etc/hosts (Windows hosts)

REMOVE FLOW:
  1. handlers::yml::remove_project_from_options_yml()
                                  → removes project from options.yml
  2. Removes project vhost.conf block
  3. Rebuilds PHP container without the project
  4. handlers::hosts::remove_domains() → removes project domain from hosts
  5. Removes project SSL files from Docker-Structure/ssl
  6. docker volume rm             → removes project volume
  7. handlers::project::delete()  → deletes project JSON

IMPORTANT:
  - Remove does NOT delete project source code.
  - Remove reverts hosts entries and WSDD-managed SSL files for that domain.
  - Domain may take a few seconds to stop resolving after Remove.

## WSDD Settings

Access via: Tools → Settings

GENERAL:
  - Projects path    — Base directory for new projects (default: C:\WSDD-Projects)
  - Docker Desktop path  — Path to Docker Desktop executable (optional, for relaunching)
  - WSL Distro           — Active WSL2 distribution (e.g., Ubuntu-22.04)
  - Max log lines    — Limit of lines kept in log panel (100-10000)
  - Auto-start containers — Start WSDD containers on app launch

PHP (Docker containers):
  These values apply when GENERATING new containers.
  They do not affect existing containers (require redeploy).
  - memory_limit              — PHP RAM limit (e.g., 512M)
  - upload_max_filesize       — Max upload file size (e.g., 256M)
  - Timezone                  — PHP timezone (e.g., America/Mexico_City, UTC)
  - Xdebug                    — Enabled by default for new/rebuilt PHP containers.
    PHP 8.x uses Xdebug 3 with mode debug,develop, host.docker.internal,
    port 9003 and trigger-based start. PHP 5.6/7.x uses the Xdebug 2 trigger
    equivalent on the same host/port.

IDE / agent debugging:
  - Configure VS Code, PHPStorm, or another DBGp listener on port 9003.
  - Map the Windows project path to /var/www/html/{project-domain}.
  - AI agents can listen too if they run a DBGp/Xdebug-compatible listener on
    the Windows host; WSDD only configures the PHP container to connect back.

OPTIONAL SERVICES:
  Redis, Memcached and Mailpit are disabled by default and are not deployed with the base
  stack. Enable a service in Settings, review its ports/auto-start option, then
  save to deploy it.
  - Redis: container host redis / WSDD-Redis-Server, internal port 6379,
    default host port 6379, persistent volume wsdd-redis-data.
  - Memcached: container host memcached / WSDD-Memcached-Server, internal port
    11211, default host port 11211, volatile cache.
  - Mailpit: SMTP host mailpit / WSDD-Mailpit-Server, internal SMTP port 1025,
    UI port 8025, default local UI http://mailpit.wsdd.dock.
  - Framework examples:
    Redis: REDIS_HOST=redis, REDIS_PORT=6379.
    Memcached: MEMCACHED_HOST=memcached, MEMCACHED_PORT=11211.
    Mailpit: MAIL_HOST=mailpit, MAIL_PORT=1025, MAIL_MAILER=smtp.

PREREQUISITES:
  - MySQL/phpMyAdmin credentials — requested before the first base-environment
    deploy when they are still missing from configuration.
  - They are stored in wsdd-secrets.json and reused on later launches.

TOOLS:
  - Webmin version — Version installed in PHP containers (e.g., 2.630)
  - Webmin credentials by PHP version — requested only the first time a
    version is deployed and its container does not already exist.
  - Changing them later does not automatically rotate the existing user inside
    the container; they apply on the next WSDD-managed rebuild.

Settings are saved to: C:\WSDD-Environment\wsdd-config.json
Secrets are saved to: C:\WSDD-Environment\wsdd-secrets.json

## WSL2 Settings

Access via: Tools → WSL Settings

Modifies: %USERPROFILE%\.wslconfig

SYSTEM RESOURCES:
  - CPU Cores    — Limit cores assigned to WSL2.
                   'No limit' uses all available.
                   Recommended: 50-70% of physical cores.

  - Max RAM        — Limit RAM assigned to WSL2.
                     'No limit' allows WSL2 to consume all system RAM.
                     Recommended: 4-8 GB for typical WSDD stacks.

  - Swap              — Virtual swap space.
                        With sufficient RAM, 0 (disabled) is optimal.

PERFORMANCE AND MEMORY:
  - Memory reclaim — How WSL2 returns free RAM to Windows host:
    - Disabled: WSL2 retains RAM indefinitely (faster but uses more)
    - Gradual: releases memory gradually (recommended balance)
    - Drop Cache: aggressively releases on process termination (more free RAM on host)

  - GUI applications (WSLg) — Support for Linux apps with graphical interface.
                              Disable if not used to improve performance.

NETWORK:
  - Localhost forwarding — Access WSL2 ports via 127.0.0.1 on Windows.
                          WSDD requires this to be active.
  - Network mode:
    - NAT (recommended) — Isolated virtual network. Maximum compatibility.
    - Mirrored          — Shares host network. Experimental. Only Win11 23H2+.

IMPORTANT NOTE:
  Changes to .wslconfig require restarting WSL2:
  Open PowerShell as Admin and run: wsl --shutdown
  Then reopen Docker Desktop.

## SSL certificates and HTTPS

WSDD uses mkcert to generate locally-trusted SSL certificates.

HOW IT WORKS:
  1. mkcert creates a local Certificate Authority (CA) on your system.
  2. The CA is installed as trusted in Windows Certificate Store.
  3. For each project with SSL enabled, a certificate signed by the CA is generated.
  4. The reverse proxy uses certificates to serve HTTPS.

DOMAINS:
  All projects use the .wsdd.dock suffix
  Example: myapp.wsdd.dock → https://myapp.wsdd.dock

RENEW A CERTIFICATE:
  Remove the project and Deploy again.
  Certificate regenerates automatically.

BROWSER TRUST:
  If browser doesn't trust certificates:
  1. Verify mkcert is installed: mkcert -version (in PowerShell)
  2. Reinstall CA: mkcert -install
  3. Restart browser.

CERTIFICATE LOCATION:
  C:\WSDD-Environment\Docker-Structure\ssl\
  ├── {domain}.crt  — Certificate
  └── {domain}.key  — Private key

phpMyAdmin and MySQL SSL:
  HTTPS to phpMyAdmin secures browser → phpMyAdmin traffic. It does not mean
  the internal phpMyAdmin → MySQL connection uses MySQL TLS.
  WSDD does not force MySQL TLS by default because existing PHP frameworks and
  ORMs may need CA paths, ssl-mode settings and certificates. Treat MySQL TLS
  as an optional project-level hardening choice, not the default local stack.

## Troubleshooting

PROBLEM: Containers don't appear in the list.
  Solution:
  - Verify Docker Desktop is running (icon in system tray).
  - Click ↺ Refresh or go to Docker → Refresh container list.
  - Check log panel for connection error messages.

PROBLEM: 'Docker not found' when starting WSDD.
  Solution:
  - Install Docker Desktop manually from docker.com
  - Or let the Loader install it (requires Chocolatey).
  - After installation, a Windows restart may be required.

PROBLEM: The .wsdd.dock domain doesn't resolve in browser.
  Solution:
  - Verify project is in 'Deployed' state.
  - Check C:\Windows\System32\drivers\etc\hosts — should have a line with the domain.
  - If not: Remove and Deploy again.
  - Verify WSDD ran as Administrator.

PROBLEM: HTTPS shows certificate error.
  Solution:
  - Run in PowerShell: mkcert -install
  - Completely restart browser.
  - If persists: Remove + Deploy the project.

PROBLEM: Deploy fails with Docker error.
  Solution:
  - Check bottom log panel for specific error.
  - Verify Docker Desktop is in 'Running' state (not 'Starting').
  - Try Docker → Reload Docker Desktop from menu.
  - Manually restart Docker Desktop and wait for full load.

PROBLEM: PowerShell gives encoding error (invalid bytes).
  Solution:
  - WSDD uses -NoProfile -NonInteractive in all PS commands.
  - If you see encoding errors in log, verify pwsh.exe is PS7+:
    pwsh --version  (should be 7.x)

PROBLEM: Chocolatey doesn't install.
  Solution:
  - Check PowerShell execution policy:
    Get-ExecutionPolicy  (should be RemoteSigned or Bypass)
  - Run as Admin: Set-ExecutionPolicy RemoteSigned -Scope LocalMachine
  - Retry Loader (close and reopen WSDD).

PROBLEM: Log shows 'Access denied' when modifying hosts.
  Solution:
  - WSDD must run as Administrator.
  - Close WSDD and run wsdd.exe with right-click → Run as administrator.

## Frequently Asked Questions (FAQ)

Q: Can I have multiple PHP versions at the same time?
A: Yes. Each project has its own PHP version and container.
   You can have PHP 5.6, 8.1, and 8.4 simultaneously in different projects.

Q: Does WSDD modify my source code files?
A: No. WSDD only mounts your directory as a Docker volume (read-write).
   Files are not copied or modified internally.

Q: What happens if I delete a project from Windows Explorer?
A: The Docker container and WSDD registry still exist.
   Do 'Remove' from WSDD first to clean up properly,
   then delete the directory.

Q: Can I use WSDD with Laravel / Symfony / WordPress projects?
A: Yes, with any PHP framework. Configure Entry Point correctly:
   - Laravel/Symfony: public/index.php
   - WordPress: index.php
   - Custom: specify relative path to working directory.

Q: How do I update automation scripts?
A: PS1 scripts are embedded in the WSDD binary.
   To update, recompilation from source code is required.

Q: Does WSDD work with WSL 1?
A: No. Docker Desktop requires WSL 2. WSDD assumes WSL 2.

Q: Can I change an existing project's domain?
A: Currently no in-place editing. Do Remove and add the project
   again with the new domain.

Q: Where is project data saved?
A: C:\WSDD-Environment\Docker-Structure\projects\{name}.json

Q: How do I backup my WSDD projects?
A: Back up the complete directory of your source code (working path)
   and the JSON file from C:\WSDD-Environment\Docker-Structure\projects\
   The JSON contains project configuration (domain, PHP, SSL, etc.)

## File and path reference

```text
C:\WSDD-Environment\
├── wsdd-config.json              WSDD application settings
├── PS-Script\                   PowerShell automation scripts
│   ├── dd-detector.ps1           Environment and dependency detection
│   ├── dd-isinstalled.ps1        Checks if a tool is installed
│   ├── dd-isrunning.ps1          Checks if Docker is active
│   ├── dd-issettingup.ps1        Checks configuration state
│   ├── dd-setting.ps1            Read/write configuration
│   ├── dd-start.ps1              WSDD services start
│   ├── dd-stop.ps1               WSDD services stop
│   ├── dd-fixmysqlpermission.ps1 MySQL permissions repair
│   └── wsl-shutdown.ps1          WSL2 restart
├── Docker-Structure\
│   ├── bin\
│   │   └── php{X.X}\
│   │       └── options.php{XX}.yml   PHP container configuration
│   ├── ssl\
│   │   ├── {domain}.crt            Domain SSL certificate
│   │   └── {domain}.key            SSL private key
│   └── projects\
│       └── {name}.json            Data for each registered project
```

Other managed paths:

- `%USERPROFILE%\.wslconfig` — WSL2 resource configuration.
- `C:\Windows\System32\drivers\etc\hosts` — Hosts file modified by WSDD on deploy.

WSDD Logs:

Logs are in-memory during the session. Use the 'Copy' button to save them before closing WSDD.



