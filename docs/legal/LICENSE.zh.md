# WebStack Deployer for Docker

**Version:** 1.0.0-rc.13
**Copyright:** © 2026 Walter Núñez / Icaros Net S.A.
**Jurisdiction:** Panama

## 1. 许可

本软件以受控开源形式分发。源代码可能公开可见，但其使用受本文件所定义限制的约束。

本软件仅供开发和技术测试使用。

## 2. 限制

以下行为被严格禁止：

- 未经作者书面授权的商业使用
- 在官方仓库之外重新分发本软件
- 在官方 GitHub 仓库之外修改代码
- 对二进制文件进行逆向工程
- 反编译
- 反汇编
- 删除版权声明

所有贡献或修改必须仅通过项目的官方仓库进行。

## 3. 分发

本软件只能从以下来源分发：

- 官方 GitHub 仓库
- 官方签名 releases
- 集成在启动器中的更新系统

不允许第三方重新分发。

## 4. 更新

软件启动器可能会自动从官方 GitHub 仓库下载更新。此行为属于软件正常运行的一部分，
不构成重新分发。

## 5. 保证

**本软件按“原样”提供，不附带任何明示或暗示的保证。**

包括但不限于：

- 适销性
- 特定用途适用性
- 持续运行
- 无错误保证

## 6. 责任限制

在任何情况下，作者均不对以下事项负责：

- 数据丢失
- 开发环境损坏
- Docker 容器导致的问题
- 错误配置
- WSL 故障
- 本地环境问题

用户有责任：

- 保持备份
- 为其项目维护版本控制
- 验证配置
- 检查应用程序生成的环境

## 7. 第三方依赖

本软件包含并使用第三方组件，这些组件受其各自许可证约束。

### 包含的字体与框架

- egui 0.29 / eframe
- JetBrains Mono v2.304 (OFL)
- Noto Sans Symbols 2 v2.008 (OFL)

### 开源依赖

- egui / eframe 0.29 — MIT / Apache 2.0
- tokio 1 — MIT
- serde / serde_json — MIT / Apache 2.0
- serde_yaml 0.9 — MIT / Apache 2.0
- quick-xml 0.36 — MIT
- anyhow / thiserror — MIT / Apache 2.0
- rfd 0.15 — MIT
- egui_commonmark 0.18 — MIT
- egui_extras 0.29 — MIT / Apache 2.0
- zip 2 — MIT
- walkdir 2 — MIT / Unlicense
- image 0.25 — MIT / Apache 2.0
- tracing — MIT
- windows 0.58 — MIT / Apache 2.0

### 必需的外部工具

- [Docker Desktop](https://www.docker.com)
- [WSL 2](https://learn.microsoft.com/windows/wsl)
- [Chocolatey](https://chocolatey.org)
- [mkcert](https://github.com/FiloSottile/mkcert)
- [PowerShell](https://github.com/PowerShell/PowerShell)

每个组件均保留其原始许可证。

## 8. 联系方式

Walter Núñez
Icaros Net S.A
Email: [wnunez@lh-2.net](mailto:wnunez@lh-2.net)

## 9. 接受

安装或使用本软件即表示用户接受本许可证中定义的条款。
