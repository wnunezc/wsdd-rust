# WebStack Deployer for Docker (WSDD)

Aplicacion de escritorio Windows que automatiza la configuracion de un entorno de desarrollo web
local usando Docker. Incluye PHP multi-version, SSL local, MySQL, phpMyAdmin y gestion de hosts.

*Idiomas: [English](../../README.md) | [Español](README.es.md) | [Français](README.fr.md) | [हिन्दी](README.hi.md) | [中文](README.zh.md)*

## Requisitos del sistema

- **Sistema operativo**: Windows 10 / Windows 11
- **Privilegios**: Administrador (obligatorio)
- **Docker Desktop**: Se instala automaticamente si no esta presente
- **WSL 2**: Se configura automaticamente
- **Chocolatey**: Se instala automaticamente si no esta presente

## Que hace esta aplicacion

1. **Verifica e instala dependencias**: Docker Desktop, WSL 2, Chocolatey, MKCert
2. **Configura el stack Docker**: Nginx reverse proxy, MySQL, phpMyAdmin
3. **Gestiona proyectos web**: Crea contenedores PHP por version con Apache + Xdebug
4. **SSL local automatico**: Certificados MKCert por dominio, sin advertencias del navegador
5. **Hosts automaticos**: Modifica `C:\Windows\System32\drivers\etc\hosts` por ti

## Contenedores Docker del stack

### Servicios base (siempre activos)
- **WSDD-Proxy-Server** — Nginx reverse proxy (puertos 80 / 443)
- **WSDD-MySql-Server** — MySQL 8 (puerto 3306)
- **WSDD-phpMyAdmin-Server** — phpMyAdmin

### Contenedores PHP (uno por version usada)
Versiones disponibles: 5.6 - 7.2 - 7.4 - 8.1 - 8.2 - 8.3 - 8.4

Por cada version activada se crean las URLs de desarrollo:
- `php{version}.wsdd.dock` — Entorno PHP principal
- `cron{version}.wsdd.dock` — Gestor de cron jobs
- `wm{version}.wsdd.dock` — Webmin (administracion del servidor)

## Estructura del entorno en disco

La aplicacion crea y gestiona el directorio `C:\WSDD-Environment\`:

```
C:\WSDD-Environment\
├── PS-Script\          — Scripts PowerShell de automatizacion
├── Docker-Structure\   — docker-compose e imagenes PHP
├── certs\              — Certificados SSL por dominio
└── wsdd-config.json    — Configuracion de la aplicacion
```

## Primer arranque — proceso automatico

1. La aplicacion verifica que tiene privilegios de administrador
2. Extrae los recursos embebidos a `C:\WSDD-Environment\`
3. Comprueba Chocolatey → instala si falta
4. Comprueba Docker Desktop → instala si falta (requiere reinicio)
5. Comprueba MKCert → instala y configura CA local
6. Levanta el stack Docker base
7. Muestra el panel principal

> **Nota**: La instalacion de Docker Desktop puede requerir reinicio del sistema.
> La aplicacion retomara automaticamente tras el reinicio.

## Uso despues del primer arranque

### Agregar un proyecto
1. Haz clic en "Agregar Proyecto"
2. Elige el dominio local (ej: `miproyecto.wsdd.dock`)
3. Selecciona la version de PHP
4. La aplicacion crea el contenedor, el certificado SSL y la entrada de hosts

### Gestionar contenedores
- Inicia / detiene contenedores individuales desde el panel principal
- Abre logs en tiempo real con un clic
- Reinicia el stack completo desde el menu

## Informacion tecnica

- **Version**: 1.0.0-rc.2 (Rust edition)
- **GUI**: egui / eframe (immediate-mode)
- **Async**: tokio
- **Configuracion**: JSON en `C:\WSDD-Environment\wsdd-config.json`
- **Logs**: Variable de entorno `RUST_LOG=wsdd=debug` para logs detallados

## Licencia

Propietaria — ver `../legal/LICENSE.es.txt` para detalles.
Copyright (c) 2026 Walter Nunez / Icaros Net S.A. All Rights Reserved.
