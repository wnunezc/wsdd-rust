# WebStack Deployer for Docker (WSDD)

这是一个 Windows 桌面应用程序，用于自动化基于 Docker 的本地 Web 开发环境配置。
它包含多版本 PHP、本地 SSL、MySQL、phpMyAdmin 以及 hosts 管理功能。

*语言: [English](../../README.md) | [Español](README.es.md) | [Français](README.fr.md) | [हिन्दी](README.hi.md) | [中文](README.zh.md)*

*快速链接: [迁移地图](../../MIGRATION.md) | [许可](../legal/LICENSE.zh.md) | [主仓库](../../README.md) | [报告问题](https://github.com/wnunezc/wsdd-rust/issues/new)*

## 系统要求

- **操作系统**: Windows 10 / Windows 11
- **权限**: Administrator（必需）
- **Docker Desktop**: 如未安装则自动安装
- **WSL 2**: 自动配置
- **Chocolatey**: 如未安装则自动安装

## 此应用程序的功能

1. **检查并安装依赖项**: Docker Desktop、WSL 2、Chocolatey、MKCert
2. **配置 Docker 栈**: Nginx reverse proxy、MySQL、phpMyAdmin
3. **管理 Web 项目**: 为每个 PHP 版本创建带 Apache + Xdebug 的容器
4. **自动本地 SSL**: 为每个域名生成 MKCert 证书，无浏览器警告
5. **自动 hosts**: 自动修改 `C:\Windows\System32\drivers\etc\hosts`

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

## 磁盘环境结构

应用程序会创建并管理 `C:\WSDD-Environment\` 目录:

```
C:\WSDD-Environment\
├── PS-Script\          — PowerShell 自动化脚本
├── Docker-Structure\   — docker-compose 和 PHP 镜像
├── certs\              — 每个域名的 SSL 证书
└── wsdd-config.json    — 应用程序配置
```

## 首次启动 — 自动流程

1. 应用程序检查是否具有管理员权限
2. 将嵌入式资源解压到 `C:\WSDD-Environment\`
3. 检查 Chocolatey → 如缺失则安装
4. 检查 Docker Desktop → 如缺失则安装（需要重启）
5. 检查 MKCert → 安装并配置本地 CA
6. 启动基础 Docker 栈
7. 显示主面板

> **注意**: Docker Desktop 的安装可能需要系统重启。
> 重启后应用程序会自动继续运行。

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

- **Version**: 1.0.0-rc.3 (Rust edition)
- **GUI**: egui / eframe (immediate-mode)
- **Async**: tokio
- **Configuration**: `C:\WSDD-Environment\wsdd-config.json` 中的 JSON
- **Logs**: 使用环境变量 `RUST_LOG=wsdd=debug` 获取详细日志

## 许可

专有许可 — 详情请参阅 [LICENSE.zh.md](../legal/LICENSE.zh.md)。
Copyright (c) 2026 Walter Nunez / Icaros Net S.A. All Rights Reserved.
