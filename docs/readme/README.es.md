# WebStack Deployer for Docker (WSDD)

Aplicacion de escritorio Windows que automatiza la configuracion de un entorno de desarrollo web
local usando Docker. Incluye PHP multi-version, SSL local, MySQL, phpMyAdmin, gestion de hosts,
Xdebug y servicios opcionales Redis/Memcached/Mailpit.

*Idiomas: [English](../../README.md) | [Español](README.es.md) | [Français](README.fr.md) | [हिन्दी](README.hi.md) | [中文](README.zh.md)*

*Enlaces rapidos: [Guia de usuario](../help/user-guide.es.md) | [Mapa de migracion](../../MIGRATION.md) | [Licencia](../legal/LICENSE.es.md) | [Repositorio principal](../../README.md) | [Reportar bug](https://github.com/wnunezc/wsdd-rust/issues/new)*

*Fallback de idioma: ingles para cualquier contenido de UI/ayuda no localizado.*

## Requisitos del sistema

- **Sistema operativo**: Windows 10 / Windows 11
- **Privilegios**: Administrador (obligatorio)
- **Docker Desktop**: Debe estar instalado por el usuario antes del primer arranque
- **WSL 2**: Requerido por Docker Desktop
- **Chocolatey**: Se instala automaticamente si no esta presente
- **PowerShell**: 7.5+ (se instala/actualiza automaticamente si falta)

## Que hace esta aplicacion

1. **Verifica y prepara dependencias**: Chocolatey, PowerShell 7.5+, Docker Desktop, MKCert
2. **Configura el stack Docker**: Nginx reverse proxy, MySQL, phpMyAdmin
3. **Gestiona proyectos web**: Crea contenedores PHP por version con Apache + Xdebug
4. **SSL local automatico**: Certificados MKCert por dominio, sin advertencias del navegador
5. **Hosts automaticos**: Modifica `C:\Windows\System32\drivers\etc\hosts` por ti
6. **Servicios opcionales**: Redis, Memcached y Mailpit estan desactivados por defecto y se despliegan solo al activarlos en Settings

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

### Servicios opcionales (desactivados por defecto)
- **WSDD-Redis-Server** — Redis para cache, colas y sesiones (`redis:7.4.8-alpine`)
- **WSDD-Memcached-Server** — Memcached para cache legacy (`memcached:1.6.39-alpine`)
- **WSDD-Mailpit-Server** — Captura SMTP local y UI web (`axllent/mailpit:v1.29.7`)

Los servicios opcionales usan compose aislado en `Docker-Structure/services/`, proyectos Compose
separados y la red compartida `wsdd-network`. No se despliegan con el stack base.

## Estructura del entorno en disco

La aplicacion crea y gestiona el directorio `C:\WSDD-Environment\`:

```
C:\WSDD-Environment\
├── PS-Script\          — Scripts PowerShell de automatizacion
├── Docker-Structure\   — docker-compose, imagenes PHP, servicios y assets SSL
├── wsdd-config.json    — Configuracion de la aplicacion
└── wsdd-secrets.json   — Secrets administrados para contenedores
```

## Primer arranque — proceso automatico

1. La aplicacion verifica que tiene privilegios de administrador
2. Extrae los recursos embebidos a `C:\WSDD-Environment\`
3. Comprueba Chocolatey → instala si falta
4. Comprueba PowerShell 7.5+ → instala/actualiza si falta
5. Comprueba Docker Desktop → bloquea si no esta instalado/configurado
6. Comprueba MKCert → instala y configura CA local
7. Levanta el stack Docker base
8. Muestra el panel principal

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

- **Version**: 1.0.0 (Rust edition)
- **GUI**: egui / eframe (immediate-mode)
- **Async**: tokio
- **Configuracion**: JSON en `C:\WSDD-Environment\wsdd-config.json`
- **Secrets**: JSON en `C:\WSDD-Environment\wsdd-secrets.json`
- **Logs**: Variable de entorno `RUST_LOG=wsdd=debug` para logs detallados

## Licencia

Propietaria — ver [LICENSE.es.md](../legal/LICENSE.es.md) para detalles.
Copyright (c) 2026 Walter Nunez / Icaros Net S.A. All Rights Reserved.
