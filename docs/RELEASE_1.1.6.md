# PushRight 1.1.6 — Git command labels and repository revert settings

桌面版与 VS Code / Cursor 扩展同步更新中文操作标注，新增两端共用的仓库级禁用 revert 设置。

## 修复与改进

- 中文工具栏、菜单、弹窗选项和扩展命令面板显示对应的 Git 原生命令；纯界面操作不附加虚构的 Git 命令。
- 调整工具栏、菜单及工作区按钮布局，容纳更长的中文操作名称。
- 桌面版顶部和扩展源代码管理菜单新增“仓库设置”。启用“禁用 revert”后，执行层每次读取最新设置，阻止发起或继续 revert；仍允许中止已有的 revert。
- 设置保存在本地仓库配置 `pushright.disableRevert`，两端及关联工作树共用，其他仓库不受影响。默认允许 revert；此设置不限制终端或其他 Git 工具。
- 冒烟测试记录各阶段时间戳，便于定位宿主错误的触发阶段。

Chinese actions now include their native Git commands across toolbars, menus, dialogs and extension commands. Repository settings can disable starting and continuing revert in both apps, with enforcement in the shared engine. Abort remains available. The local setting is shared by linked worktrees, defaults to allowing revert, and does not restrict terminal Git or other tools.

## Downloads

- `PushRight_1.1.6_x64-setup.exe` — Windows x64 desktop installer.
- `pushright-win32-x64-1.1.6.vsix` — VS Code / Cursor extension.
- `PushRight-1.1.6-source.zip` — Source code from the release tag, without dependencies or build outputs.
- `SHA256SUMS.txt` — SHA-256 checksums for the installer, VSIX and source archive.

## Validation and scope

- 28 Node tests and Svelte/TypeScript checks passed.
- 31 Rust workspace tests passed. The existing `pull_explicit_mode_and_conflict_recovery` test was excluded because it sets `pull.rebase=false` and executes a merge pull, contrary to the required rebase-only policy.
- Packaged extension smoke tests cover 24 checks in both VS Code and Cursor, including persistence of the repository setting and execution-time rejection without changing HEAD.
- Regression tests cover linked worktrees, repository isolation, stale sessions, invalid configuration, and allowing abort while revert is disabled.
- Shared Chinese UI layouts were visually checked with test data. Full desktop installer acceptance was not performed.
- Cursor host logs include mutex, authentication, disposed-context, SCM tree and shutdown-storage errors. All five categories were reproduced without PushRight loaded, using Cursor's built-in Git. They remain host limitations; passing feature checks does not imply an error-free host run.
- Windows 10/11 x64 local workspaces only. System Git is required; desktop requires WebView2. The installer is unsigned.
