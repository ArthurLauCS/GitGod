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

test('SCM graph routes only bounded reads to workspace repositories and disposes listeners', async () => {
  let provider
  const messages = new Event(), changed = new Event(), prefs = new Event(), visibility = new Event(), closed = new Event()
  const posted = [], calls = [], diffs = []
  const repos = [0, 1].map((tab) => ({ tab, root: `/repo${tab}`, name: `repo${tab}`, refs: { refs: [] } }))
  const view = { visible: true, webview: { postMessage: (m) => posted.push(m), onDidReceiveMessage: messages.event }, onDidChangeVisibility: visibility.event, onDidDispose: closed.event }
  const detail = { id: 'commit', parents: ['parent', 'second-parent'], files: [{ status: 'R', path: 'new.txt', old_path: 'old.txt' }] }
  const core = { repos, onRepoChange: changed, onPrefsChange: prefs, revUri: (repo, path, rev, empty) => ({ root: repo.root, path, rev, empty }), engine: { call: async (...args) => { calls.push(args); return args[0] === 'detail' ? detail : [] } } }
  const vscode = { window: { registerWebviewViewProvider: (id, p) => { assert.equal(id, 'pushright.graph'); provider = p } }, commands: { registerCommand: () => ({ dispose() {} }), executeCommand: (...args) => diffs.push(args) } }
  load('graph-view', { vscode, './core': core, './webview': { setupWebview: () => ({ dispose() {} }) } }).registerGraphView({ subscriptions: [] })
  provider.resolveWebviewView(view)
  const send = async (cmd, args = {}) => { await Promise.all(messages.fire({ id: 1, cmd, args })); return posted.at(-1) }
  assert.deepEqual((await send('graph_repos')).value, repos)
  await send('rows', { tab: 1, start: 0, count: 128 })
  assert.deepEqual(calls, [['rows', { tab: 1, start: 0, count: 128 }]])
  assert.equal((await send('reveal_commit', { tab: 1, id: 'abc' })).ok, false)
  assert.equal((await send('rows', { tab: 9, start: 0, count: 128 })).ok, false)
  assert.equal((await send('rows', { tab: 1, start: 0, count: 100000 })).ok, false)
  assert.equal((await send('op', { tab: 1, op: { op: 'reset', mode: 'hard' } })).ok, false)
  assert.equal(calls.length, 1)
  assert.equal((await send('detail', { tab: 1, id: 'commit' })).value, detail)
  assert.deepEqual(diffs, [], 'expanding a commit never opens an editor')
  await send('open_commit_file', { tab: 1, id: 'commit', path: 'new.txt' })
  assert.deepEqual(diffs[0], ['vscode.diff', { root: '/repo1', path: 'old.txt', rev: 'parent', empty: false }, { root: '/repo1', path: 'new.txt', rev: 'commit', empty: false }, 'new.txt (commit)', { preview: true, preserveFocus: true }])
  for (const status of ['A', 'D']) {
    detail.files = [{ status, path: 'file.txt', old_path: null }]
    await send('open_commit_file', { tab: 0, id: 'commit', path: 'file.txt', preview: false })
    assert.equal(diffs.at(-1)[1].empty, status === 'A')
    assert.equal(diffs.at(-1)[2].empty, status === 'D')
    assert.deepEqual(diffs.at(-1)[4], { preview: false, preserveFocus: false })
  }
  detail.parents = []
  detail.files = [{ status: 'A', path: 'root.txt', old_path: null }]
  await send('open_commit_file', { tab: 0, id: 'commit', path: 'root.txt' })
  assert.equal(diffs.at(-1)[1].empty, true)
  assert.equal((await send('open_commit_file', { tab: 0, id: 'commit', path: '../../secret' })).ok, false)
  assert.equal(diffs.length, 4, 'paths absent from the commit cannot open')
  changed.fire(repos[0]); assert.deepEqual(posted.at(-1), { refresh: true })
  view.visible = false
  const n = posted.length
  changed.fire(repos[0]); assert.equal(posted.length, n)
  closed.fire()
  assert.equal(changed.listeners.length + prefs.listeners.length + messages.listeners.length + visibility.listeners.length, 0)
})

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
  const webviews = load('webview', { vscode, './core': core })
  load('panel', { vscode, './core': core, './webview': webviews }).openPanel({ extensionUri: uri('/extension') }, { id: 'older', root: 'repo' })
  const html = webview.html
  await Promise.all(messages.fire({ id: 1, cmd: 'open_repo', args: { path: 'repo' } }))
  await Promise.all(messages.fire({ id: 3, cmd: 'refs', args: { tab: 7 } }))
  assert.equal(posted.some((m) => m.reveal), false)
  await Promise.all(messages.fire({ id: 4, cmd: 'load_graph', args: { tab: 7, full: true } }))
  assert.deepEqual(posted.at(-1), { reveal: { id: 'older', root: 'repo' } })
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
      onDidChangeTextEditorSelection: new Event().event,
      createTreeView: (id, { treeDataProvider }) => { const view = { visible: id !== 'pushright.lineHistory', onDidChangeVisibility: new Event().event, provider: treeDataProvider }; views.set(id, view); return view },
    },
    workspace: { onDidChangeTextDocument: new Event().event, onDidSaveTextDocument: new Event().event },
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

test('line history follows selection without focus, coalesces reads and rejects stale results', async () => {
  const selection = new Event(), editorChanged = new Event(), visibility = new Event(), changed = new Event(), saved = new Event(), repoChanged = new Event()
  const views = new Map(), commands = new Map(), executed = [], calls = []
  const repo = { tab: 0, refs: { head_id: 'head' } }
  const range = (start, end = start, character = 2) => ({ start: { line: start, character: 0 }, end: { line: end, character } })
  const editor = { document: { uri: uri('/repo/a.txt'), isDirty: false }, selection: range(0) }
  const vscode = {
    EventEmitter: Event,
    window: {
      activeTextEditor: editor, onDidChangeActiveTextEditor: editorChanged.event, onDidChangeTextEditorSelection: selection.event,
      withProgress: (_, fn) => fn(),
      createTreeView: (id, { treeDataProvider }) => { const view = { visible: id === 'pushright.lineHistory', onDidChangeVisibility: visibility.event, provider: treeDataProvider }; views.set(id, view); return view },
    },
    workspace: { onDidChangeTextDocument: changed.event, onDidSaveTextDocument: saved.event },
    commands: { registerCommand: (id, fn) => commands.set(id, fn), executeCommand: (...args) => executed.push(args) },
  }
  const core = { onRepoChange: repoChanged, repoOf: () => repo, rel: () => 'a.txt', v: { lineHistoryLoading: 'loading' }, t: { errors: { PR_LINE_HISTORY_CHANGED: 'changed' } }, errorText: String,
    engine: { call: (cmd, args) => { const job = deferred(); calls.push({ cmd, args, job }); return job.promise } },
  }
  const context = { subscriptions: [] }
  load('history', { vscode, './core': core }).registerHistory(context)
  const pause = () => new Promise((r) => setTimeout(r, 160))
  const move = (r) => { editor.selection = r; selection.fire({ textEditor: editor }) }
  const list = views.get('pushright.lineHistory').provider
  try {
    await pause()
    assert.equal(calls[0].args.start, 1)
    move(range(1)); move(range(2, 3, 0)); await pause()
    assert.equal(calls.length, 1, 'wait for the existing query instead of launching parallel Git logs')
    calls[0].job.resolve([{ id: 'stale' }]); await pause()
    assert.deepEqual(list.items, [])
    assert.equal(calls.length, 2)
    assert.deepEqual([calls[1].args.start, calls[1].args.end], [3, 3])
    const latest = [{ id: 'latest', path: 'a.txt' }]
    calls[1].job.resolve(latest); await pause()
    assert.equal(list.items, latest)
    move(range(2)); await pause()
    assert.equal(calls.length, 2, 'same line does not reload')
    assert.deepEqual(executed, [], 'following the cursor must not focus the history pane')
    editor.document.isDirty = true; changed.fire({ document: editor.document }); await pause()
    assert.equal(list.view.message, 'changed')
    assert.deepEqual(list.items, [])
    assert.equal(calls.length, 2)
    editor.document.isDirty = false; saved.fire(editor.document); await pause()
    assert.equal(calls.length, 3)
    calls[2].job.resolve(latest); await pause()
    list.view.visible = false; visibility.fire(); move(range(5)); await pause()
    assert.equal(calls.length, 3, 'hidden views do not request history')
    list.view.visible = true; visibility.fire(); await pause()
    assert.equal(calls[3].args.start, 6)
    calls[3].job.resolve([]); await pause()
    repo.refs.head_id = 'new-head'; repoChanged.fire(repo); await pause()
    assert.equal(calls.length, 5)
    calls[4].job.resolve([])
  } finally { context.subscriptions.forEach((s) => s?.dispose?.()) }
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
