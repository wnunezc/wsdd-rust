# WSDD Help

<!-- Canonical source for the in-app help rendered by src/ui/helps.rs. -->

## सिस्टम आवश्यकताएँ

WSDD Windows पर local PHP + Docker development stack चलाने के लिए बनाया गया है।

न्यूनतम आवश्यकताएँ:
  - Windows 10 / Windows 11
  - x64 architecture (AMD64)
  - Administrator privileges
  - Docker Desktop installed and configured
  - WSL2 enabled
  - Internet connection for first setup

WSDD first run पर इन tools को verify या prepare करता है:
  - Chocolatey
  - PowerShell 7.5+
  - Docker Desktop
  - mkcert

Docker Desktop user को पहले से install करना चाहिए. WSDD Docker install नहीं करता, लेकिन missing या misconfigured Docker Desktop पर setup रोक देता है।

## Installation and first launch

1. MSI installer चलाएँ।
2. Start Menu से WSDD खोलें।
3. Administrator prompt accept करें।
4. Welcome screen पर prerequisites पढ़कर checkbox mark करें।
5. Continue दबाएँ।
6. Loader setup शुरू करेगा।

First launch flow:
  - Embedded resources extract होते हैं: `C:\WSDD-Environment\`
  - PowerShell scripts copy होते हैं: `C:\WSDD-Environment\PS-Script\`
  - Docker structure copy होती है: `C:\WSDD-Environment\Docker-Structure\`
  - `wsdd-config.json` और `wsdd-secrets.json` create या migrate होते हैं।
  - Chocolatey, PowerShell, Docker Desktop और mkcert verify होते हैं।
  - Base Docker stack start होता है।

Base stack containers:
  - `WSDD-Proxy-Server`
  - `WSDD-MySql-Server`
  - `WSDD-phpMyAdmin-Server`

अगर reboot की ज़रूरत हो तो WSDD message दिखाएगा. Reboot के बाद WSDD फिर खोलें।

## Main panel — overview

Main panel WSDD का daily dashboard है।

मुख्य क्षेत्र:
  - Top toolbar: Add Project, refresh, terminal shortcuts.
  - Containers panel: Docker containers और status.
  - Projects panel: deployed projects और domains.
  - Bottom logs: general log और live container logs.
  - Status bar: container count, project count, Docker status, CPU/RAM poll.

Menu:
  - File → Add Project, Exit
  - Tools → Settings, WSL Settings, open terminals
  - Help → Help, About

Logs:
  - General log WSDD actions दिखाता है।
  - Container log running WSDD containers से live `docker logs --tail 200 --follow` stream करता है।
  - Long lines wrap होती हैं ताकि panel horizontally expand न हो।

## Docker container management

Containers panel WSDD-managed Docker containers दिखाता है।

Statuses:
  - Running
  - Stopped
  - Missing
  - Unknown

Common actions:
  - Start / stop WSDD containers.
  - Open browser URLs from container toolbox.
  - Inspect live logs in the bottom panel.
  - Reload state from Docker.

Base containers:
  - Proxy: HTTP/HTTPS reverse proxy.
  - MySQL: database server.
  - phpMyAdmin: database UI.

PHP containers per version:
  - `WSDD-Web-Server-PHP5.6`
  - `WSDD-Web-Server-PHP7.2`
  - `WSDD-Web-Server-PHP7.4`
  - `WSDD-Web-Server-PHP8.1`
  - `WSDD-Web-Server-PHP8.2`
  - `WSDD-Web-Server-PHP8.3`
  - `WSDD-Web-Server-PHP8.4`

Optional services:
  - `WSDD-Redis-Server`
  - `WSDD-Memcached-Server`
  - `WSDD-Mailpit-Server`

Optional services are configured in their Tools dialogs. Save stores options; Deploy applies the selected service.

## Project management

Projects panel उन projects को दिखाता है जिन्हें WSDD ने deploy किया है।

Per project data:
  - Project name
  - Local domain
  - PHP version
  - SSL status
  - Deploy/remove actions
  - Toolbox shortcuts

Toolbox shortcuts:
  - Website URL
  - Webmin URL
  - Cron URL
  - Project folder
  - Logs and container actions

Remove का मतलब:
  - WSDD project deployment हटाता है।
  - hosts entries और WSDD-managed SSL files साफ करता है।
  - Source code files delete नहीं करता।

## Adding a project

Add Project से नया local PHP project deploy होता है।

Required fields:
  - Project name
  - Local domain
  - PHP version
  - Working directory
  - Entry point

Domain:
  - `myapp` लिखने पर WSDD उसे `myapp.wsdd.dock` domain की तरह manage करता है।
  - hosts entry Windows hosts file में add होती है।

Entry point:
  - Project root
  - `/public` for frameworks like Laravel/Symfony
  - Custom folder

Deploy creates:
  - Project metadata JSON
  - SSL certificate when enabled
  - hosts entry
  - Nginx vhost
  - PHP container/image for selected version if needed

## Deploy and Remove — detailed flow

Deploy:
  1. Project settings validate होते हैं।
  2. Domain normalize होता है।
  3. hosts entry prepare होती है।
  4. mkcert SSL files generate करता है।
  5. Docker/Webmin resources selected PHP version के लिए sync होते हैं।
  6. Nginx vhost regenerate होता है।
  7. Docker compose build/up चलता है।
  8. Project visible हो जाता है।

Remove:
  1. Project deployment metadata read होता है।
  2. Container/project compose state remove होता है।
  3. hosts entry हटती है।
  4. SSL files हटते हैं।
  5. Project list refresh होती है।

Safety:
  - Remove source files delete नहीं करता।
  - WSDD rollback best-effort करता है जब deploy बीच में fail हो।
  - Retry safe होना चाहिए जब previous deployment partial state में रह गया हो।

## WSDD Settings

Tools → Settings अब केवल भाषा बदलता है। Save सार्वजनिक WSDD कॉन्फ़िगरेशन सहेजता है; Docker, WSL और संग्रहीत secrets को नहीं बदलता।

Tools → ngrok, Redis, Memcached या Mailpit में प्रत्येक सेवा का अलग डायलॉग है। Save विकल्प सहेजता है और deploy नहीं करता; Deploy चुनी हुई सेवा लागू करता है; Stop वॉल्यूम मिटाए बिना कंटेनर रोकता है; Cancel बिना सहेजे बदलाव छोड़ता है। ngrok Agent API जांच सकता है। Redis, Memcached और Mailpit का वास्तविक इंस्टॉलेशन पर पूर्ण परीक्षण अभी बाकी है।

WSDD स्टोरेज और रिकवरी विकल्प Tools → WSL Settings में अलग Save बटन के साथ हैं। पुराने PHP/Xdebug मान और MySQL/Webmin credentials सुरक्षित हैं, लेकिन Settings में संपादित नहीं होते। अनुपस्थित credentials संबंधित पहले deploy पर मांगे जाते हैं। संग्रहीत credentials बदलने से मौजूदा कंटेनर के उपयोगकर्ता नहीं बदलते।

सार्वजनिक कॉन्फ़िगरेशन: C:\WSDD-Environment\wsdd-config.json। Secrets: C:\WSDD-Environment\wsdd-secrets.json।

## WSL2 Settings

Access: Tools → WSL Settings

WSDD स्टोरेज के लिए अलग Save बटन project path, distro और managed storage को `wsdd-config.json` में सहेजता है। वैश्विक WSL Save केवल बदलाव होने पर `%USERPROFILE%\.wslconfig` लिखता है।

Options:
  - CPU cores
  - Max RAM
  - Swap
  - Memory reclaim
  - WSLg GUI applications
  - Localhost forwarding
  - Network mode

Recommendations:
  - Typical WSDD stacks: 4-8 GB RAM.
  - CPU: 50-70% physical cores.
  - Swap: 0 if RAM is enough.
  - NAT network mode for compatibility.

Important:
  - `.wslconfig` changes require WSL restart.
  - Run PowerShell as Admin: `wsl --shutdown`
  - Reopen Docker Desktop after shutdown.

## SSL certificates and HTTPS

WSDD mkcert से locally trusted SSL certificates generate करता है।

How it works:
  1. mkcert local Certificate Authority create करता है।
  2. CA Windows Certificate Store में trusted install होती है।
  3. Project SSL enabled होने पर domain certificate generate होता है।
  4. Reverse proxy HTTPS serve करता है।

Domains:
  - Projects `.wsdd.dock` suffix use करते हैं।
  - Example: `myapp.wsdd.dock` → `https://myapp.wsdd.dock`

Renew certificate:
  - Project Remove करें।
  - Deploy फिर से करें।

Browser trust:
  - `mkcert -version` verify करें।
  - `mkcert -install` run करें।
  - Browser restart करें।

Certificate location:
  - `C:\WSDD-Environment\Docker-Structure\ssl\`
  - `{domain}.crt`
  - `{domain}.key`

phpMyAdmin and MySQL SSL:
  - Browser → phpMyAdmin traffic HTTPS से secure होता है।
  - phpMyAdmin → MySQL internal connection MySQL TLS नहीं मानना चाहिए।
  - WSDD default से MySQL TLS force नहीं करता क्योंकि frameworks/ORMs को CA paths, `ssl-mode` और certificates चाहिए हो सकते हैं।
  - MySQL TLS project-level hardening option है, default local stack behavior नहीं।

## Troubleshooting

Docker not running:
  - Docker Desktop खोलें।
  - WSL2 backend ready होने तक wait करें।
  - WSDD में refresh करें।

Port conflict:
  - Ports 80/443 किसी और service ने use किए हों तो proxy start fail हो सकता है।
  - IIS/Apache/Nginx local services stop करें।

Domain not resolving:
  - WSDD as Administrator चलाएँ।
  - hosts file check करें।
  - DNS/browser cache clear करें।

SSL warning:
  - mkcert CA reinstall करें।
  - Browser restart करें।

Xdebug not connecting:
  - IDE listener port `9003` पर running हो।
  - Trigger use करें: `XDEBUG_TRIGGER=WSDD`.
  - Path mapping `/var/www/html/{project-domain}` से match करे।

Optional service missing:
  - Tools में संबंधित सेवा का डायलॉग खोलें।
  - Deploy दबाएँ।
  - Container panel refresh करें।

## Frequently Asked Questions (FAQ)

Q: क्या WSDD source code files modify करता है?
A: WSDD deployment metadata, Docker config, hosts entries और SSL files manage करता है. Project source files delete नहीं करता।

Q: क्या Docker Desktop automatic install होता है?
A: नहीं. User को Docker Desktop install करना चाहिए. WSDD verify/configuration flow संभालता है।

Q: क्या optional services base stack के साथ start होते हैं?
A: नहीं. Redis, Memcached और Mailpit default से disabled हैं; इनके Tools डायलॉग में Deploy करने पर ही सेवा लागू होती है।

Q: क्या WSDD MySQL TLS force करता है?
A: नहीं. phpMyAdmin HTTPS browser traffic secure करता है, लेकिन internal MySQL TLS project-level hardening है।

Q: Logs कहाँ हैं?
A: App UI में general log और container live log panels हैं. Detailed Rust logs के लिए `RUST_LOG=wsdd=debug` use करें।

Q: Kya ngrok sirf browser login se chalega?
A: Nahin. Browser login aapka ngrok account kholta hai, lekin Docker agent ko
   authtoken chahiye. Tools → ngrok me dashboard se token lekar Save karein,
   phir Deploy dabayein. WSDD ise wsdd-secrets.json me rakhta hai.

Q: Kaunsa project storage mode chunna chahiye?
A: Classic Bind current Windows bind-mount workflow rakhta hai. Fast Sync
   Windows ko source rakhta hai aur Docker-native volume se run karta hai.
   Native Linux Project existing Linux/WSL paths ke liye hai.

## File and path reference

Main environment:
  - `C:\WSDD-Environment\`

Resources:
  - `C:\WSDD-Environment\PS-Script\`
  - `C:\WSDD-Environment\Docker-Structure\`

Configuration:
  - `C:\WSDD-Environment\wsdd-config.json`
  - `C:\WSDD-Environment\wsdd-secrets.json`

Docker structure:
  - `C:\WSDD-Environment\Docker-Structure\bin\`
  - `C:\WSDD-Environment\Docker-Structure\services\`
  - `C:\WSDD-Environment\Docker-Structure\ssl\`
  - `C:\WSDD-Environment\Docker-Structure\projects\`

Windows:
  - `C:\Windows\System32\drivers\etc\hosts`
  - `%USERPROFILE%\.wslconfig`
