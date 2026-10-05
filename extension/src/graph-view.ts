import * as vscode from 'vscode'
import { engine, onPrefsChange, onRepoChange, repos, store, stored } from './core'
import { openPanel } from './panel'
import { setupWebview } from './webview'

/** 原生历史图仍需要 proposed API；使用稳定的 WebviewView 放在相同的 SCM 容器内。 */
export function registerGraphView(context: vscode.ExtensionContext) {
  context.subscriptions.push(vscode.window.registerWebviewViewProvider('pushright.graph', {
    resolveWebviewView(view) {
      const { webview } = view
      const changed = () => { if (view.visible) webview.postMessage({ refresh: true }) }
      const subscriptions = [
        setupWebview(context, webview, 'sidebar.js'),
        onRepoChange.event(changed),
        view.onDidChangeVisibility(changed),
        onPrefsChange.event(() => webview.postMessage({ store: 'prefs', value: stored().prefs ?? '{}' })),
        webview.onDidReceiveMessage(async (m) => {
          if (m.store === 'prefs') return void await store(m.store, m.value)
          if (!m.cmd) return
          try {
            let value: unknown
            if (m.cmd === 'graph_repos') {
              value = repos.map(({ tab, root, name, refs }) => ({ tab, root, name, refs }))
            } else {
              const repo = repos.find((r) => r.tab === m.args?.tab)
              if (!repo) throw new Error('PR_TAB_CLOSED')
              if (m.cmd === 'reveal_commit' && typeof m.args.id === 'string') {
                openPanel(context, { id: m.args.id, root: repo.root })
              } else if (m.cmd === 'load_graph' && typeof m.args.full === 'boolean') {
                value = await engine.call(m.cmd, { tab: repo.tab, full: m.args.full })
              } else if (m.cmd === 'rows' && Number.isInteger(m.args.start) && m.args.start >= 0 && Number.isInteger(m.args.count) && m.args.count > 0 && m.args.count <= 128) {
                value = await engine.call(m.cmd, { tab: repo.tab, start: m.args.start, count: m.args.count })
              } else throw new Error('Unsupported graph request')
            }
            webview.postMessage({ id: m.id, ok: true, value })
          } catch (error) {
            webview.postMessage({ id: m.id, ok: false, value: String(error) })
          }
        }),
      ]
      view.onDidDispose(() => subscriptions.forEach((s) => s.dispose()))
    },
  }))
}
