# PushRight 1.1.2 — Branch picker commits and branch naming

桌面版与 VS Code / Cursor 扩展同步更新。

## 修复与改进

- 扩展版的分支切换列表在每个分支下显示最新提交：作者、短 ID、标题和提交时间，可按提交内容筛选。
- 扩展版提交图显示的分支改在顶部选择框中选择，列表与分支切换一致；筛选不会切换检出的分支。
- 新建分支时提示并校验命名规则 `(fix|feat)_(S|C|SC)_小驼峰`（如 `feat_S_userLogin`）；最后一段以小写英文字母开头，仅包含英文字母和数字。桌面版新建分支对话框和扩展分支选择框可选择起始分支；桌面新建工作树对话框也校验新分支名。已有分支名称不受影响。

## Downloads

- `PushRight_1.1.2_x64-setup.exe` — Windows x64 desktop installer.
- `pushright-win32-x64-1.1.2.vsix` — VS Code / Cursor extension.
- `PushRight-1.1.2-source.zip` — Source code from the release tag, without dependencies or build outputs.
- `SHA256SUMS.txt` — SHA-256 checksums for the installer, VSIX and source archive.

## Validation and scope

- Passed 23 Node tests, 31 Rust workspace tests, and Svelte/TypeScript checks (zero errors or warnings).
- The packaged VSIX passed the existing smoke suite in both VS Code and Cursor. The desktop release executable passed a process-startup check; this is not a full installer/UI interaction test.
- During the multi-repository VS Code smoke run, the host logged duplicate quick-diff visibility command registrations; the smoke assertions still passed. This diagnostic remains for follow-up.
- Windows 10/11 x64 local workspaces only. System Git is required; desktop requires WebView2. The installer is unsigned.
