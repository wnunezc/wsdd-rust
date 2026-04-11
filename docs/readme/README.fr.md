# WebStack Deployer for Docker (WSDD)

Application de bureau Windows qui automatise la configuration d'un environnement de developpement web local
avec Docker. Inclut PHP multi-version, SSL local, MySQL, phpMyAdmin et la gestion du fichier hosts.

*Langues: [English](../../README.md) | [Español](README.es.md) | [Français](README.fr.md) | [हिन्दी](README.hi.md) | [中文](README.zh.md)*

*Liens rapides: [Carte de migration](../../MIGRATION.md) | [Licence](../legal/LICENSE.fr.md) | [Depot principal](../../README.md) | [Signaler un bug](https://github.com/wnunezc/wsdd-rust/issues/new)*

## Configuration systeme requise

- **Systeme d'exploitation**: Windows 10 / Windows 11
- **Privileges**: Administrateur (obligatoire)
- **Docker Desktop**: Installe automatiquement s'il n'est pas present
- **WSL 2**: Configure automatiquement
- **Chocolatey**: Installe automatiquement s'il n'est pas present

## Ce que fait cette application

1. **Verifie et installe les dependances**: Docker Desktop, WSL 2, Chocolatey, MKCert
2. **Configure la stack Docker**: Nginx reverse proxy, MySQL, phpMyAdmin
3. **Gere les projets web**: Cree des conteneurs PHP par version avec Apache + Xdebug
4. **SSL local automatique**: Certificats MKCert par domaine, sans avertissements du navigateur
5. **Hosts automatiques**: Modifie `C:\Windows\System32\drivers\etc\hosts` pour vous

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

## Structure de l'environnement sur disque

L'application cree et gere le repertoire `C:\WSDD-Environment\`:

```
C:\WSDD-Environment\
├── PS-Script\          — Scripts PowerShell d'automatisation
├── Docker-Structure\   — docker-compose et images PHP
├── certs\              — Certificats SSL par domaine
└── wsdd-config.json    — Configuration de l'application
```

## Premier lancement — processus automatique

1. L'application verifie qu'elle dispose des privileges administrateur
2. Extrait les ressources integrees vers `C:\WSDD-Environment\`
3. Verifie Chocolatey → l'installe s'il manque
4. Verifie Docker Desktop → l'installe s'il manque (redemarrage requis)
5. Verifie MKCert → installe et configure l'autorite locale
6. Demarre la stack Docker de base
7. Affiche le panneau principal

> **Note**: L'installation de Docker Desktop peut necessiter un redemarrage du systeme.
> L'application reprendra automatiquement apres le redemarrage.

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

- **Version**: 1.0.0-rc.5 (edition Rust)
- **GUI**: egui / eframe (immediate-mode)
- **Async**: tokio
- **Configuration**: JSON dans `C:\WSDD-Environment\wsdd-config.json`
- **Logs**: Variable d'environnement `RUST_LOG=wsdd=debug` pour des logs detailles

## Licence

Proprietaire — voir [LICENSE.fr.md](../legal/LICENSE.fr.md) pour les details.
Copyright (c) 2026 Walter Nunez / Icaros Net S.A. All Rights Reserved.
