# WebStack Deployer for Docker (WSDD)

这是一个 Windows 桌面应用程序，用于自动化基于 Docker 的本地 Web 开发环境配置。
它包含多版本 PHP、本地 SSL、MySQL、phpMyAdmin、hosts 管理、Xdebug，以及可选 Redis/Memcached/Mailpit 服务。

*语言: [English](../../README.md) | [Español](README.es.md) | [Français](README.fr.md) | [हिन्दी](README.hi.md) | [中文](README.zh.md)*

*快速链接: [用户指南](../help/user-guide.zh.md) | [迁移地图](../../MIGRATION.md) | [许可](../legal/LICENSE.zh.md) | [主仓库](../../README.md) | [报告问题](https://github.com/wnunezc/wsdd-rust/issues/new)*

*Language fallback: English for any missing localized UI/help content.*

## 系统要求

- **操作系统**: Windows 10 / Windows 11
- **权限**: Administrator（必需）
- **Docker Desktop**: 首次启动前必须由用户安装
- **WSL 2**: Docker Desktop 需要
- **Chocolatey**: 如未安装则自动安装
- **PowerShell**: 7.5+（如缺失则自动安装/更新）

## 此应用程序的功能

1. **检查并准备依赖项**: Chocolatey、PowerShell 7.5+、Docker Desktop、MKCert
2. **配置 Docker 栈**: Nginx reverse proxy、MySQL、phpMyAdmin
3. **管理 Web 项目**: 为每个 PHP 版本创建带 Apache + Xdebug 的容器
4. **自动本地 SSL**: 为每个域名生成 MKCert 证书，无浏览器警告
5. **自动 hosts**: 自动修改 `C:\Windows\System32\drivers\etc\hosts`
6. **可选服务**: Redis、Memcached 和 Mailpit 默认关闭，只在 Settings 中启用并保存后部署

## Docker 栈容器

### 基础服务（始终启用）
- **WSDD-Proxy-Server** — Nginx reverse proxy（端口 80 / 443）
- **WSDD-MySql-Server** — MySQL 8（端口 3306）
- **WSDD-phpMyAdmin-Server** — phpMyAdmin

### PHP 容器（每个使用中的版本一个）
可用版本: 5.6 - 7.2 - 7.4 - 8.1 - 8.2 - 8.3 - 8.4

对于每个激活的版本，将创建以下开发 URL:
- `php{version}.wsdd.dock` — 主 PHP 环境
- `cron{version}.wsdd.dock` — Cron 作业管理器
- `wm{version}.wsdd.dock` — Webmin（服务器管理）

### 可选服务（默认关闭）
- **WSDD-Redis-Server** — Redis cache/queues/sessions (`redis:7.4.8-alpine`)
- **WSDD-Memcached-Server** — Memcached legacy cache (`memcached:1.6.39-alpine`)
- **WSDD-Mailpit-Server** — local SMTP capture and web UI (`axllent/mailpit:v1.29.7`)

可选服务使用 `Docker-Structure/services/` 下的独立 compose 文件、独立 Compose project，以及共享
`wsdd-network`。它们不会随基础 stack 一起部署。

## 磁盘环境结构

应用程序会创建并管理 `C:\WSDD-Environment\` 目录:

```
C:\WSDD-Environment\
├── PS-Script\          — PowerShell 自动化脚本
├── Docker-Structure\   — docker-compose、PHP 镜像、services 和 SSL 资源
├── wsdd-config.json    — 应用程序配置
└── wsdd-secrets.json   — 容器托管 secrets
```

## 首次启动 — 自动流程

1. 应用程序检查是否具有管理员权限
2. 将嵌入式资源解压到 `C:\WSDD-Environment\`
3. 检查 Chocolatey → 如缺失则安装
4. 检查 PowerShell 7.5+ → 如缺失则安装/更新
5. 检查 Docker Desktop → 如未安装/配置则停止
6. 检查 MKCert → 安装并配置本地 CA
7. 启动基础 Docker 栈
8. 显示主面板

## 首次启动后的使用

### 添加项目
1. 点击 “Add Project”
2. 选择本地域名（例如 `myproject.wsdd.dock`）
3. 选择 PHP 版本
4. 应用程序会创建容器、SSL 证书和 hosts 条目

### 管理容器
- 从主面板启动 / 停止单个容器
- 一键打开实时日志
- 从菜单重启整个栈

## 技术信息

- **Version**: 1.0.0 (Rust edition)
- **GUI**: egui / eframe (immediate-mode)
- **Async**: tokio
- **Configuration**: `C:\WSDD-Environment\wsdd-config.json` 中的 JSON
- **Secrets**: `C:\WSDD-Environment\wsdd-secrets.json` 中的 JSON
- **Logs**: 使用环境变量 `RUST_LOG=wsdd=debug` 获取详细日志

## 许可

专有许可 — 详情请参阅 [LICENSE.zh.md](../legal/LICENSE.zh.md)。
Copyright (c) 2026 Walter Nunez / Icaros Net S.A. All Rights Reserved.
