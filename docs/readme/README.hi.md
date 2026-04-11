# WebStack Deployer for Docker (WSDD)

Windows डेस्कटॉप एप्लिकेशन जो Docker का उपयोग करके स्थानीय वेब डेवलपमेंट वातावरण की सेटअप प्रक्रिया को स्वचालित बनाता है।
इसमें बहु-संस्करण PHP, लोकल SSL, MySQL, phpMyAdmin और hosts प्रबंधन शामिल है।

*भाषाएँ: [English](../../README.md) | [Español](README.es.md) | [Français](README.fr.md) | [हिन्दी](README.hi.md) | [中文](README.zh.md)*

*त्वरित लिंक: [माइग्रेशन मैप](../../MIGRATION.md) | [लाइसेंस](../legal/LICENSE.hi.md) | [मुख्य रिपॉजिटरी](../../README.md) | [बग रिपोर्ट करें](https://github.com/wnunezc/wsdd-rust/issues/new)*

## सिस्टम आवश्यकताएँ

- **ऑपरेटिंग सिस्टम**: Windows 10 / Windows 11
- **अधिकार**: Administrator (अनिवार्य)
- **Docker Desktop**: मौजूद न होने पर स्वतः इंस्टॉल होता है
- **WSL 2**: स्वतः कॉन्फ़िगर होता है
- **Chocolatey**: मौजूद न होने पर स्वतः इंस्टॉल होता है

## यह एप्लिकेशन क्या करती है

1. **निर्भरताओं की जाँच और स्थापना करती है**: Docker Desktop, WSL 2, Chocolatey, MKCert
2. **Docker स्टैक कॉन्फ़िगर करती है**: Nginx reverse proxy, MySQL, phpMyAdmin
3. **वेब प्रोजेक्ट प्रबंधित करती है**: Apache + Xdebug के साथ प्रति संस्करण PHP कंटेनर बनाती है
4. **स्वचालित लोकल SSL**: प्रति डोमेन MKCert प्रमाणपत्र, बिना ब्राउज़र चेतावनी
5. **स्वचालित hosts**: आपके लिए `C:\Windows\System32\drivers\etc\hosts` संशोधित करती है

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

## डिस्क पर वातावरण की संरचना

एप्लिकेशन `C:\WSDD-Environment\` डायरेक्टरी बनाती और प्रबंधित करती है:

```
C:\WSDD-Environment\
├── PS-Script\          — PowerShell ऑटोमेशन स्क्रिप्ट्स
├── Docker-Structure\   — docker-compose और PHP इमेजेज
├── certs\              — प्रति डोमेन SSL प्रमाणपत्र
└── wsdd-config.json    — एप्लिकेशन कॉन्फ़िगरेशन
```

## पहला लॉन्च — स्वचालित प्रक्रिया

1. एप्लिकेशन जाँचती है कि उसके पास administrator privileges हैं
2. एम्बेडेड resources को `C:\WSDD-Environment\` में extract करती है
3. Chocolatey की जाँच करती है → न होने पर इंस्टॉल करती है
4. Docker Desktop की जाँच करती है → न होने पर इंस्टॉल करती है (restart आवश्यक)
5. MKCert की जाँच करती है → install और local CA configure करती है
6. बेस Docker स्टैक शुरू करती है
7. मुख्य पैनल दिखाती है

> **नोट**: Docker Desktop की स्थापना के लिए सिस्टम restart की आवश्यकता हो सकती है।
> restart के बाद एप्लिकेशन स्वतः फिर से शुरू हो जाएगी।

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

- **Version**: 1.0.0-rc.5 (Rust edition)
- **GUI**: egui / eframe (immediate-mode)
- **Async**: tokio
- **Configuration**: `C:\WSDD-Environment\wsdd-config.json` में JSON
- **Logs**: विस्तृत logs के लिए environment variable `RUST_LOG=wsdd=debug`

## लाइसेंस

स्वामित्वाधीन — विवरण के लिए [LICENSE.hi.md](../legal/LICENSE.hi.md) देखें।
Copyright (c) 2026 Walter Nunez / Icaros Net S.A. All Rights Reserved.
