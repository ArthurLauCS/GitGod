# PushRight 1.1.7 — Automatic desktop refresh and fetch

桌面版新增自动刷新与定时 fetch，桌面版和扩展版同步加强强制推送保护。

## 修复与改进

- 桌面应用每分钟刷新所有已打开的仓库，后台页签也生效；保留正在查看的页面和选中的提交。
- 打开仓库时及之后每十分钟自动 fetch。语言选择框旁新增“自动刷新与获取”，两项默认开启，可分别关闭并记住设置。
- 仓库忙碌或弹窗打开时延后执行，任务不重叠；失败记录到命令日志并按周期重试，休眠后不补跑错过的周期。
- 后台 fetch 保留 `FETCH_HEAD`、暂存区和工作区文件，不自动 pull、merge 或 rebase。
- 两端强制推送使用 `--force-with-lease --force-if-includes`，防止后台 fetch 更新远程引用后覆盖尚未在本地整合的远端提交。

The desktop app refreshes all open repositories every minute and fetches on opening and every ten minutes, including background tabs. Both settings default on and can be disabled independently. Refresh preserves the viewed page and selected commit. Busy repositories defer work; failed fetches are logged and retried on schedule. Background fetch preserves `FETCH_HEAD`, the index and working files. Both apps strengthen force-push protection with `--force-if-includes` alongside the lease check.

These timers run while the desktop app is open and stop when it exits. They do not update the application binary. The extension does not enable these desktop timers.

## Downloads

- `PushRight_1.1.7_x64-setup.exe` — Windows x64 desktop installer.
- `pushright-win32-x64-1.1.7.vsix` — VS Code / Cursor extension.
- `PushRight-1.1.7-source.zip` — Source code from the release tag, without dependencies or build outputs.
- `SHA256SUMS.txt` — SHA-256 checksums for the installer, VSIX and source archive.

## Validation and scope

- 32 Node tests and Svelte/TypeScript checks passed.
- The packaged VSIX passed all 24 smoke checks in both VS Code and Cursor.
- 32 Rust workspace tests passed. The existing `pull_explicit_mode_and_conflict_recovery` test was excluded because it sets `pull.rebase=false` and executes a merge pull, contrary to the required rebase-only policy.
- Regression tests cover timer intervals, independent preferences, busy and overlapping operations, closing tabs, failed fetches and missed periods after sleep. Real Git tests verify that background fetch preserves local work and that force push rejects unintegrated remote updates while permitting a legitimate amended push.
- Shared desktop UI and selection preservation were checked with accelerated timers and test data. Full installer acceptance and prolonged native minimized/sleep-resume operation were not performed.
- Cursor host mutex, authentication, disposed-context, SCM tree and shutdown-storage errors were previously reproduced without PushRight loaded, using Cursor's built-in Git. They remain host limitations; passing feature checks does not imply an error-free host run.
- Windows 10/11 x64 local workspaces only. System Git is required; desktop requires WebView2. The installer is unsigned.
