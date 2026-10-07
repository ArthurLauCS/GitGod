# PushRight

**See your branches. Push to the right place.**

[English](https://github.com/ArthurLauCS/PushRight/blob/main/extension/README.md) · [简体中文](https://github.com/ArthurLauCS/PushRight/blob/main/extension/README.zh-CN.md)

PushRight brings Git source control, blame, history and a commit graph into VS Code. It makes push destinations explicit and defaults every Pull to **Rebase instead of merge**. English and Simplified Chinese are included; the extension follows VS Code's display language.

**[Download VSIX · Windows x64 · 1.0.1](https://github.com/ArthurLauCS/PushRight/releases/download/v1.0.1/pushright-win32-x64-1.0.1.vsix)** · [Release notes and checksums](https://github.com/ArthurLauCS/PushRight/releases/tag/v1.0.1)

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
- **PushRight Graph** appears below changes in **Source Control**, including when built-in Git is disabled. It opens automatically once per workspace. Click a commit to expand its changed files in place; click a file for a native diff preview, or double-click to keep the diff open. Arrow keys navigate commits and files; Left/Right collapse/expand.
- Choose **Auto**, **All branches**, or a local/remote branch. Auto shows HEAD, its upstream and a remote base (the existing VS Code base setting, the branch creation source, then the remote default branch). Selection is remembered per repository and does not check out a branch. Fetch, Pull and Push act on the displayed repository's **checked-out branch**; the graph's Push always targets a remote branch with the same name and confirms the destination.
- Initialized submodules and nested repositories have separate repository entries, graphs and source controls. Child names include their parent path. Use **Refresh** after adding a repository; workspace-folder additions are detected automatically. An ignored nested repository is also detected when you open one of its files.
- New folders list their untracked files individually, so you can inspect and stage each file before committing. Ignored files stay excluded.
- Run **PushRight: Open Commit Graph**, or use the sidebar graph's title button, to browse branches, commits and file differences in an editor tab. If the sidebar graph was hidden, enable **PushRight Graph** in Source Control's **Views** menu.
- Change the commit name and email for the current repository.
- Right-click a repository and choose **Close Repository** to keep its files open without rediscovering it on file switches, refreshes or reloads. Use **PushRight: Reopen Closed Repository…** to restore it.

On first use, PushRight offers to turn off built-in Git, so push and pull go through its checks. This is optional. Extensions that depend on built-in Git may stop working when it is disabled. If you keep GitLens enabled, turn off duplicate blame annotations in one extension.

## Private local files

Expand **Source Control > PushRight Local Files** to browse **Untracked files** and **Ignored files** for each repository. Ignored directories load their direct children only when expanded.

Right-click a file in Explorer to choose:

| Action | Effect |
| --- | --- |
| Ignore Locally | Add its exact path to `.git/info/exclude`; suitable for private skills, never committed or shared. |
| Add to .gitignore | Add its exact path to a rule file that can be committed and shared. |
| Track File (Stage) | Stage the file, even if ignored. It will be included in the next commit. |
| Stop Tracking and Keep Locally | Keep the working file, remove it from the index and ignore it locally. For committed files this **stages a deletion for the team**; the confirmation explains this before applying it. |

Ignore rules do not hide changes to tracked files. Stop-tracking refuses to discard independently staged contents. To undo an ignore rule, edit the corresponding `.gitignore` or `.git/info/exclude`; force-tracking a file does not remove its rule.

## Current limits

- PR / Issue and hosting-provider collaboration are not included yet. PushRight does not replace those parts of GitLens or GitHub Pull Requests.
- Submodules must already be initialized. Ignored directory trees are not scanned for repositories in the background; open a file inside one or add it as a workspace folder.
- File history lists at most 200 commits. Commit or stash changes to a file before viewing its line history, so selection line numbers match HEAD.
- Text revisions and patches are limited to 4 MiB. Binary revisions are not rendered as text. Whole-file Git operations remain available.
- Full commit history loads in the background. See the [performance report](https://github.com/ArthurLauCS/PushRight/blob/main/docs/PERFORMANCE.md) for measured limits.

## Install from VSIX

Download the Windows x64 VSIX from [GitHub Releases](https://github.com/ArthurLauCS/PushRight/releases/tag/v1.0.1), then use **Extensions: Install from VSIX** in VS Code or Cursor, or:

```sh
code --install-extension pushright-win32-x64-1.0.1.vsix
cursor --install-extension pushright-win32-x64-1.0.1.vsix
```

Report problems in [GitHub Issues](https://github.com/ArthurLauCS/PushRight/issues), including the version and reproduction steps. Remove credentials and private repository information from logs before sharing.

Third-party notices ship in `THIRD_PARTY_NOTICES.txt`. PushRight is licensed under the [MIT License](https://github.com/ArthurLauCS/PushRight/blob/main/LICENSE).
