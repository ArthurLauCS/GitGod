# PushRight 1.1.9 — Diff colors and Windows blame

新增可自定义的差异配色，修复 Windows 文件路径大小写不一致造成的 Blame 缺失。

## Changes / 更新

- 桌面版顶部和扩展完整提交图新增“差异配色”。提供蓝／橙、紫／金、青／玫红三组预设，可分别选择新增、删除颜色或输入六位 HEX，并实时预览。偏好保存在本机；差异行保留 `+ / −` 标记和颜色边框辅助区分。
- 扩展命令面板与源代码管理菜单提供 **PushRight: 差异配色…**，同时应用于原生 VS Code / Cursor diff，包括 Review Branch。写入用户级 `workbench.colorCustomizations`；工作区的颜色设置可能优先覆盖。
- “恢复原配色”恢复此前的编辑器设置，保留无关配置和之后的手动修改。桌面版恢复为跟随应用主题的颜色。
- Windows 编辑器保留旧文件名大小写时，Blame 根据暂存区或正在查看的历史提交解析真实路径后重试，不依赖当前磁盘上的历史文件。遇到大小写重名不猜测，Git 错误写入 PushRight 输出面板。

Choose Blue / Orange, Purple / Gold, Cyan / Rose, or independent custom added/deleted colors. The shared full graph offers color pickers, HEX inputs and a live preview; diff lines retain signs and colored borders. The extension also applies the palette to native diff editors through user-level color customizations. Restoring colors preserves unrelated settings and later manual edits; workspace overrides may take precedence. Windows blame now resolves stale editor path casing against the index or the displayed historical revision, avoids ambiguous case collisions, and reports Git failures.

## Downloads

- `PushRight_1.1.9_x64-setup.exe` — Windows x64 desktop installer.
- `pushright-win32-x64-1.1.9.vsix` — VS Code / Cursor extension.
- `PushRight-1.1.9-source.zip` — Source code from the release tag.
- `SHA256SUMS.txt` — SHA-256 checksums for all three artifacts.

## Validation and scope

- Regression coverage includes preset/custom color validation, persistence, live preference synchronization, cancellation, failed settings writes, restoring user/theme colors and preserving manual edits; Windows blame tests cover index snapshots, historical renames/deletions, Unicode paths and case collisions.
- Browser checks cover preset and custom color application, preview, invalid input, cancellation, reload persistence, restoration, and light/dark themes.
- Release checks: Node tests, Svelte/TypeScript checks, Rust workspace tests, desktop build, and packaged VSIX smoke checks in VS Code and Cursor, including native color application/restoration and Windows blame.
- The Rust `pull_explicit_mode_and_conflict_recovery` test is excluded because it configures `pull.rebase=false` and executes a merge pull, contrary to the required rebase-only policy.
- Windows 10/11 x64 local workspaces only; system Git and WebView2 are required. The installer is unsigned. Full native installer acceptance and individual color-vision usability are not verified by these automated checks.
