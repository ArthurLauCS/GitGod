# PushRight

**See your branches. Push to the right place.**

[English](https://github.com/ArthurLauCS/PushRight/blob/main/extension/README.md) · [简体中文](https://github.com/ArthurLauCS/PushRight/blob/main/extension/README.zh-CN.md)

PushRight brings Git source control, blame, history and a commit graph into VS Code. It makes push destinations explicit and defaults every Pull to **Rebase instead of merge**. English and Simplified Chinese are included; the extension follows VS Code's display language.

## Requirements

- Windows 10 / 11 x64, VS Code 1.90 or newer, and Git on PATH.
- A trusted local Git workspace. Linux, macOS, WSL, SSH and browser workspaces are not supported by this build.
- Use your existing Git credential helper or SSH configuration for authentication.

## Push to the intended branch

If local `feature/login` accidentally tracks `origin/main`, PushRight warns about the mismatch and recommends `origin/feature/login`. You choose the destination before it pushes with an explicit remote and refspec. Force push uses `--force-with-lease` and requires an additional confirmation.

Pull asks every time, with **Rebase instead of merge** checked. Keep it checked when your team requires linear pull history. Automatic stashing is disabled; commit or stash local changes first. Resolve conflicts and use **Continue / Abort** when Git pauses.

## Source control and history

- Review staged, unstaged and conflicted files; stage, unstage or discard files or selected lines; commit and amend.
- Inspect current-line blame, author hover cards, CodeLens and whole-file blame (`Alt+B`). Apply backgrounds and borders to individual authors.
- Keep **PushRight > Line History** expanded to follow the clicked line or selected range automatically, without a context-menu command or moving focus away from the editor. Rapid cursor moves are coalesced and hidden views do not query history.
- Browse file history; compare revisions (`Alt+,` / `Alt+.`); search by message, author, file, changed text or commit ID.
- **PushRight Graph** appears below changes in the **Source Control** sidebar, including when built-in Git is disabled. It shows all branches with commit lanes and branch/tag labels. Choose a repository from its dropdown; click a commit to open its details in the full graph.
- Run **PushRight: Open Commit Graph**, or use the sidebar graph's title button, to browse branches, commits and file differences in an editor tab. If the sidebar graph was hidden, enable **PushRight Graph** in Source Control's **Views** menu.
- Change the commit name and email for the current repository.

On first use, PushRight offers to turn off built-in Git, so push and pull go through its checks. This is optional. Extensions that depend on built-in Git may stop working when it is disabled. If you keep GitLens enabled, turn off duplicate blame annotations in one extension.

## Preview limits

- PR / Issue and hosting-provider collaboration are not included yet. PushRight does not replace those parts of GitLens or GitHub Pull Requests.
- Workspace-folder repositories are detected at startup. Reload after changing workspace folders; nested repository discovery is not included.
- File history lists at most 200 commits. Commit or stash changes to a file before viewing its line history, so selection line numbers match HEAD.
- Text revisions and patches are limited to 4 MiB. Binary revisions are not rendered as text. Whole-file Git operations remain available.
- Full commit history loads in the background. See the [performance report](https://github.com/ArthurLauCS/PushRight/blob/main/docs/PERFORMANCE.md) for measured limits.

## Install from VSIX

Download the Windows x64 VSIX from [GitHub Releases](https://github.com/ArthurLauCS/PushRight/releases/tag/vscode-v0.2.2), then use **Extensions: Install from VSIX** in VS Code, or:

```sh
code --install-extension pushright-win32-x64-0.2.2.vsix
```

Report problems in [GitHub Issues](https://github.com/ArthurLauCS/PushRight/issues), including the version and reproduction steps. Remove credentials and private repository information from logs before sharing.

Third-party notices ship in `THIRD_PARTY_NOTICES.txt`. The author has not selected an open-source license for PushRight.
