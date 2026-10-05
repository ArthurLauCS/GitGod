import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { test } from 'node:test'
import ts from 'typescript'

// Exercise host handlers without launching VS Code; the smoke test covers the actual host.
const require = createRequire(import.meta.url)
function load(name, deps) {
  const source = readFileSync(new URL(`../extension/src/${name}.ts`, import.meta.url), 'utf8')
  const js = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2023 } }).outputText
  const exports = {}
  new Function('require', 'exports', js)((id) => deps[id] ?? require(id), exports)
  return exports
}
class Event {
  listeners = []
  event = (fn) => { this.listeners.push(fn); return { dispose: () => this.listeners = this.listeners.filter((x) => x !== fn) } }
  fire = (value) => this.listeners.map((fn) => fn(value))
  dispose() {}
}
const deferred = () => { let resolve; const promise = new Promise((r) => resolve = r); return { promise, resolve } }
const uri = (fsPath) => ({ fsPath, scheme: 'file', with: (other) => ({ ...uri(fsPath), ...other }) })

test('graph keeps its document on theme changes and releases both loaded and late sessions', async () => {
  const messages = new Event(), closed = new Event(), theme = new Event()
  const posted = [], calls = []
  const late = deferred()
  const webview = { html: '', cspSource: 'self', asWebviewUri: (u) => u.fsPath, postMessage: (m) => posted.push(m), onDidReceiveMessage: messages.event }
  const panel = { webview, onDidDispose: closed.event }
  const vscode = {
    Uri: { file: uri, joinPath: (u, p) => uri(`${u.fsPath}/${p}`) }, ColorThemeKind: { Dark: 2, HighContrast: 3 }, ViewColumn: { Active: 1 },
    window: { createWebviewPanel: () => panel, activeColorTheme: { kind: 2 }, onDidChangeActiveColorTheme: theme.event },
  }
  const core = { engine: { call: async (cmd, args) => { calls.push([cmd, args]); return cmd === 'open_repo' ? args.path === 'late' ? late.promise : [7, 'repo', 2] : null } }, panel: {}, stored: () => ({}), repos: [], locale: 'en' }
  load('panel', { vscode, './core': core }).openPanel({ extensionUri: uri('/extension') })
  const html = webview.html
  await Promise.all(messages.fire({ id: 1, cmd: 'open_repo', args: { path: 'repo' } }))
  theme.fire({ kind: 1 })
  assert.equal(webview.html, html)
  assert.deepEqual(posted.at(-1), { store: 'theme', value: 'light' })
  const opening = messages.fire({ id: 2, cmd: 'open_repo', args: { path: 'late' } })
  closed.fire()
  late.resolve([8, 'late', 2])
  await Promise.all(opening)
  assert.deepEqual(calls.filter(([cmd]) => cmd === 'close_repo').map(([, args]) => args.tab), [7, 8])
})

test('history context menus resolve their list and stale file requests cannot replace it', async () => {
  const editorChanged = new Event(), repoChanged = new Event(), commands = new Map(), views = new Map(), diffs = []
  const repo = { tab: 0, refs: { head_id: 'head' } }
  const pending = new Map()
  const vscode = {
    EventEmitter: Event, TabInputTextDiff: class {},
    window: {
      activeTextEditor: undefined, onDidChangeActiveTextEditor: editorChanged.event,
      createTreeView: (id, { treeDataProvider }) => { const view = { visible: true, onDidChangeVisibility: new Event().event, provider: treeDataProvider }; views.set(id, view); return view },
    },
    commands: { registerCommand: (id, fn) => commands.set(id, fn), executeCommand: (...args) => diffs.push(args) },
  }
  const core = {
    onRepoChange: repoChanged, repoOf: () => repo, rel: (_, u) => u.fsPath, v: {}, attempt: (p) => p,
    revUri: (_, path, rev) => ({ path, rev }),
    engine: { call: (_, { path }) => { const d = deferred(); pending.set(path, d); return d.promise } },
  }
  load('history', { vscode, './core': core }).registerHistory({ subscriptions: [] })
  const follow = async (path) => { vscode.window.activeTextEditor = { document: { uri: uri(path) } }; editorChanged.fire(); await Promise.resolve() }
  const a = [{ id: 'a', path: 'a.txt', subject: 'a' }]
  await follow('a.txt'); pending.get('a.txt').resolve(a); await new Promise(setImmediate)
  const list = views.get('pushright.fileHistory').provider
  assert.equal(list.items, a)
  await commands.get('pushright.item.openChanges')(a[0])
  assert.equal(diffs.at(-1)[0], 'vscode.diff')
  assert.equal(diffs.at(-1)[2].rev, 'a')
  await follow('b.txt'); await follow('a.txt')
  pending.get('b.txt').resolve([{ id: 'b', path: 'b.txt' }]); await new Promise(setImmediate)
  assert.equal(list.items, a)
  const count = pending.size
  repoChanged.fire(repo)
  assert.equal(pending.size, count)
})

test('push mismatch recommends an explicit same-name destination; cancel sends nothing', async () => {
  const sent = [], prompts = []
  const repo = { refs: { head: 'refs/heads/feature/x', refs: [{ name: 'refs/heads/feature/x', upstream: 'refs/remotes/origin/main' }] }, remotes: ['origin'] }
  const v = {}, t = { pushTitle: (b) => b, pushToSameName: (b) => b, pushToUpstream: (b) => b, pushMismatch: (b, up) => `${b}:${up}`, pushTo: (b) => b, push: 'Push', forcePush: 'Force' }
  const vscode = { window: { showQuickPick: async (items) => { prompts.push(items); return items[0] }, showInformationMessage: async () => t.push } }
  const { splitUpstream } = await import('../src/lib/push-target.ts')
  const core = { headBranch: () => 'feature/x', run: (_, op) => sent.push(op), t, v, help: { push: { what: '' } } }
  const { push } = load('sync', { vscode, './core': core, '../../src/lib/push-target': { splitUpstream } })
  await push(repo)
  assert.equal(prompts[0][0].label, 'origin/feature/x')
  assert.deepEqual(sent, [{ op: 'push', branch: 'feature/x', force: false, remote: 'origin', remote_branch: 'feature/x', set_upstream: true }])
  vscode.window.showQuickPick = async () => undefined
  await push(repo)
  assert.equal(sent.length, 1)
})

test('SCM separates repositories, stops on failed saves and uses selections after formatting', async () => {
  const commands = new Map(), providers = new Map(), calls = [], writes = []
  const repos = [{ tab: 0, root: '/one', name: 'one', status: [] }, { tab: 1, root: '/two', name: 'two', status: [] }]
  const selection = (line) => ({ start: { line, character: 0 }, end: { line, character: 1 } })
  const editor = { document: { uri: uri('/one/file'), isDirty: true, lineCount: 2, save: async () => false }, selections: [selection(0)] }
  const event = () => ({ dispose() {} })
  const vscode = {
    EventEmitter: Event, Uri: { file: uri },
    scm: { createSourceControl: () => ({ createResourceGroup: () => ({}), inputBox: {} }) },
    commands: { registerCommand: (id, fn) => commands.set(id, fn) },
    window: { activeTextEditor: editor, onDidChangeWindowState: event },
    workspace: {
      textDocuments: [], createFileSystemWatcher: () => ({ onDidChange: event, onDidCreate: event, onDidDelete: event }),
      registerTextDocumentContentProvider: (scheme, provider) => providers.set(scheme, provider),
    },
  }
  vscode.window.registerFileDecorationProvider = event
  const diff = { hunks: [{ header: '@@ -1,0 +1,2 @@', lines: [1, 2].map((n) => ({ kind: '+', new_no: n })) }] }
  const core = {
    repos, onRepoChange: new Event(), REV: 'rev', v: {}, t: { identityMissing: '' }, help: {},
    repoOf: () => repos[0], rel: () => 'file', revOf: () => '', errorText: String, attempt: (p) => p,
    engine: { call: async (cmd) => { calls.push(cmd); return diff } },
    write: async (repo, cmd, args) => { writes.push([repo.tab, cmd, args]); return null },
  }
  load('scm', { vscode, './core': core, './sync': { statusCommands: () => [] } }).registerScm({ subscriptions: [] })
  await commands.get('pushright.stage')({ repo: repos[0], entry: { path: 'a' } }, { repo: repos[1], entry: { path: 'b' } })
  assert.deepEqual(writes.splice(0), [[0, 'stage', { paths: ['a'] }], [1, 'stage', { paths: ['b'] }]])
  await commands.get('pushright.stageSelection')()
  assert.deepEqual(calls, [])
  editor.document.save = async () => { editor.selections = [selection(1)]; return true }
  await commands.get('pushright.stageSelection')()
  assert.deepEqual(writes[0][2].lines, [1])
  core.engine.call = async () => null
  const content = providers.get('rev').provideTextDocumentContent
  assert.equal(await content({ query: '{}' }), '')
  core.engine.call = async () => { throw new Error('PR_FILE_TOO_LARGE') }
  await assert.rejects(content({ query: '{}' }), /PR_FILE_TOO_LARGE/)
})
