# GitGod

面向多分支日常工作的 Git 桌面可视化客户端，使用 Svelte、Tauri 和 Rust 构建。

当前版本为 **v0.1.0 预览版**，提供 Windows x64 安装包。

![GitGod 桌面版：提交图、独立作者样式与差异视图](docs/images/desktop.jpg)

截图来自安装后的桌面程序，使用虚构作者和本地示例仓库。

## 下载与安装

从 [GitHub Releases](https://github.com/ArthurLauCS/GitGod/releases) 下载 `GitGod_0.1.0_x64-setup.exe`，运行安装程序。

- 支持 Windows 10 / 11 x64；本次发布在 Windows x64 上构建和验证。
- 需要安装 [Git for Windows](https://git-scm.com/downloads/win)，并确保 `git` 在 PATH 中可用。
- 需要 Microsoft Edge WebView2 Runtime；缺少时安装程序会尝试联网安装。
- 安装包尚未进行代码签名，Windows 可能显示未知发布者提示。Release 附有 `SHA256SUMS.txt`，可用 PowerShell 的 `Get-FileHash .\GitGod_0.1.0_x64-setup.exe -Algorithm SHA256` 核对文件。

首次使用前，在 Git 中配置提交身份及远程认证：

```sh
git config --global user.name "Your Name"
git config --global user.email "you@example.com"
```

GitGod 使用系统 Git 的凭据管理器或 SSH 配置；遇到认证失败，先在终端完成相应仓库的 Git 认证。应用本身不提供账号登录或交互式终端提示。

## 使用

打开应用后选择本地 Git 仓库。也可将仓库路径作为启动参数传入：

```powershell
& "C:\path\to\gitgod.exe" "C:\path\to\repository"
```

### 提交与推送

1. 在工作区查看变更，按文件、差异块或选中行暂存，然后填写提交说明。
2. 推送时检查目标远程和分支。当前分支与 upstream 名称不一致时，应用要求明确选择，并优先推荐同名远程分支。
3. 推送使用明确的远程和分支 refspec。强制推送采用 `--force-with-lease`；仍需确认目标和预期影响。

例如本地 `feature/login` 误跟踪 `origin/main` 时，推送候选会优先推荐 `origin/feature/login`，避免直接沿用错误的 upstream。

### 单独设置作者样式

在提交列表中右键某个作者，独立设置该作者的底色和边框。设置按作者原始邮箱保存；没有邮箱时按姓名匹配。同名、不同邮箱的作者不会互相影响。设置跨仓库保存在本机。

左键作者可临时聚焦其提交；作者获得键盘焦点后，按 `Shift+F10` 也可打开样式设置。

### 已提供的功能

- 多仓库标签页、最近仓库与会话恢复。
- 虚拟滚动提交图、分支和标签标记、提交详情及文件差异。
- 暂存、取消暂存、提交、修改上次提交；按文件、差异块或选中行处理变更。
- 分支创建、切换、重命名与删除；合并、变基、cherry-pick、revert、reset 和标签操作。
- Fetch、Pull、明确选择目标的 Push，以及 stash 操作。
- Worktree 管理、冲突块处理、进行中操作的继续与中止。
- 深浅主题、可调面板、独立作者样式、操作说明和命令日志。

## 当前边界

- 这是桌面预览版。VS Code 扩展、行级 Blame、PR / Issue 和托管平台协作集成尚未交付。
- 提交列表只渲染可见行，但完整历史仍会在后台读取；超大仓库的时间和内存开销需要进一步实测优化。
- 超过 4 MiB 的补丁不显示完整差异；当前限制在读取之后判断，不能视为大文件读取的内存上限。
- 提交图最多显示 24 条轨道，实际数量随面板宽度变化；高度并行的复杂历史可能无法显示所有连线。
- 尚未提供自动更新，也未发布 macOS / Linux 安装包。
- 部分操作提供撤销入口，并非所有 Git 操作都能撤销；删除分支、丢弃变更、重置等操作前应检查提示。

## 本地开发

需要 Node.js 24、Rust stable，以及 Windows 的 MSVC C++ 工具链和 Windows SDK。完整环境要求见 [Tauri 官方文档](https://v2.tauri.app/start/prerequisites/)。

```sh
npm ci
npm run desktop:dev
```

`npm run dev` 只启动前端网页，不能独立调用桌面 Git 后端。

检查与构建：

```sh
npm run check
npm test
cargo test -p engine
npm run desktop:build
```

Windows 安装包输出到 `target/release/bundle/nsis/`。首次构建可能需要联网下载 Rust 依赖及 NSIS 打包工具。安装包构建方式见 [Tauri Windows Installer 文档](https://v2.tauri.app/distribute/windows-installer/)。

目录：

| 路径 | 职责 |
| --- | --- |
| `src/` | Svelte 界面、提交图、操作交互 |
| `crates/engine/` | Rust Git 引擎：gix 读取、Git CLI 操作 |
| `src-tauri/` | 桌面窗口、命令接口和安装包配置 |
| `tests/` | 前端回归检查 |

## 反馈与许可

请在 [Issues](https://github.com/ArthurLauCS/GitGod/issues) 中提交问题，附上版本、系统、复现步骤和必要日志；分享日志前请移除凭据及私密仓库信息。

第三方组件的许可声明见 [THIRD_PARTY_NOTICES.txt](THIRD_PARTY_NOTICES.txt)，安装目录也包含该文件。仓库公开不代表已授予本项目的开源使用许可；项目许可证尚待作者确定。
