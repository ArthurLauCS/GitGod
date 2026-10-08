# PushRight 1.1.1 — Branch and local-file workflow fixes

桌面版与 VS Code / Cursor 扩展同步更新。

## 修复与改进

- 桌面版按仓库记录最近查看的分支，并在各分支区内优先显示最近查看项。
- 扩展版的分支切换列表优先显示本地分支；本地和远程分支各自按最近使用排序。
- 扩展版支持一次选择多个未跟踪文件并批量设为“仅本机忽略”。
- 单击贮藏即可查看其中的文件与差异，无需先应用；包含 `git stash -u` 保存的未跟踪文件。
- 比较版本和交互式变基改用可点击的版本选择器，同时保留手工输入提交表达式。

## Downloads

- `PushRight_1.1.1_x64-setup.exe` — Windows x64 desktop installer.
- `pushright-win32-x64-1.1.1.vsix` — VS Code / Cursor extension.
- `SHA256SUMS.txt` — SHA-256 checksums for both packages.

## Validation and scope

- Node tests, Svelte/TypeScript checks, Rust workspace tests and packaged-extension smoke tests are run before publishing.
- Windows 10/11 x64 local workspaces only. System Git is required; desktop requires WebView2. The installer is unsigned.

MIT licensed. No Node.js or Rust installation is needed to use the packaged application or extension.
