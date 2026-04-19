# WebStack Deployer for Docker (WSDD)

Windows डेस्कटॉप एप्लिकेशन जो Docker का उपयोग करके स्थानीय वेब डेवलपमेंट वातावरण की सेटअप प्रक्रिया को स्वचालित बनाता है।
इसमें बहु-संस्करण PHP, लोकल SSL, MySQL, phpMyAdmin, hosts प्रबंधन, Xdebug और वैकल्पिक Redis/Memcached/Mailpit सेवाएँ शामिल हैं।

*भाषाएँ: [English](../../README.md) | [Español](README.es.md) | [Français](README.fr.md) | [हिन्दी](README.hi.md) | [中文](README.zh.md)*

*त्वरित लिंक: [User Guide](../help/user-guide.hi.md) | [माइग्रेशन मैप](../../MIGRATION.md) | [लाइसेंस](../legal/LICENSE.hi.md) | [Notice](../../NOTICE.md) | [Third-Party Licenses](../../THIRD_PARTY_LICENSES.md) | [Contributing](../../CONTRIBUTING.md) | [Security](../../SECURITY.md) | [Discussions](https://github.com/wnunezc/wsdd-rust/discussions) | [बग रिपोर्ट करें](https://github.com/wnunezc/wsdd-rust/issues/new)*

*Language fallback: English for any missing localized UI/help content.*

## WSDD के बारे में

WSDD Windows-first local stack manager है, PHP + Docker development के लिए।
यह first-run environment setup automate करता है, per-project PHP containers बनाता है,
MKCert के साथ local SSL configure करता है, Windows `hosts` file update करता है और
container/project operations को एक desktop app में centralize करता है।

- **Current stage**: Stable `1.0.0` release
- **Primary distribution package**: Windows MSI installer
- **Current UI languages**: English, Spanish, French, Hindi, Chinese
- **Language fallback**: English for missing localized UI/help content
- **Issue reporting**: [GitHub Issues](https://github.com/wnunezc/wsdd-rust/issues/new)

## सिस्टम आवश्यकताएँ

- **ऑपरेटिंग सिस्टम**: Windows 10 / Windows 11
- **अधिकार**: Administrator (अनिवार्य)
- **Docker Desktop**: पहले लॉन्च से पहले उपयोगकर्ता द्वारा इंस्टॉल होना चाहिए
- **WSL 2**: Docker Desktop के लिए आवश्यक
- **Chocolatey**: मौजूद न होने पर स्वतः इंस्टॉल होता है
- **PowerShell**: 7.5+ (मौजूद न होने पर स्वतः इंस्टॉल/अपडेट होता है)

## यह एप्लिकेशन क्या करती है

1. **निर्भरताओं की जाँच और तैयारी करती है**: Chocolatey, PowerShell 7.5+, Docker Desktop, MKCert
2. **Docker स्टैक कॉन्फ़िगर करती है**: Nginx reverse proxy, MySQL, phpMyAdmin
3. **वेब प्रोजेक्ट प्रबंधित करती है**: Apache + Xdebug के साथ प्रति संस्करण PHP कंटेनर बनाती है
4. **स्वचालित लोकल SSL**: प्रति डोमेन MKCert प्रमाणपत्र, बिना ब्राउज़र चेतावनी
5. **स्वचालित hosts**: आपके लिए `C:\Windows\System32\drivers\etc\hosts` संशोधित करती है
6. **वैकल्पिक सेवाएँ**: Redis, Memcached और Mailpit डिफ़ॉल्ट रूप से बंद रहते हैं और Settings में सक्रिय करने पर ही deploy होते हैं

## Docker स्टैक कंटेनर

### बेस सेवाएँ (हमेशा सक्रिय)
- **WSDD-Proxy-Server** — Nginx reverse proxy (ports 80 / 443)
- **WSDD-MySql-Server** — MySQL 8 (port 3306)
- **WSDD-phpMyAdmin-Server** — phpMyAdmin

### PHP कंटेनर (प्रत्येक उपयोग किए गए संस्करण के लिए एक)
उपलब्ध संस्करण: 5.6 - 7.2 - 7.4 - 8.1 - 8.2 - 8.3 - 8.4

प्रत्येक सक्रिय संस्करण के लिए निम्न विकास URL बनाए जाते हैं:
- `php{version}.wsdd.dock` — मुख्य PHP वातावरण
- `cron{version}.wsdd.dock` — Cron jobs प्रबंधक
- `wm{version}.wsdd.dock` — Webmin (सर्वर प्रशासन)

### वैकल्पिक सेवाएँ (डिफ़ॉल्ट रूप से बंद)
- **WSDD-Redis-Server** — cache, queues और sessions के लिए Redis (`redis:7.4.8-alpine`)
- **WSDD-Memcached-Server** — legacy cache के लिए Memcached (`memcached:1.6.39-alpine`)
- **WSDD-Mailpit-Server** — local SMTP capture और web UI (`axllent/mailpit:v1.29.7`)

वैकल्पिक सेवाएँ `Docker-Structure/services/` में अलग compose files, अलग Compose projects और shared
`wsdd-network` का उपयोग करती हैं। ये base stack के साथ deploy नहीं होतीं।

## डिस्क पर वातावरण की संरचना

एप्लिकेशन `C:\WSDD-Environment\` डायरेक्टरी बनाती और प्रबंधित करती है:

```
C:\WSDD-Environment\
├── PS-Script\          — PowerShell ऑटोमेशन स्क्रिप्ट्स
├── Docker-Structure\   — docker-compose, PHP इमेजेज, services और SSL assets
├── wsdd-config.json    — एप्लिकेशन कॉन्फ़िगरेशन
└── wsdd-secrets.json   — कंटेनरों के लिए प्रबंधित secrets
```

## पहला लॉन्च — स्वचालित प्रक्रिया

1. एप्लिकेशन जाँचती है कि उसके पास administrator privileges हैं
2. एम्बेडेड resources को `C:\WSDD-Environment\` में extract करती है
3. Chocolatey की जाँच करती है → न होने पर इंस्टॉल करती है
4. PowerShell 7.5+ की जाँच करती है → न होने पर इंस्टॉल/अपडेट करती है
5. Docker Desktop की जाँच करती है → न होने/configure न होने पर रोकती है
6. MKCert की जाँच करती है → install और local CA configure करती है
7. बेस Docker स्टैक शुरू करती है
8. मुख्य पैनल दिखाती है

## पहले लॉन्च के बाद उपयोग

### प्रोजेक्ट जोड़ना
1. "Add Project" पर क्लिक करें
2. लोकल डोमेन चुनें (उदाहरण: `myproject.wsdd.dock`)
3. PHP संस्करण चुनें
4. एप्लिकेशन कंटेनर, SSL प्रमाणपत्र और hosts entry बना देती है

### कंटेनर प्रबंधन
- मुख्य पैनल से व्यक्तिगत कंटेनर शुरू / बंद करें
- एक क्लिक में real-time logs खोलें
- मेनू से पूरी स्टैक restart करें

## तकनीकी जानकारी

- **Version**: 1.0.0 (Rust edition)
- **GUI**: egui / eframe (immediate-mode)
- **Async**: tokio
- **Configuration**: `C:\WSDD-Environment\wsdd-config.json` में JSON
- **Secrets**: `C:\WSDD-Environment\wsdd-secrets.json` में JSON
- **Logs**: विस्तृत logs के लिए environment variable `RUST_LOG=wsdd=debug`

## लाइसेंस

स्वामित्वाधीन — विवरण के लिए [LICENSE.hi.md](../legal/LICENSE.hi.md) देखें।
Copyright (c) 2026 Walter Nunez / Icaros Net S.A. All Rights Reserved.
