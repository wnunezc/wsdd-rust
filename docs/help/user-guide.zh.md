# WSDD Help

<!-- Canonical source for the in-app help rendered by src/ui/helps.rs. -->

## 系统要求

操作系统:
  - Windows 10 (build 2004+) 或 Windows 11
  - x64 架构 (AMD64)
  - 最低 8 GB RAM — PHP 堆栈推荐 16 GB
  - 20 GB 可用磁盘空间

权限:
  - WSDD 必须以管理员身份运行 (需要 UAC)
  - 需要修改 C:\Windows\System32\drivers\etc\hosts

WSDD 自动安装的软件:
  - Chocolatey — Windows 包管理器
  - Docker Desktop — 容器引擎
  - mkcert — 本地 SSL 证书生成
  - WSL 2 — Linux 子系统 (Docker Desktop 需要)

必须预先安装的软件:
  - PowerShell 7 (pwsh.exe) — 自动化必需

## 安装和首次启动

1. 以管理员身份运行 wsdd.exe (右键 → 以管理员身份运行)。

2. 首次启动时会出现欢迎向导:
   - 阅读并勾选要求复选框。
   - 点击「下一步」开始环境验证。

3. 在首次部署基础环境之前，WSDD 会打开一个对话框来获取
   MySQL 和 phpMyAdmin 凭据。
   - 这些凭据会保存在 wsdd-config.json 中。
   - 只有在配置中缺失时才会请求。

4. 加载器自动验证和安装:
   - Chocolatey (包管理器)
   - Docker Desktop
   - mkcert (本地 SSL)
   - 网络配置和根证书

4. 如果 Docker Desktop 未安装:
   - WSDD 通过 Chocolatey 安装它。
   - 可能需要重启系统。
   - 重启后再次运行 WSDD。

5. 加载器完成后，显示主面板。

WSDD 数据位置:
  C:\WSDD-Environment\
  ├── PS-Script\         PowerShell 自动化脚本
  ├── Docker-Structure\  容器、项目和 SSL 配置
  └── wsdd-config.json  应用程序设置

## 主面板 — 概览

主面板有三个区域:

1. 菜单栏 (顶部):
   - 文件 → 添加项目, 退出
   - Docker → 刷新列表, 重新加载 Docker Desktop, 清除日志
   - 工具 → WSL 设置, 设置
   - 帮助 → 帮助, 关于...

2. 工具栏 (菜单下方):
   - ⬡ phpMyAdmin   — 在浏览器中打开 phpMyAdmin
   - ⚡ PS 终端      — 在 C:\WSDD-Environment 中打开 PowerShell 7
   - ⬛ CMD 终端    — 在 C:\WSDD-Environment 中打开 CMD
   - + 添加         — 新项目表单
   - ↺ 刷新         — 更新容器和项目
   - 主题选择器 (右侧) — 切换主题

3. 中央选项卡:
   - 容器   — 活动/非活动 Docker 容器列表
   - 项目   — 已注册的 WSDD 项目列表

4. 日志面板 (底部):
   - 显示操作历史，带颜色编码。
   - 「复制」按钮 — 复制日志到剪贴板。
   - 「清除」按钮 — 清除可见日志。

## Docker 容器管理

「容器」选项卡显示所有 WSDD 容器及其当前状态。

列:
  - 名称       — Docker 容器名称
  - 状态       — Running / Exited / ...
  - 镜像       — 基础 Docker 镜像
  - 工具箱 (⚙)  — 高级容器操作

每个容器的操作:
  - ▶ 启动   — 启动容器
  - ■ 停止   — 停止容器
  - ↺ 重启   — 重启容器

容器工具箱 (⚙ 按钮):
  - 实时查看容器日志
  - 在容器内打开交互式 TTY 终端
  - 查看暴露的 URL 和端口
  - 详细信息 (镜像, 端口, 卷)

自动轮询:
  容器每 3 秒自动更新。
  无需手动刷新。

## 项目管理

「项目」选项卡显示 WSDD 中注册的 Web 项目。

列:
  - 名称       — 项目名称
  - 域名       — 本地域名 (例如: myapp.wsdd.dock)
  - PHP        — 分配的 PHP 版本 (5.6 — 8.4)
  - 状态       — Deployed / Not Deployed
  - 操作       — Deploy, Remove, Toolbox

操作:
  - ⬆ Deploy    — 部署项目 (创建容器, SSL, hosts)
  - ⬇ Remove    — 移除部署 (不删除源文件)
  - ⚙ Toolbox   — 高级项目操作

项目工具箱:
  - 在 Windows 资源管理器中打开项目文件夹
  - 在浏览器中打开项目
  - 查看详细信息 (路径, 域名, 入口点)

删除项目:
  - 点击 Remove 会提示确认。
  - 删除内容: 容器, SSL, hosts 条目, JSON 记录。
  - 源代码文件不会被删除。

## 添加项目

添加新 Web 项目:

1. 点击工具栏中的「+ 添加」或转到 文件 → 添加项目。

2. 填写表单:

   名称:
     项目标识符 (无空格, 仅字母/数字/连字符)。
     示例: my-project

   域名:
     本地子域名。自动添加 '.wsdd.dock' 后缀。
     示例: 输入 'myapp' → 最终域名: myapp.wsdd.dock

   PHP 版本:
     从 PHP 5.6, 7.2, 7.4, 8.1, 8.2, 8.3, 8.4 中选择。

   工作目录:
     本地磁盘上的项目根目录。
     使用「浏览...」按钮选择文件夹。

   入口点:
     应用程序主文件。
     选项: index.php, index.html, index.htm, 自定义。

   SSL:
     使用 mkcert 生成本地 SSL 证书的复选框。
     推荐: 启用。自动生成 HTTPS。

3. 点击「Deploy」创建项目。

## Deploy 和 Remove — 详细流程

DEPLOY 流程:
  1. 保存项目到 JSON
  2. 创建包含项目代码的 Docker 卷
  3. 在 options.php{XX}.yml 中注册项目
  4. 停止并删除之前的 PHP 容器
  5. 重建并启动 PHP 容器
  6. 生成 vhost.conf
  7. 生成 SSL 证书 (如果启用)
  8. 重启代理
  9. 更新 hosts 文件

REMOVE 流程:
  1. 从 options.yml 中移除项目
  2. 不含项目重建 PHP 容器
  3. 删除项目卷
  4. 移除项目 vhost.conf 块
  5. 删除项目 JSON

重要:
  - Remove 不会删除项目源代码。
  - 域名可能需要几秒钟才能停止解析。

## WSDD 设置

访问方式: 工具 → 设置

常规:
  - 项目路径    — 新项目的基础目录（默认: C:\WSDD-Projects）
  - Docker Desktop 路径  — Docker Desktop 可执行文件路径（可选，用于重新启动）
  - WSL 发行版           — 当前 WSL2 发行版（例如 Ubuntu-22.04）
  - 最大日志行数    — 日志面板保留的行数限制（100-10000）
  - 自动启动容器 — 应用启动时启动 WSDD 容器

PHP (Docker 容器):
  这些值在生成新容器时应用。
  它们不会影响已经存在的容器（需要 redeploy）。
  - memory_limit              — PHP RAM 限制
  - upload_max_filesize       — 最大上传文件大小
  - Timezone                  — PHP 时区
  - Xdebug                    — 对新的或重建的 PHP 容器默认启用。
    PHP 8.x 使用 Xdebug 3，模式为 debug,develop，host.docker.internal，
    端口 9003，并使用 trigger 启动。PHP 5.6/7.x 使用 Xdebug 2 的等效
    trigger 配置，host/port 相同。

IDE / agent 调试:
  - 在 VS Code、PHPStorm 或其他 DBGp listener 中监听端口 9003。
  - 将 Windows 项目路径映射到 /var/www/html/{project-domain}。
  - AI agents 也可以监听，只要它们在 Windows host 上运行兼容 DBGp/Xdebug
    的 listener；WSDD 只负责配置 PHP 容器回连。

可选服务:
  Redis、Memcached 和 Mailpit 默认关闭，不会随基础 stack 部署。
  在 Settings 中启用服务，检查端口/auto-start 选项，然后保存以部署。
  - Redis: 容器 host redis / WSDD-Redis-Server，内部端口 6379，
    默认 host 端口 6379，持久卷 wsdd-redis-data。
  - Memcached: 容器 host memcached / WSDD-Memcached-Server，内部端口
    11211，默认 host 端口 11211，易失缓存。
  - Mailpit: SMTP host mailpit / WSDD-Mailpit-Server，内部 SMTP 端口 1025，
    UI 端口 8025，默认本地 UI http://mailpit.wsdd.dock。
  - Framework 示例:
    Redis: REDIS_HOST=redis, REDIS_PORT=6379.
    Memcached: MEMCACHED_HOST=memcached, MEMCACHED_PORT=11211.
    Mailpit: MAIL_HOST=mailpit, MAIL_PORT=1025, MAIL_MAILER=smtp.

先决条件:
  - MySQL/phpMyAdmin 凭据 — 当配置中还不存在时，会在首次部署基础环境前请求。
  - 这些凭据会保存在 wsdd-secrets.json 中，并在后续启动时重复使用。

工具:
  - Webmin 版本 — PHP 容器中安装的版本（例如 2.630）
  - 按 PHP 版本保存的 Webmin 凭据 — 仅在该版本首次部署且容器尚不存在时请求一次。
  - 之后修改这些凭据不会自动轮换现有容器中的用户；它们会在下一次由 WSDD 管理的 rebuild 中生效。

更改保存到: C:\WSDD-Environment\wsdd-config.json
Secrets 保存到: C:\WSDD-Environment\wsdd-secrets.json

## WSL2 设置

访问方式: 工具 → WSL 设置

修改: %USERPROFILE%\.wslconfig

系统资源:
  - CPU 核心    — 限制分配给 WSL2 的核心数。
  - 最大 RAM    — 限制分配给 WSL2 的 RAM。
  - Swap        — 虚拟交换空间。

性能和内存:
  - 内存回收 — WSL2 如何将空闲 RAM 返回给 Windows 主机。
  - GUI 应用 (WSLg) — 支持带图形界面的 Linux 应用。

网络:
  - Localhost 转发 — 通过 127.0.0.1 访问 WSL2 端口。
  - 网络模式:
    - NAT (推荐) — 隔离的虚拟网络。
    - Mirrored — 共享主机网络。实验性。

重要说明:
  .wslconfig 的更改需要重启 WSL2:
  以管理员身份打开 PowerShell 并运行: wsl --shutdown

## SSL 证书和 HTTPS

WSDD 使用 mkcert 生成本地受信任的 SSL 证书。

工作原理:
  1. mkcert 在您的系统上创建本地证书颁发机构 (CA)。
  2. CA 作为受信任的安装在 Windows 证书存储中。
  3. 对于每个启用 SSL 的项目，生成由 CA 签名的证书。
  4. 反向代理使用证书提供 HTTPS。

域名:
  所有项目使用 .wsdd.dock 后缀
  示例: myapp.wsdd.dock → https://myapp.wsdd.dock

更新证书:
  Remove 项目然后重新 Deploy。
  证书自动重新生成。

证书位置:
  C:\WSDD-Environment\Docker-Structure\ssl\
  ├── {domain}.crt  — 证书
  └── {domain}.key  — 私钥

phpMyAdmin 和 MySQL SSL:
  通过 HTTPS 访问 phpMyAdmin 只保护浏览器 → phpMyAdmin 的流量。
  这不代表内部 phpMyAdmin → MySQL 连接使用 MySQL TLS。
  WSDD 默认不强制 MySQL TLS，因为现有 PHP frameworks 和 ORMs 可能需要
  CA 路径、ssl-mode 设置和证书。请把 MySQL TLS 视为按项目启用的可选加固，
  而不是本地 stack 的默认行为。

## 故障排除

问题: 容器未出现在列表中。
  解决方案:
  - 验证 Docker Desktop 正在运行。
  - 点击 ↺ 刷新。
  - 检查日志面板的错误消息。

问题: 启动 WSDD 时显示「Docker 未找到」。
  解决方案:
  - 从 docker.com 手动安装 Docker Desktop
  - 或让加载器安装它。
  - 安装后可能需要重启 Windows。

问题: .wsdd.dock 域名在浏览器中无法解析。
  解决方案:
  - 验证项目处于「Deployed」状态。
  - 检查 hosts 文件。
  - 如果没有: Remove 然后重新 Deploy。
  - 验证 WSDD 以管理员身份运行。

问题: HTTPS 显示证书错误。
  解决方案:
  - 在 PowerShell 中运行: mkcert -install
  - 完全重启浏览器。
  - 如果持续: Remove + Deploy 项目。

问题: Deploy 失败并显示 Docker 错误。
  解决方案:
  - 检查日志面板的具体错误。
  - 验证 Docker Desktop 处于「Running」状态。
  - 尝试菜单中的 Docker → 重新加载 Docker Desktop。

## 常见问题 (FAQ)

问: 我可以同时拥有多个 PHP 版本吗?
答: 是的。每个项目都有自己的 PHP 版本和容器。

问: WSDD 会修改我的源代码文件吗?
答: 不会。WSDD 只是将您的目录作为 Docker 卷挂载。
   文件不会在内部被复制或修改。

问: 如果我从 Windows 资源管理器删除项目会怎样?
答: Docker 容器和 WSDD 注册仍然存在。
   首先从 WSDD 执行「Remove」以正确清理。

问: 我可以将 WSDD 用于 Laravel / Symfony / WordPress 项目吗?
答: 是的，适用于任何 PHP 框架。正确配置入口点:
   - Laravel/Symfony: public/index.php
   - WordPress: index.php

问: 如何更新自动化脚本?
答: PS1 脚本嵌入在 WSDD 二进制文件中。
   要更新，需要从源代码重新编译。

问: WSDD 可以与 WSL 1 一起使用吗?
答: 不可以。Docker Desktop 需要 WSL 2。WSDD 假设使用 WSL 2。

## 文件和路径参考

```text
C:\WSDD-Environment\
├── wsdd-config.json              WSDD 应用程序设置
├── PS-Script\                   PowerShell 自动化脚本
├── Docker-Structure\
│   ├── bin\
│   │   └── php{X.X}\
│   │       └── options.php{XX}.yml   PHP 容器配置
│   └── projects\
│       └── {name}.json            每个注册项目的数据
│   ├── ssl\
│   │   ├── {domain}.crt          域名 SSL 证书
│   │   └── {domain}.key          SSL 私钥
```

其他托管路径:

- `%USERPROFILE%\.wslconfig` — WSL2 资源配置。
- `C:\Windows\System32\drivers\etc\hosts` — WSDD 在部署时修改的 hosts 文件。

WSDD 日志:

日志在会话期间保存在内存中。关闭 WSDD 前使用「复制」按钮保存日志。



