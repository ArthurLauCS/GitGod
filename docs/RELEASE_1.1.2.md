# PushRight 1.1.2 — Branch picker commits and branch naming

桌面版与 VS Code / Cursor 扩展同步更新。

## 修复与改进

- 扩展版的分支切换列表在每个分支下显示最新提交：作者、短 ID、标题和提交时间，可按提交内容筛选。
- 扩展版提交图显示的分支改在顶部选择框中选择，列表与分支切换一致；筛选不会切换检出的分支。
- 新建分支时提示并校验命名规则 `fix|feat_S|C|SC_小驼峰`（如 `feat_S_userLogin`），并可选择从哪个分支签出。桌面版的新建分支和新建工作树对话框、扩展版分支切换列表里的“新建分支…”均适用。

## Downloads

- `PushRight_1.1.2_x64-setup.exe` — Windows x64 desktop installer.
- `pushright-win32-x64-1.1.2.vsix` — VS Code / Cursor extension.

## Validation and scope

- Node tests, Svelte/TypeScript checks and Rust workspace tests are run before packaging.
- Windows 10/11 x64 local workspaces only. System Git is required; desktop requires WebView2. The installer is unsigned.
