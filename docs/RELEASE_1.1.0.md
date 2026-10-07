# PushRight 1.1.0 — Windows Git Tools

桌面版与 VS Code / Cursor 扩展同步更新，完善 Windows 本地 Git 工作流。

## 新增与改进

- 整仓比较任意提交、分支和标签，可从共同祖先比较；按版本保存文件审阅标记。
- 添加、删除、重命名远程，分别编辑 Fetch / Push 地址。
- 交互式变基：调整顺序，支持 pick/reword/edit/squash/fixup/drop；自动备份分支，并支持冲突后继续或中止。
- Reflog 分页浏览，并从选中提交恢复为新分支。
- 图片前后预览，欢迎页增加克隆和初始化入口。
- 文件历史、行历史和搜索可加载超过 200 条；支持本地与未保存改动中未改变选区的行历史映射。
- 大补丁每页 1000 行只读预览；历史文本读取上限可从 4 MiB 调整至 64 MiB。

## Downloads

- `PushRight_1.1.0_x64-setup.exe` — Windows x64 desktop installer.
- `pushright-win32-x64-1.1.0.vsix` — VS Code / Cursor extension; use **Extensions: Install from VSIX**.
- `SHA256SUMS.txt` — SHA-256 checksums for both packages.

## English summary

Adds repository comparisons, persistent reviewed-file markers, remote management, interactive rebase, reflog recovery, image diffs and clone/init. File/line history and search now page beyond 200 entries. Unchanged selections in dirty buffers map back to HEAD. Large patches offer read-only streaming pages and native text revision limits are configurable.

## Validation and scope

- 30 Rust engine tests and 20 Node tests passed; Svelte and TypeScript checks passed.
- Packaged extension passed 23 native smoke checks in each of VS Code 1.140.0 and Cursor 3.22.12.
- Desktop comparison, review markers, remote details, rebase planning and reflog UI checked.
- Windows 10/11 x64 local workspaces only. System Git is required; desktop requires WebView2. The installer is unsigned.
- Rebase editor handles linear ranges after an ancestor of HEAD; merge ranges and root editing are excluded. New or modified selected lines cannot map to committed line history.
- PR/Issue collaboration and cloud patches are not included. See [verification and limits](https://github.com/ArthurLauCS/PushRight/blob/v1.1.0/docs/FEATURE_COMPLETION.md).

MIT licensed. No Node.js or Rust installation is needed to use the packaged application or extension.
