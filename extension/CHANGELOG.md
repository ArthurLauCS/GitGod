# Changelog

## 1.0.1

- Adopt the MIT license and include it in the VSIX and desktop installer.
- Update direct downloads to the licensed release. Runtime behavior is unchanged from 1.0.0.

采用 MIT 开源许可证，随 VSIX 与桌面安装包分发，并更新直接下载入口。功能与首个正式版 1.0.0 保持一致。

## 1.0.0

First stable release for Windows x64, with English and Simplified Chinese UI.

- Fix duplicate text-model errors when opening unstaged diffs and inline changes.
- Share revision reads between gutter change markers and diff editors.
- Close individual repositories from Source Control and reopen them explicitly. Closed repositories stay closed across file switches, refreshes and window reloads.
- Include native commit-file diffs, branch filters, nested repositories and local ignore/tracking controls from the preview releases.
- Provide a direct VSIX download for VS Code and Cursor, alongside the desktop installer.

首次正式版，提供 Windows x64 桌面安装包和扩展 VSIX，内置英文与简体中文。修复未暂存差异及行内改动详情的重复模型错误，复用行边标记与差异编辑器的版本读取；支持关闭、重新打开仓库，并在刷新和重载后保留关闭状态。
