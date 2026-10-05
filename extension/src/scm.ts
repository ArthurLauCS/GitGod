import { basename } from 'node:path'
import * as vscode from 'vscode'
import type { Diff, Entry, Identity } from '../../src/lib/api'
import {
  activeRepo, attempt, engine, fileUri, help, onRepoChange, refresh, rel, repoOf, repos, REV, revOf, revUri, run, t, v, write, type Repo,
} from './core'
import { statusCommands } from './sync'

/** 源代码管理面板里的一行 */
export interface Resource extends vscode.SourceControlResourceState {
  repo: Repo
  entry: Entry
  staged: boolean
}

const COLORS: Record<string, string> = {
  M: 'gitDecoration.modifiedResourceForeground',
  T: 'gitDecoration.modifiedResourceForeground',
  A: 'gitDecoration.addedResourceForeground',
  D: 'gitDecoration.deletedResourceForeground',
  R: 'gitDecoration.renamedResourceForeground',
  C: 'gitDecoration.renamedResourceForeground',
  '?': 'gitDecoration.untrackedResourceForeground',
  U: 'gitDecoration.conflictingResourceForeground',
}

const controls = new Map<Repo, vscode.SourceControl>()

/** 命令的参数可能是仓库（状态栏）、SourceControl（面板标题栏）或没有（命令面板）。 */
export function repoArg(arg: unknown): Repo | undefined {
  return repos.find((r) => r === arg || controls.get(r) === arg)
}

function resource(repo: Repo, entry: Entry, staged: boolean): Resource {
  const letter = entry.conflicted ? 'U' : (staged ? entry.staged : entry.unstaged)!
  const state: Resource = {
    resourceUri: fileUri(repo, entry.path),
    repo, entry, staged,
    contextValue: letter === '?' ? 'untracked' : 'tracked',
    decorations: { strikeThrough: letter === 'D', tooltip: t.status[letter] },
  }
  return Object.assign(state, { command: { command: 'pushright.openChange', title: '', arguments: [state] } })
}

async function openChange({ repo, entry, staged, resourceUri }: Resource) {
  // 未跟踪的目录在列表里是一条，没有内容可看
  if (entry.path.endsWith('/')) return void vscode.commands.executeCommand('revealInExplorer', resourceUri)
  // 冲突文件直接打开，VS Code 自带的冲突标记工具可以逐块选择
  if (entry.conflicted || entry.unstaged === '?') return void vscode.window.showTextDocument(resourceUri)
  const left = staged ? revUri(repo, entry.old_path ?? entry.path, 'HEAD') : revUri(repo, entry.path, '')
  const right = staged ? revUri(repo, entry.path, '') : resourceUri
  vscode.commands.executeCommand('vscode.diff', left, right, `${basename(entry.path)} (${staged ? t.staged : t.unstaged})`)
}

async function commit(repo: Repo, amend: boolean) {
  const box = controls.get(repo)!.inputBox
  const message = box.value.trim() || (amend ? await attempt(engine.call<string>('last_message', { tab: repo.tab })) : '')
  if (!message) return void vscode.window.showWarningMessage(v.emptyMessage)
  if (!amend && !repo.status.some((e) => e.staged)) {
    if (!repo.status.length) return void vscode.window.showInformationMessage(t.clean)
    if (!(await vscode.window.showInformationMessage(v.nothingStaged, { modal: true, detail: help.stage.what }, v.stageAllAndCommit))) return
    if ((await write(repo, 'stage', { paths: repo.status.map((e) => e.path) })) === undefined) return
  }
  if ((await write(repo, 'commit', { message, amend })) !== undefined) box.value = ''
}

async function editIdentity(repo: Repo) {
  const ask = (title: string, value: string) => vscode.window.showInputBox({ title, value, prompt: t.identityScope, ignoreFocusOut: true })
  const name = await ask(t.identityName, repo.identity?.name ?? '')
  if (name === undefined) return
  const email = await ask(t.identityEmail, repo.identity?.email ?? '')
  if (email === undefined) return
  const identity = await write<Identity>(repo, 'set_commit_identity', { name, email })
  const expected = `${name.trim()} <${email.trim()}>`
  if (identity && (identity.author !== expected || identity.committer !== expected)) vscode.window.showWarningMessage(t.identityOverridden)
}

async function discard(repo: Repo, paths: string[]) {
  const detail = `${help.discard.what}\n\n${help.discard.undo}`
  if (await vscode.window.showWarningMessage(t.discardTitle(paths.length), { modal: true, detail }, t.discardConfirm)) await run(repo, { op: 'discard', paths })
}

/** 暂存 / 取消暂存 / 丢弃编辑器里选中的行。取消暂存要在暂存区版本的编辑器里选。 */
async function applySelection(mode: 'stage' | 'unstage' | 'discard') {
  const editor = vscode.window.activeTextEditor
  const repo = repoOf(editor?.document.uri)
  if (!editor || !repo) return
  const staged = mode === 'unstage'
  if (staged ? revOf(editor.document.uri) !== '' : editor.document.uri.scheme !== 'file') return void vscode.window.showWarningMessage(v.selectionSide)
  if (mode === 'discard' && !(await vscode.window.showWarningMessage(t.discardLinesTitle, { modal: true, detail: help.discard.undo }, t.discardConfirm))) return
  // 选区在行首结束时，最后那一行其实没被选中
  const ranges = editor.selections.map((s) => [s.start.line + 1, s.end.line + (s.end.character === 0 && s.end.line > s.start.line ? 0 : 1)])
  const selected = (line: number) => ranges.some(([from, to]) => line >= from && line <= to)
  if (editor.document.isDirty) await editor.document.save()

  const path = rel(repo, editor.document.uri)
  const diff = await attempt(engine.call<Diff>('diff_worktree', { tab: repo.tab, path, staged, untracked: false }))
  if (!diff) return
  let applied = false
  // 从后往前处理：改掉后面的区块不影响前面区块的位置和标题行
  for (let hunk = diff.hunks.length - 1; hunk >= 0; hunk--) {
    const { header, lines } = diff.hunks[hunk]
    const picked: number[] = []
    // 删除行在编辑器里不存在，算在紧跟它的那一行上；文件末尾的删除算在最后一行上
    let next = Math.max(1, +(/\+(\d+)/.exec(header)?.[1] ?? 1))
    lines.forEach((line, i) => {
      if (line.kind === '-') {
        if (selected(Math.min(next, editor.document.lineCount))) picked.push(i)
      } else if (line.new_no !== null) {
        if (line.kind === '+' && selected(line.new_no)) picked.push(i)
        next = line.new_no + 1
      }
    })
    if (!picked.length) continue
    applied = true
    const args = mode === 'discard' ? { path, hunk, header, lines: picked } : { path, staged, hunk, header, lines: picked }
    if ((await write(repo, mode === 'discard' ? 'discard_lines' : 'apply_lines', args)) === undefined) return
  }
  if (!applied) vscode.window.showInformationMessage(v.noChangedLines)
}

/** 第一次启用时说明后果，由用户决定是否关掉 VS Code 自带的 Git。 */
export async function offerTakeover(context: vscode.ExtensionContext, force = false) {
  const git = vscode.workspace.getConfiguration('git')
  if (git.get('enabled') && (force || !context.globalState.get('takeoverAsked'))) {
    const pick = await vscode.window.showInformationMessage(v.takeoverTitle, { modal: true, detail: v.takeoverDetail }, v.takeoverEverywhere, v.takeoverWorkspace)
    await context.globalState.update('takeoverAsked', true)
    if (pick) await git.update('enabled', false, pick === v.takeoverEverywhere ? vscode.ConfigurationTarget.Global : vscode.ConfigurationTarget.Workspace)
  } else if (force) {
    vscode.window.showInformationMessage(v.takeoverDone)
  }
  if (vscode.extensions.getExtension('eamodio.gitlens') && !context.globalState.get('gitlensNoted')) {
    await context.globalState.update('gitlensNoted', true)
    vscode.window.showInformationMessage(v.gitlensNote)
  }
}

export function registerScm(context: vscode.ExtensionContext) {
  const revChanged = new vscode.EventEmitter<vscode.Uri>()
  const decorationsChanged = new vscode.EventEmitter<undefined>()
  /** 每个仓库里有改动的文件：小写路径 → 状态 */
  const changed = new Map<Repo, Map<string, Entry>>()
  const groups = new Map<Repo, Record<'conflicts' | 'staged' | 'changes', vscode.SourceControlResourceGroup>>()

  for (const repo of repos) {
    const control = vscode.scm.createSourceControl('pushright', `PushRight · ${repo.name}`, vscode.Uri.file(repo.root))
    control.acceptInputCommand = { command: 'pushright.commit', title: t.commit, arguments: [control] }
    control.quickDiffProvider = {
      // 暂存区里没有这个文件（未跟踪、被忽略）时不显示行号旁的改动标记
      provideOriginalResource: (uri) =>
        uri.scheme === 'file' && repoOf(uri) === repo
          ? engine.call('show', { tab: repo.tab, rev: '', path: rel(repo, uri) }).then(() => revUri(repo, rel(repo, uri), ''), () => undefined)
          : undefined,
    }
    const group = (id: string, label: string, hide: boolean) => {
      const g = control.createResourceGroup(id, label)
      g.hideWhenEmpty = hide
      return g
    }
    groups.set(repo, { conflicts: group('conflicts', v.conflicts, true), staged: group('staged', t.staged, true), changes: group('changes', t.unstaged, false) })
    controls.set(repo, control)
    context.subscriptions.push(control)
  }

  const update = (repo: Repo) => {
    const control = controls.get(repo)!
    const g = groups.get(repo)!
    g.conflicts.resourceStates = repo.status.filter((e) => e.conflicted).map((e) => resource(repo, e, false))
    g.staged.resourceStates = repo.status.filter((e) => e.staged).map((e) => resource(repo, e, true))
    g.changes.resourceStates = repo.status.filter((e) => e.unstaged && !e.conflicted).map((e) => resource(repo, e, false))
    control.count = repo.status.length
    control.statusBarCommands = statusCommands(repo)
    control.inputBox.placeholder = repo.identity?.name ? v.commitAs(`${repo.identity.name} <${repo.identity.email}>`) : t.identityMissing
    changed.set(repo, new Map(repo.status.map((e) => [fileUri(repo, e.path).fsPath.toLowerCase(), e])))
    decorationsChanged.fire(undefined)
    // 暂存区和 HEAD 的内容可能变了，让打开着的对比视图重新取
    for (const doc of vscode.workspace.textDocuments) {
      if (repoOf(doc.uri) === repo && ['', 'HEAD'].includes(revOf(doc.uri) ?? '-')) revChanged.fire(doc.uri)
    }
  }
  repos.forEach(update)

  // 文件或 .git 有变化时刷新，300 毫秒内的连续变化合并成一次
  const timers = new Map<Repo, ReturnType<typeof setTimeout>>()
  const schedule = (uri: vscode.Uri) => {
    const repo = repoOf(uri)
    if (!repo || /[\\/]\.git[\\/].*\.lock$/.test(uri.fsPath)) return
    clearTimeout(timers.get(repo))
    timers.set(repo, setTimeout(() => refresh(repo), 300))
  }
  const watcher = vscode.workspace.createFileSystemWatcher('**')
  const command = (id: string, fn: (...args: any[]) => unknown) => vscode.commands.registerCommand(`pushright.${id}`, fn)
  const paths = (states: Resource[]) => states.map((s) => s.entry.path)
  /** 命令作用的仓库：分组右键菜单传来的是分组对象，其余见 repoArg；都不是就取当前文件所在的仓库 */
  const target = (arg: unknown) =>
    repos.find((r) => Object.values(groups.get(r)!).includes(arg as vscode.SourceControlResourceGroup)) ?? repoArg(arg) ?? activeRepo()

  context.subscriptions.push(
    revChanged, decorationsChanged, watcher,
    onRepoChange.event(update),
    watcher.onDidChange(schedule), watcher.onDidCreate(schedule), watcher.onDidDelete(schedule),
    vscode.window.onDidChangeWindowState((state) => state.focused && repos.forEach(refresh)),
    vscode.workspace.registerTextDocumentContentProvider(REV, {
      onDidChange: revChanged.event,
      provideTextDocumentContent: (uri) => {
        const repo = repoOf(uri)
        // 这个版本里没有该文件时显示为空，对比视图里就是整份新增或删除
        return repo ? engine.call<string>('show', { tab: repo.tab, rev: revOf(uri), path: rel(repo, uri) }).catch(() => '') : ''
      },
    }),
    vscode.window.registerFileDecorationProvider({
      onDidChangeFileDecorations: decorationsChanged.event,
      provideFileDecoration: (uri) => {
        const repo = uri.scheme === 'file' ? repoOf(uri) : undefined
        const entry = repo && changed.get(repo)?.get(uri.fsPath.toLowerCase())
        if (!entry) return
        const letter = entry.conflicted ? 'U' : (entry.unstaged ?? entry.staged)!
        // 未跟踪显示 U，冲突显示 C
        const badge = letter === '?' ? 'U' : letter === 'U' ? 'C' : letter
        return { badge, tooltip: t.status[letter], color: new vscode.ThemeColor(COLORS[letter]), propagate: true }
      },
    }),
    command('openChange', openChange),
    command('openFile', (state: Resource) => vscode.window.showTextDocument(state.resourceUri)),
    command('stage', (...states: Resource[]) => write(states[0].repo, 'stage', { paths: paths(states) })),
    command('unstage', (...states: Resource[]) => write(states[0].repo, 'unstage', { paths: paths(states) })),
    command('discard', (...states: Resource[]) => discard(states[0].repo, paths(states))),
    command('takeOurs', (state: Resource) => write(state.repo, 'conflict_take', { path: state.entry.path, theirs: false })),
    command('takeTheirs', (state: Resource) => write(state.repo, 'conflict_take', { path: state.entry.path, theirs: true })),
    command('stageAll', (arg) => {
      const repo = target(arg)
      return repo && write(repo, 'stage', { paths: repo.status.filter((e) => e.unstaged).map((e) => e.path) })
    }),
    command('unstageAll', (arg) => {
      const repo = target(arg)
      return repo && write(repo, 'unstage', { paths: repo.status.filter((e) => e.staged).map((e) => e.path) })
    }),
    command('discardAll', (arg) => ((repo) => repo && discard(repo, []))(target(arg))),
    command('commit', (arg) => ((repo) => repo && commit(repo, false))(target(arg))),
    command('commitAmend', (arg) => ((repo) => repo && commit(repo, true))(target(arg))),
    command('editIdentity', (arg) => ((repo) => repo && editIdentity(repo))(target(arg))),
    command('refresh', () => repos.forEach(refresh)),
    command('stageSelection', () => applySelection('stage')),
    command('unstageSelection', () => applySelection('unstage')),
    command('discardSelection', () => applySelection('discard')),
    command('takeover', () => offerTakeover(context, true)),
  )
}
