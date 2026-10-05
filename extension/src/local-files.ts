import { basename } from 'node:path'
import * as vscode from 'vscode'
import { attempt, discoverForFile, engine, fileUri, onRepoChange, rel, repoOf, repos, v, write, type Repo } from './core'

type Item = { repo: Repo; kind: 'repo' | 'untracked' | 'ignored'; path: string; directory: boolean }

export function registerLocalFiles(context: vscode.ExtensionContext) {
  const changed = new vscode.EventEmitter<void>()
  const provider: vscode.TreeDataProvider<Item> = {
    onDidChangeTreeData: changed.event,
    getTreeItem(item) {
      const category = !item.path
      const tree = new vscode.TreeItem(category ? item.kind === 'repo' ? item.repo.name : v[item.kind] : basename(item.path), item.directory ? vscode.TreeItemCollapsibleState.Collapsed : vscode.TreeItemCollapsibleState.None)
      tree.id = `${item.repo.root}:${item.kind}:${item.path}`
      tree.tooltip = category ? item.repo.root : fileUri(item.repo, item.path).fsPath
      if (category) tree.iconPath = new vscode.ThemeIcon(item.kind === 'repo' ? 'repo' : item.kind === 'ignored' ? 'eye-closed' : 'diff-added')
      else {
        tree.resourceUri = fileUri(item.repo, item.path)
        tree.contextValue = item.directory ? 'localFolder' : 'localFile'
        if (!item.directory) tree.command = { command: 'vscode.open', title: '', arguments: [tree.resourceUri] }
      }
      return tree
    },
    async getChildren(item) {
      if (!item) return repos.map((repo) => ({ repo, kind: 'repo', path: '', directory: true }))
      if (item.kind === 'repo') return (['untracked', 'ignored'] as const).map((kind) => ({ ...item, kind }))
      const paths = item.kind === 'ignored'
        ? await attempt(engine.call<string[]>('ignored', { tab: item.repo.tab, path: item.path })) ?? []
        : item.repo.status.filter((e) => e.unstaged === '?').map((e) => e.path)
      const prefix = item.path ? `${item.path}/` : ''
      const children = new Map<string, Item>()
      for (const path of paths) {
        if (!path.startsWith(prefix)) continue
        const tail = path.slice(prefix.length)
        const name = tail.split('/')[0]
        if (name) children.set(name, { ...item, path: prefix + name, directory: tail.includes('/') })
      }
      return [...children.values()].sort((a, b) => Number(b.directory) - Number(a.directory) || a.path.localeCompare(b.path))
    },
  }
  const view = vscode.window.createTreeView('pushright.localFiles', { treeDataProvider: provider })
  const command = (id: string, action: 'local' | 'shared' | 'track' | 'untrack') => vscode.commands.registerCommand(`pushright.${id}`, async (arg?: vscode.Uri | Item | { resourceUri: vscode.Uri }) => {
    const uri = arg && 'repo' in arg ? fileUri(arg.repo, arg.path) : arg && 'resourceUri' in arg ? arg.resourceUri : arg as vscode.Uri | undefined ?? vscode.window.activeTextEditor?.document.uri
    if (!uri || uri.scheme !== 'file') return
    await discoverForFile(uri)
    const repo = repoOf(uri)
    if (!repo) return void vscode.window.showWarningMessage(v.noRepo)
    const path = rel(repo, uri)
    if (!path) return
    const tracked = await attempt(engine.call<boolean>('is_tracked', { tab: repo.tab, path }))
    if (tracked === undefined) return
    if ((action === 'local' || action === 'shared') && tracked) return void vscode.window.showWarningMessage(v.ignoreTracked)
    if (action === 'untrack' && !tracked) return void vscode.window.showInformationMessage(v.alreadyUntracked)
    const [title, hint] = { local: [v.ignoreLocal, v.ignoreLocalHint], shared: [v.ignoreShared, v.ignoreSharedHint], track: [v.trackFile, v.trackHint], untrack: [v.untrackFile, v.untrackHint] }[action]
    const detail = `${repo.root}\n${path}\n\n${hint}`
    if (!(await vscode.window.showInformationMessage(title, { modal: true, detail }, title))) return
    if (action === 'local' || action === 'shared') await write(repo, 'ignore_file', { path, shared: action === 'shared' })
    else await write(repo, 'track_file', { path, track: action === 'track' })
    changed.fire()
  })
  context.subscriptions.push(view, changed, onRepoChange.event(() => { if (view.visible) changed.fire() }), view.onDidChangeVisibility(() => changed.fire()),
    command('ignoreLocal', 'local'), command('ignoreShared', 'shared'), command('trackFile', 'track'), command('untrackFile', 'untrack'))
}
