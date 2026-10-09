# PushRight 1.1.3 — Stable branch clicks

桌面版与 VS Code / Cursor 扩展同步更新。

## 修复与改进

- 分支栏不再按最近使用重排，单击后条目保持原位，避免连续点击或双击时目标移动。
- 单击分支、标签或贮藏后高亮选中项，当前检出的分支以对钩单独标记。
- 扩展分支选择框保持本地分支优先、组内顺序固定；提交图范围选择框中的当前分支带对钩。
- 保留单击查看提交或贮藏、双击执行原有操作，以及分支最新提交预览。

## Downloads

- `PushRight_1.1.3_x64-setup.exe` — Windows x64 desktop installer.
- `pushright-win32-x64-1.1.3.vsix` — VS Code / Cursor extension.
- `PushRight-1.1.3-source.zip` — Source code from the release tag, without dependencies or build outputs.
- `SHA256SUMS.txt` — SHA-256 checksums for the installer, VSIX and source archive.

## Validation and scope

- Passed 22 Node tests, 31 Rust workspace tests, and Svelte/TypeScript checks (zero errors or warnings).
- Added a regression assertion that changing the checked-out branch preserves picker order and moves only the tick.
- The packaged VSIX passed all 23 existing smoke checks in both VS Code and Cursor. The desktop executable passed a process-startup check; full installer and manual click testing were not performed.
- The VS Code multi-repository smoke run still logged duplicate quick-diff visibility command registrations, as in 1.1.2; all smoke assertions passed. This existing diagnostic is outside this branch-click fix.
- Windows 10/11 x64 local workspaces only. System Git is required; desktop requires WebView2. The installer is unsigned.
