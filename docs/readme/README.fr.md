# WebStack Deployer for Docker (WSDD)

Application de bureau Windows qui automatise la configuration d'un environnement de developpement web local
avec Docker. Inclut PHP multi-version, SSL local, MySQL, phpMyAdmin, la gestion du fichier hosts,
Xdebug et les services optionnels Redis/Memcached/Mailpit.

*Langues: [English](../../README.md) | [Español](README.es.md) | [Français](README.fr.md) | [हिन्दी](README.hi.md) | [中文](README.zh.md)*

*Liens rapides: [Guide utilisateur](../help/user-guide.fr.md) | [Carte de migration](../../MIGRATION.md) | [Licence](../legal/LICENSE.fr.md) | [Notice](../../NOTICE.md) | [Licences tierces](../../THIRD_PARTY_LICENSES.md) | [Contribuer](../../CONTRIBUTING.md) | [Securite](../../SECURITY.md) | [Discussions](https://github.com/wnunezc/wsdd-rust/discussions) | [Signaler un bug](https://github.com/wnunezc/wsdd-rust/issues/new)*

*Fallback de langue: anglais pour tout contenu UI/aide non localise.*

## A propos de WSDD

WSDD est un gestionnaire de stack local, oriente Windows, pour le developpement PHP + Docker.
Il automatise la preparation initiale de l'environnement, provisionne des conteneurs PHP par
projet, configure le SSL local avec MKCert, met a jour le fichier Windows `hosts` et centralise
les operations de conteneurs/projets dans une seule application de bureau.

- **Etape actuelle**: release stable `1.0.0`
- **Paquet principal de distribution**: installateur MSI Windows
- **Langues UI actuelles**: anglais, espagnol, francais, hindi, chinois
- **Fallback de langue**: anglais pour tout contenu UI/aide localise manquant
- **Signalement d'issues**: [GitHub Issues](https://github.com/wnunezc/wsdd-rust/issues/new)

## Configuration systeme requise

- **Systeme d'exploitation**: Windows 10 / Windows 11
- **Privileges**: Administrateur (obligatoire)
- **Docker Desktop**: Doit etre installe par l'utilisateur avant le premier lancement
- **WSL 2**: Requis par Docker Desktop
- **Chocolatey**: Installe automatiquement s'il n'est pas present
- **PowerShell**: 7.5+ (installe/mis a jour automatiquement si absent)

## Ce que fait cette application

1. **Verifie et prepare les dependances**: Chocolatey, PowerShell 7.5+, Docker Desktop, MKCert
2. **Configure la stack Docker**: Nginx reverse proxy, MySQL, phpMyAdmin
3. **Gere les projets web**: Cree des conteneurs PHP par version avec Apache + Xdebug
4. **SSL local automatique**: Certificats MKCert par domaine, sans avertissements du navigateur
5. **Hosts automatiques**: Modifie `C:\Windows\System32\drivers\etc\hosts` pour vous
6. **Services optionnels**: Redis, Memcached et Mailpit sont desactives par defaut et deployes seulement apres activation dans Settings

## Conteneurs Docker de la stack

### Services de base (toujours actifs)
- **WSDD-Proxy-Server** — Nginx reverse proxy (ports 80 / 443)
- **WSDD-MySql-Server** — MySQL 8 (port 3306)
- **WSDD-phpMyAdmin-Server** — phpMyAdmin

### Conteneurs PHP (un par version utilisee)
Versions disponibles: 5.6 - 7.2 - 7.4 - 8.1 - 8.2 - 8.3 - 8.4

Pour chaque version activee, les URL de developpement suivantes sont creees:
- `php{version}.wsdd.dock` — Environnement PHP principal
- `cron{version}.wsdd.dock` — Gestionnaire de taches cron
- `wm{version}.wsdd.dock` — Webmin (administration du serveur)

### Services optionnels (desactives par defaut)
- **WSDD-Redis-Server** — Redis pour cache, files et sessions (`redis:7.4.8-alpine`)
- **WSDD-Memcached-Server** — Memcached pour cache legacy (`memcached:1.6.39-alpine`)
- **WSDD-Mailpit-Server** — Capture SMTP locale et UI web (`axllent/mailpit:v1.29.7`)

Les services optionnels utilisent des compose isoles dans `Docker-Structure/services/`, des projets
Compose separes et le reseau partage `wsdd-network`. Ils ne sont pas deployes avec la stack de base.

## Structure de l'environnement sur disque

L'application cree et gere le repertoire `C:\WSDD-Environment\`:

```
C:\WSDD-Environment\
├── PS-Script\          — Scripts PowerShell d'automatisation
├── Docker-Structure\   — docker-compose, images PHP, services et assets SSL
├── wsdd-config.json    — Configuration de l'application
└── wsdd-secrets.json   — Secrets geres pour les conteneurs
```

## Premier lancement — processus automatique

1. L'application verifie qu'elle dispose des privileges administrateur
2. Extrait les ressources integrees vers `C:\WSDD-Environment\`
3. Verifie Chocolatey → l'installe s'il manque
4. Verifie PowerShell 7.5+ → l'installe/met a jour si absent
5. Verifie Docker Desktop → bloque s'il n'est pas installe/configure
6. Verifie MKCert → installe et configure l'autorite locale
7. Demarre la stack Docker de base
8. Affiche le panneau principal

## Utilisation apres le premier lancement

### Ajouter un projet
1. Cliquez sur "Ajouter un projet"
2. Choisissez le domaine local (ex: `monprojet.wsdd.dock`)
3. Selectionnez la version de PHP
4. L'application cree le conteneur, le certificat SSL et l'entree hosts

### Gerer les conteneurs
- Demarrer / arreter des conteneurs individuels depuis le panneau principal
- Ouvrir les journaux en temps reel en un clic
- Redemarrer toute la stack depuis le menu

## Informations techniques

- **Version**: 1.0.0 (edition Rust)
- **GUI**: egui / eframe (immediate-mode)
- **Async**: tokio
- **Configuration**: JSON dans `C:\WSDD-Environment\wsdd-config.json`
- **Secrets**: JSON dans `C:\WSDD-Environment\wsdd-secrets.json`
- **Logs**: Variable d'environnement `RUST_LOG=wsdd=debug` pour des logs detailles

## Licence

Proprietaire — voir [LICENSE.fr.md](../legal/LICENSE.fr.md) pour les details.
Copyright (c) 2026 Walter Nunez / Icaros Net S.A. All Rights Reserved.
