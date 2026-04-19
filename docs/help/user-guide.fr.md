# WSDD Help

<!-- Canonical source for the in-app help rendered by src/ui/helps.rs. -->

## Configuration requise

Systeme d'exploitation:
  - Windows 10 (build 2004+) ou Windows 11
  - Architecture x64 (AMD64)
  - 8 Go RAM minimum — 16 Go recommandes pour les stacks PHP
  - 20 Go d'espace disque libre

Privileges:
  - WSDD doit s'executer en tant qu'Administrateur (UAC requis)
  - Necessaire pour modifier C:\Windows\System32\drivers\etc\hosts

Logiciels installes automatiquement par WSDD:
  - Chocolatey — gestionnaire de paquets Windows
  - Docker Desktop — moteur de conteneurs
  - mkcert — generation de certificats SSL locaux
  - WSL 2 — sous-systeme Linux (requis par Docker Desktop)

Logiciels a installer au prealable:
  - PowerShell 7 (pwsh.exe) — obligatoire pour l'automatisation

## Installation et premier lancement

1. Executer wsdd.exe en tant qu'Administrateur (clic droit → Executer en tant qu'administrateur).

2. Au premier lancement, l'Assistant de bienvenue apparait:
   - Lire et cocher la case des prerequis.
   - Cliquer sur 'Suivant' pour demarrer la verification de l'environnement.

3. Avant le premier deploy de l'environnement de base, WSDD ouvre un dialogue
   pour capturer les identifiants MySQL et phpMyAdmin.
   - Ces identifiants sont enregistres dans wsdd-config.json.
   - Ils ne sont demandes que s'ils manquent dans la configuration.

4. Le Loader verifie et installe automatiquement:
   - Chocolatey (gestionnaire de paquets)
   - Docker Desktop
   - mkcert (SSL local)
   - Configuration reseau et certificat racine

4. Si Docker Desktop n'etait pas installe:
   - WSDD l'installe via Chocolatey.
   - Un redemarrage du systeme peut etre necessaire.
   - Apres le redemarrage, relancer WSDD.

5. Une fois le Loader termine, le Panneau Principal apparait.

Emplacement des donnees WSDD:
  C:\WSDD-Environment\
  ├── PS-Script\         Scripts d'automatisation PowerShell
  ├── Docker-Structure\  Configuration des conteneurs, projets et SSL
  └── wsdd-config.json  Parametres de l'application

## Panneau principal — apercu

Le panneau principal comporte trois zones:

1. BARRE DE MENU (haut):
   - Fichier → Ajouter Projet, Quitter
   - Docker → Actualiser liste, Recharger Docker Desktop, Effacer logs
   - Outils → Parametres WSL, Parametres
   - Aide → Aide, A propos...

2. BARRE D'OUTILS (sous le menu):
   - ⬡ phpMyAdmin   — Ouvrir phpMyAdmin dans le navigateur
   - ⚡ Terminal PS  — Ouvrir PowerShell 7 dans C:\WSDD-Environment
   - ⬛ Terminal CMD — Ouvrir CMD dans C:\WSDD-Environment
   - + Ajouter      — Formulaire nouveau projet
   - ↺ Actualiser   — Mettre a jour conteneurs et projets
   - Selecteur de theme (droite) — Basculer entre themes

3. ONGLETS CENTRAUX:
   - Conteneurs   — Liste des conteneurs Docker actifs/inactifs
   - Projets      — Liste des projets WSDD enregistres

4. PANNEAU DE LOG (bas):
   - Affiche l'historique des operations avec code couleur.
   - Bouton 'Copier' — copie le log dans le presse-papiers.
   - Bouton 'Effacer' — efface le log visible.

## Gestion des conteneurs Docker

L'onglet 'Conteneurs' affiche tous les conteneurs WSDD avec leur etat actuel.

COLONNES:
  - Nom       — Nom du conteneur Docker
  - Statut    — Running / Exited / ...
  - Image     — Image Docker de base
  - Toolbox (⚙)  — Actions avancees du conteneur

ACTIONS PAR CONTENEUR:
  - ▶ Demarrer   — Demarrer le conteneur
  - ■ Arreter    — Arreter le conteneur
  - ↺ Redemarrer — Redemarrer le conteneur

TOOLBOX CONTENEUR (bouton ⚙):
  - Voir les logs du conteneur en temps reel
  - Ouvrir un terminal TTY interactif dans le conteneur
  - Voir les URLs et ports exposes
  - Informations detaillees (image, ports, volumes)

POLLING AUTOMATIQUE:
  Les conteneurs se mettent a jour automatiquement toutes les 3 secondes.
  L'actualisation manuelle n'est pas necessaire.

## Gestion des projets

L'onglet 'Projets' affiche les projets web enregistres dans WSDD.

COLONNES:
  - Nom       — Nom du projet
  - Domaine   — Domaine local (ex: monapp.wsdd.dock)
  - PHP       — Version PHP assignee (5.6 — 8.4)
  - Statut    — Deployed / Not Deployed
  - Actions   — Deploy, Remove, Toolbox

ACTIONS:
  - ⬆ Deploy    — Deployer le projet (cree conteneurs, SSL, hosts)
  - ⬇ Remove    — Supprimer le deploiement (NE supprime PAS les fichiers sources)
  - ⚙ Toolbox   — Actions avancees du projet

TOOLBOX PROJET:
  - Ouvrir le dossier du projet dans l'Explorateur Windows
  - Ouvrir le projet dans le navigateur
  - Voir les informations detaillees (chemins, domaine, entrypoint)

SUPPRIMER PROJET:
  - Cliquer sur Remove demande une confirmation.
  - Sont supprimes: conteneurs, SSL, entree hosts, enregistrement JSON.
  - Les fichiers de code source NE sont PAS supprimes.

## Ajouter un projet

Pour ajouter un nouveau projet web:

1. Cliquer sur '+ Ajouter' dans la barre d'outils ou aller dans Fichier → Ajouter Projet.

2. Remplir le formulaire:

   Nom:
     Identifiant du projet (sans espaces, uniquement lettres/chiffres/tirets).
     Exemple: mon-projet

   Domaine:
     Sous-domaine local. Le suffixe '.wsdd.dock' est ajoute automatiquement.
     Exemple: taper 'monapp' → domaine final: monapp.wsdd.dock

   Version PHP:
     Selectionner parmi PHP 5.6, 7.2, 7.4, 8.1, 8.2, 8.3, 8.4.

   Repertoire de travail:
     Repertoire racine du projet sur le disque local.
     Utiliser le bouton 'Parcourir...' pour selectionner le dossier.

   Point d'entree:
     Fichier principal de l'application.
     Options: index.php, index.html, index.htm, personnalise.

   SSL:
     Case a cocher pour generer un certificat SSL local avec mkcert.
     Recommande: active. Genere HTTPS automatiquement.

3. Cliquer sur 'Deploy' pour creer le projet.

## Deploy et Remove — flux detaille

FLUX DEPLOY:
  1. handlers::project::save()    → sauvegarde {nom}.json
  2. docker volume create         → cree le volume avec le code du projet
  3. Enregistre le projet dans options.php{XX}.yml
  4. docker stop + docker rm      → arrete et supprime le conteneur PHP precedent
  5. docker create --build + up -d → reconstruit et demarre le conteneur PHP
  6. Genere vhost.conf            → configure le virtual host
  7. mkcert + SSL                 → genere le certificat si SSL actif
  8. Proxy restart                → redemarre le proxy
  9. Met a jour le fichier hosts

FLUX REMOVE:
  1. Supprime le projet du options.yml
  2. Supprime le bloc vhost.conf du projet
  3. Reconstruit le conteneur PHP sans le projet
  4. Retire le domaine du fichier hosts
  5. Supprime les fichiers SSL du projet dans Docker-Structure/ssl
  6. docker volume rm             → supprime le volume du projet
  7. Supprime le JSON du projet

IMPORTANT:
  - Remove NE supprime PAS le code source du projet.
  - Remove revert aussi les entrees hosts et les SSL geres par WSDD.
  - Le domaine peut mettre quelques secondes a ne plus resoudre.

## Parametres WSDD

Acces via: Outils → Parametres

GENERAL:
  - Chemin des projets    — Repertoire de base pour nouveaux projets (defaut: C:\WSDD-Projects)
  - Chemin Docker Desktop — Chemin vers l'executable Docker Desktop (optionnel, pour le relancer)
  - Distro WSL            — Distribution WSL2 active (ex: Ubuntu-22.04)
  - Max lignes de log     — Limite de lignes conservees dans le panneau de log (100-10000)
  - Demarrage auto        — Demarre les conteneurs WSDD au lancement

PHP (conteneurs Docker):
  Ces valeurs s'appliquent lors de la GENERATION de nouveaux conteneurs.
  Elles n'affectent pas les conteneurs existants (redeploy requis).
  - memory_limit              — Limite RAM pour PHP
  - upload_max_filesize       — Taille max des fichiers uploades
  - Timezone                  — Fuseau horaire PHP
  - Xdebug                    — Active par defaut pour les conteneurs PHP nouveaux
    ou reconstruits. PHP 8.x utilise Xdebug 3 avec mode debug,develop,
    host.docker.internal, port 9003 et demarrage par trigger. PHP 5.6/7.x
    utilise l'equivalent trigger de Xdebug 2 sur le meme host/port.

Debug IDE / agents:
  - Configurer VS Code, PHPStorm ou un autre listener DBGp sur le port 9003.
  - Mapper le chemin Windows du projet vers /var/www/html/{domaine-du-projet}.
  - Les agents IA peuvent aussi ecouter s'ils executent un listener compatible
    DBGp/Xdebug sur le host Windows; WSDD configure seulement le conteneur PHP
    pour se reconnecter.

SERVICES OPTIONNELS:
  Redis, Memcached et Mailpit sont desactives par defaut et ne sont pas deployes
  avec la stack de base. Activer le service dans Settings, verifier ports/auto-start,
  puis enregistrer pour le deployer.
  - Redis: host de conteneur redis / WSDD-Redis-Server, port interne 6379,
    port host par defaut 6379, volume persistant wsdd-redis-data.
  - Memcached: host de conteneur memcached / WSDD-Memcached-Server, port
    interne 11211, port host par defaut 11211, cache volatile.
  - Mailpit: host SMTP mailpit / WSDD-Mailpit-Server, port SMTP interne 1025,
    UI sur port 8025, UI locale par defaut http://mailpit.wsdd.dock.
  - Exemples frameworks:
    Redis: REDIS_HOST=redis, REDIS_PORT=6379.
    Memcached: MEMCACHED_HOST=memcached, MEMCACHED_PORT=11211.
    Mailpit: MAIL_HOST=mailpit, MAIL_PORT=1025, MAIL_MAILER=smtp.

PREREQUIS:
  - Identifiants MySQL/phpMyAdmin — demandes avant le premier deploy
    de l'environnement de base s'ils n'existent pas encore dans la configuration.
  - Ils sont enregistres dans wsdd-secrets.json et reutilises ensuite.

OUTILS:
  - Version Webmin — Version installee dans les conteneurs PHP (ex: 2.630)
  - Identifiants Webmin par version PHP — demandes une seule fois lors du
    premier deploy d'une version dont le conteneur n'existe pas encore.
  - Les modifier plus tard ne fait pas tourner automatiquement l'utilisateur
    existant dans le conteneur; ils s'appliquent au prochain rebuild gere par WSDD.

Les modifications sont enregistrees dans: C:\WSDD-Environment\wsdd-config.json
Les secrets sont enregistres dans: C:\WSDD-Environment\wsdd-secrets.json

## Parametres WSL2

Acces via: Outils → Parametres WSL

Modifie: %USERPROFILE%\.wslconfig

RESSOURCES SYSTEME:
  - Coeurs CPU    — Limiter les coeurs assignes a WSL2.
  - RAM max       — Limiter la RAM assignee a WSL2.
  - Swap          — Espace d'echange virtuel.

PERFORMANCE ET MEMOIRE:
  - Recuperation memoire — Comment WSL2 libere la RAM vers l'hote Windows.
  - Applications GUI (WSLg) — Support des apps Linux avec interface graphique.

RESEAU:
  - Localhost forwarding — Acceder aux ports WSL2 via 127.0.0.1.
  - Mode reseau:
    - NAT (recommande) — Reseau virtuel isole.
    - Mirrored — Partage le reseau de l'hote. Experimental.

NOTE IMPORTANTE:
  Les modifications de .wslconfig necessitent un redemarrage de WSL2:
  Ouvrir PowerShell en Admin et executer: wsl --shutdown

## Certificats SSL et HTTPS

WSDD utilise mkcert pour generer des certificats SSL de confiance locale.

FONCTIONNEMENT:
  1. mkcert cree une Autorite de Certification (CA) locale sur votre systeme.
  2. La CA est installee comme approuvee dans le Windows Certificate Store.
  3. Pour chaque projet avec SSL actif, un certificat signe par la CA est genere.
  4. Le proxy inverse utilise les certificats pour servir HTTPS.

DOMAINES:
  Tous les projets utilisent le suffixe .wsdd.dock
  Exemple: monapp.wsdd.dock → https://monapp.wsdd.dock

RENOUVELER UN CERTIFICAT:
  Faire Remove du projet puis Deploy a nouveau.
  Le certificat est regenere automatiquement.

EMPLACEMENT DES CERTIFICATS:
  C:\WSDD-Environment\Docker-Structure\ssl\
  ├── {domaine}.crt  — Certificat
  └── {domaine}.key  — Cle privee

phpMyAdmin et SSL MySQL:
  HTTPS vers phpMyAdmin protege le trafic navigateur → phpMyAdmin. Cela ne signifie
  pas que la connexion interne phpMyAdmin → MySQL utilise TLS MySQL.
  WSDD ne force pas TLS MySQL par defaut, car les frameworks et ORM existants
  peuvent necessiter des chemins CA, des parametres ssl-mode et des certificats.
  Traiter TLS MySQL comme un durcissement optionnel par projet, pas comme le
  comportement par defaut de la stack locale.

## Depannage

PROBLEME: Les conteneurs n'apparaissent pas dans la liste.
  Solution:
  - Verifier que Docker Desktop est en cours d'execution.
  - Cliquer sur ↺ Actualiser.
  - Verifier le panneau de log pour les messages d'erreur.

PROBLEME: 'Docker non trouve' au demarrage de WSDD.
  Solution:
  - Installer Docker Desktop manuellement depuis docker.com
  - Ou laisser le Loader l'installer.
  - Apres l'installation, un redemarrage peut etre necessaire.

PROBLEME: Le domaine .wsdd.dock ne resout pas dans le navigateur.
  Solution:
  - Verifier que le projet est en etat 'Deployed'.
  - Verifier le fichier hosts.
  - Si absent: faire Remove et Deploy a nouveau.
  - Verifier que WSDD s'est execute en tant qu'Administrateur.

PROBLEME: HTTPS affiche une erreur de certificat.
  Solution:
  - Executer dans PowerShell: mkcert -install
  - Redemarrer completement le navigateur.
  - Si persiste: Remove + Deploy du projet.

PROBLEME: Le deploy echoue avec une erreur Docker.
  Solution:
  - Verifier le panneau de log pour l'erreur specifique.
  - Verifier que Docker Desktop est en etat 'Running'.
  - Essayer Docker → Recharger Docker Desktop depuis le menu.

## Questions frequentes (FAQ)

Q: Puis-je avoir plusieurs versions de PHP en meme temps?
R: Oui. Chaque projet a sa propre version PHP et son propre conteneur.

Q: WSDD modifie-t-il mes fichiers de code source?
R: Non. WSDD monte uniquement votre repertoire comme volume Docker.
   Les fichiers ne sont ni copies ni modifies en interne.

Q: Que se passe-t-il si je supprime un projet depuis l'Explorateur Windows?
R: Le conteneur Docker et l'enregistrement WSDD existent toujours.
   Faire 'Remove' depuis WSDD d'abord pour nettoyer correctement.

Q: Puis-je utiliser WSDD avec des projets Laravel / Symfony / WordPress?
R: Oui, avec n'importe quel framework PHP. Configurer le Point d'entree:
   - Laravel/Symfony: public/index.php
   - WordPress: index.php

Q: Comment mettre a jour les scripts d'automatisation?
R: Les scripts PS1 sont embarques dans le binaire WSDD.
   Pour mettre a jour, une recompilation depuis le code source est necessaire.

Q: WSDD fonctionne-t-il avec WSL 1?
R: Non. Docker Desktop necessite WSL 2. WSDD suppose WSL 2.

## Reference des fichiers et chemins

```text
C:\WSDD-Environment\
├── wsdd-config.json              Parametres de l'application WSDD
├── PS-Script\                   Scripts d'automatisation PowerShell
├── Docker-Structure\
│   ├── bin\
│   │   └── php{X.X}\
│   │       └── options.php{XX}.yml   Configuration des conteneurs PHP
│   ├── ssl\
│   │   ├── {domaine}.crt            Certificat SSL du domaine
│   │   └── {domaine}.key            Cle privee SSL
│   └── projects\
│       └── {nom}.json            Donnees de chaque projet enregistre
```

Autres chemins geres:

- `%USERPROFILE%\.wslconfig` — Configuration des ressources WSL2.
- `C:\Windows\System32\drivers\etc\hosts` — Fichier hosts modifie par WSDD lors du deploy.

Logs WSDD:

Les logs restent en memoire pendant la session. Utiliser le bouton 'Copier' pour les sauvegarder avant de fermer WSDD.



