# PushRight

**看清分支，推送到正确的位置。**

[English](https://github.com/ArthurLauCS/PushRight/blob/main/extension/README.md) · 简体中文

PushRight 在 VS Code 中提供 Git 源代码管理、Blame、历史和提交图。推送前明确目标，每次 Pull 默认勾选 **Rebase instead of merge**。包含中英文界面，扩展语言跟随 VS Code。

## 使用条件

- Windows 10 / 11 x64、VS Code 1.90 及以上、PATH 中可运行 Git。
- 已信任的本地 Git 工作区。本版不支持 Linux、macOS、WSL、SSH 或浏览器工作区。
- 远程认证沿用已有 Git 凭据助手或 SSH 配置。

## 推送与拉取

例如本地 `feature/login` 错误跟踪 `origin/main`，PushRight 会提示名称不一致，推荐 `origin/feature/login`，由你明确选择目标。推送使用指定的远程和分支 refspec；强制推送使用 `--force-with-lease` 并再次确认。

每次 Pull 都会显示选择框，默认勾选 **Rebase instead of merge**。团队要求线性拉取历史时保持勾选。不会自动贮藏本地改动，请先提交或贮藏；遇到冲突后可解决并继续，或中止。

## 已有能力

- 查看暂存、未暂存和冲突文件；按文件或选中行暂存、取消暂存、丢弃；提交和 Amend。
- 当前行 Blame、作者悬浮信息、CodeLens、整文件 Blame（`Alt+B`）；给单个作者设置底色和边框。
- 展开 **PushRight > 行历史** 后，点击一行或选中几行就自动更新历史，无需右键命令，也不会切走编辑器焦点。连续移动光标时合并查询，面板隐藏时不查询。
- 文件历史、前后版本对比（`Alt+,` / `Alt+.`），按消息、作者、文件、改动内容或提交 ID 搜索。
- **源代码管理**侧栏的改动列表下方显示 **PushRight 提交图**，关闭原生 Git 后仍可使用。每个工作区首次自动展开，之后保留你的布局选择。22px 紧凑行高与清晰的分支标签适配窄侧栏；下拉框可切换仓库，点击提交可打开完整提交图中的对应详情。
- 新增目录逐个列出未跟踪文件，可在暂存前查看每个文件，并分别暂存；仍遵守忽略规则。
- 运行 **PushRight: 打开提交图** 或点击侧栏提交图标题栏按钮，在编辑器页签中浏览分支、提交和差异。如果曾隐藏侧栏图，可在源代码管理的“视图”菜单中重新勾选 **PushRight 提交图**。
- 快速修改当前仓库的提交名和邮箱。

首次启用会说明关闭 VS Code 内置 Git 的影响，由你决定是否让推送和拉取统一经过 PushRight。依赖内置 Git 的扩展可能因此停止工作。与 GitLens 同时启用时，可关闭其中一个的 Blame 注释以免重复。

## 预览版限制

- 尚无 PR／Issue 和托管平台协作，不能替代 GitLens 或 GitHub Pull Requests 的这些功能。
- 启动时识别工作区文件夹中的仓库；更改工作区文件夹后需重新加载，不自动发现嵌套仓库。
- 每个文件历史列表最多 200 条。查看行历史前需提交或贮藏该文件的改动，确保选区行号与 HEAD 一致。
- 历史文本与补丁限 4 MiB；二进制版本不会当作文本展示，仍可按整个文件操作。
- 完整提交历史在后台加载。实测范围见[性能报告](https://github.com/ArthurLauCS/PushRight/blob/main/docs/PERFORMANCE.md)。

## VSIX 安装

在 [GitHub Releases](https://github.com/ArthurLauCS/PushRight/releases/tag/vscode-v0.2.3) 下载 Windows x64 VSIX，然后执行 VS Code 的“扩展：从 VSIX 安装”，或：

```sh
code --install-extension pushright-win32-x64-0.2.3.vsix
```

反馈问题请提交到 [GitHub Issues](https://github.com/ArthurLauCS/PushRight/issues)，附版本和复现步骤；分享日志前移除凭据和私有仓库信息。

第三方声明随包提供，位于 `THIRD_PARTY_NOTICES.txt`。作者尚未为 PushRight 选择开源许可证。
