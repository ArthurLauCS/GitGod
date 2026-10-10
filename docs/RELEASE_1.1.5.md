# PushRight 1.1.5 — Blame in extension diffs

修复 VS Code / Cursor 扩展 diff 两侧的行末提交信息。桌面版同步版本号，本次没有桌面功能变更。

## 修复与改进

- 光标选中删除行时，显示左侧版本中该行的原提交；选中新增行时，显示右侧版本中的提交。
- 未修改行同样显示其提交归属；尚未提交的新增行显示“尚未提交”。
- 暂存区使用 diff 实际显示的内容快照，避免后续工作区或未保存修改造成行号和归属错配。
- 同时加载 diff 左右两侧；重新打开已缓存的编辑器或刷新非活动侧暂存内容时，更新对应装饰。

Current-line blame now works on both sides of native extension diffs, including added, deleted and unchanged lines. Historical sides use their own revisions, and index sides use the displayed snapshot. Uncommitted additions are labeled as uncommitted. Annotations continue to follow each editor's selected line.

## Downloads

- `PushRight_1.1.5_x64-setup.exe` — Windows x64 desktop installer.
- `pushright-win32-x64-1.1.5.vsix` — VS Code / Cursor extension.
- `PushRight-1.1.5-source.zip` — Source code from the release tag, without dependencies or build outputs.
- `SHA256SUMS.txt` — SHA-256 checksums for the installer, VSIX and source archive.

## Validation and scope

- Passed all 26 Node tests and Svelte/TypeScript checks (zero errors or warnings).
- Passed 30 Rust workspace tests. The existing `pull_explicit_mode_and_conflict_recovery` test was excluded because it sets `pull.rebase=false` and executes a merge pull, contrary to the required rebase-only Git policy.
- The packaged VSIX passed all 23 smoke checks in both VS Code and Cursor.
- Regression tests cover both diff sides, unchanged/added/deleted lines, historical renamed/deleted files, staged and unsaved snapshots, newly staged files, cached editor reopening, inactive index refresh, and empty diff sides.
- Cursor's smoke log includes host errors for a mutex, authentication, a disposed context, a missing SCM tree node and shutdown storage. All smoke assertions passed; this is not a warning-free host run.
- The Windows desktop installer and VSIX built successfully. Full installer acceptance and manual visual verification of the line-end annotations were not performed.
- Windows 10/11 x64 local workspaces only. System Git is required; desktop requires WebView2. The installer is unsigned.
