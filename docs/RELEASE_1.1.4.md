# PushRight 1.1.4 — Smoother refreshes and quick-diff fix

桌面版与 VS Code / Cursor 扩展同步更新。

## 修复与改进

- 修复多个仓库同时打开时 quick-diff 显示/隐藏命令重复注册的问题；各仓库仍通过根目录区分。
- 普通工作区文件变化只重读状态，状态不变时不触发视图刷新；Git 元数据变化仍完整刷新，连续事件合并处理。
- 并行读取互不依赖的 Git 信息，提交图加载跳过不需要的领先/落后计数。
- 同一仓库的提交图重载时保留已有图和行，减少清空或回退首屏造成的闪烁；紧凑图显示提交作者。
- 历史文件的二进制识别只检查前 8000 字节的 NUL，允许后续包含 NUL 的源码按文本打开。

## Downloads

- `PushRight_1.1.4_x64-setup.exe` — Windows x64 desktop installer.
- `pushright-win32-x64-1.1.4.vsix` — VS Code / Cursor extension.
- `PushRight-1.1.4-source.zip` — Source code from the release tag, without dependencies or build outputs.
- `SHA256SUMS.txt` — SHA-256 checksums for the installer, VSIX and source archive.

## Validation and scope

- Passed 23 Node tests, 30 Rust workspace tests, and Svelte/TypeScript checks (zero errors or warnings).
- The packaged VSIX passed all 23 smoke checks in both VS Code and Cursor, including multi-repository discovery and closing/reopening repositories. Neither log contains the duplicate quick-diff visibility command registration error seen in 1.1.3.
- Regression checks cover quiet file-only refreshes, queued full refreshes, shared quick-diff menu labels, and NUL detection beyond byte 8000.
- The existing `pull_explicit_mode_and_conflict_recovery` Rust test is excluded because it sets `pull.rebase=false` and executes a merge pull, contrary to the required rebase-only Git policy.
- The desktop executable passed a process-startup check. Full installer and manual UI acceptance tests were not performed; no performance benchmark was run.
- Windows 10/11 x64 local workspaces only. System Git is required; desktop requires WebView2. The installer is unsigned.
