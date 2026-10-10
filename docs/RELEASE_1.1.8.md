# PushRight 1.1.8 — Searchable revision pickers

版本比较与交互式变基的版本选择框支持直接输入筛选，并按层级显示分支。

## Changes / 更新

- 当前分支及上游排在最前，其次为常用分支（`main`、`master`、`season`、`dev`、`develop`），随后为其他本地分支、远程分支、标签及其他引用。常用分支按名称识别，不统计使用频率。
- 输入时实时筛选，不区分大小写；支持鼠标选择、方向键、回车确认和 Esc 关闭。
- 可直接填写提交号或 `HEAD~1` 等版本表达式；选择分支和标签时仍使用完整引用，避免同名歧义。
- 桌面版与扩展完整提交图复用同一控件，交互式变基的基点选择也同步改进。侧栏分支列表和扩展的检出选择框保持原有顺序。

Comparison and interactive rebase tools now use editable, searchable revision pickers. The current branch and upstream come first, followed by common branches (`main`, `master`, `season`, `dev`, `develop`), other local branches, remote branches, tags and other refs. Common branches are recognized by name, not usage frequency. Mouse and keyboard selection are supported; direct commit IDs and revision expressions remain available. Selected refs retain their fully qualified names. Both the desktop and extension full graph share the change.

## Downloads

- `PushRight_1.1.8_x64-setup.exe` — Windows x64 desktop installer.
- `pushright-win32-x64-1.1.8.vsix` — VS Code / Cursor extension.
- `PushRight-1.1.8-source.zip` — Source code from the release tag.
- `SHA256SUMS.txt` — SHA-256 checksums for all three artifacts.

## Validation and scope

- Regression coverage includes grouping, stable ordering, same-named branches/tags, case-insensitive filtering, detached HEAD, empty repositories and missing upstreams.
- Browser interaction checks used 559 real refs from `rokemon_061`, including season/dev prioritization, mouse and keyboard selection, Escape dismissal and direct revision input.
- Release checks: Node tests, Svelte/TypeScript checks, Rust workspace tests, desktop build and packaged VSIX smoke checks in VS Code and Cursor.
- The Rust `pull_explicit_mode_and_conflict_recovery` test is excluded because it configures `pull.rebase=false` and executes a merge pull, contrary to the required rebase-only policy.
- Full native installer acceptance is not covered by browser checks. Windows 10/11 x64 local workspaces only; system Git and WebView2 are required. The installer is unsigned.
