import { randomUUID } from 'node:crypto'
import * as vscode from 'vscode'
import { engine, locale, panel as hook, refresh, repos, store, stored } from './core'

/** 每个窗口各自决定的键，不跨窗口保存：页签来自工作区，主题和语言跟随 VS Code */
const SESSION = new Set(['tabs', 'recent', 'theme', 'locale'])
const WRITES = new Set(['op', 'stage', 'unstage', 'commit', 'apply_lines', 'discard_lines', 'conflict_resolve', 'conflict_take', 'set_commit_identity'])

let current: vscode.WebviewPanel | undefined

/** 打开提交图面板（界面就是桌面版的 src/）；给了 `reveal` 就跳到那个提交。 */
export function openPanel(context: vscode.ExtensionContext, reveal?: string) {
  if (current) {
    current.reveal()
    if (reveal) current.webview.postMessage({ reveal })
    return
  }
  const media = vscode.Uri.joinPath(context.extensionUri, 'media')
  current = vscode.window.createWebviewPanel('pushright', 'PushRight', vscode.ViewColumn.Active, {
    enableScripts: true,
    retainContextWhenHidden: true,
    localResourceRoots: [media],
  })
  const { webview } = current
  hook.post = (message) => void webview.postMessage(message)

  const render = () => {
    const nonce = randomUUID().replaceAll('-', '')
    const asset = (file: string) => webview.asWebviewUri(vscode.Uri.joinPath(media, file))
    const kind = vscode.window.activeColorTheme.kind
    const dark = kind === vscode.ColorThemeKind.Dark || kind === vscode.ColorThemeKind.HighContrast
    const seed = JSON.stringify({ ...stored(), theme: dark ? 'dark' : 'light', locale }).replace(/</g, '\\u003c')
    webview.html = `<!doctype html>
<html>
  <head>
    <meta charset="UTF-8" />
    <meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src ${webview.cspSource} data:; font-src ${webview.cspSource}; style-src ${webview.cspSource} 'unsafe-inline'; script-src 'nonce-${nonce}' ${webview.cspSource};" />
    <link rel="stylesheet" href="${asset('style.css')}" />
  </head>
  <body style="padding: 0">
    <div id="app"></div>
    <script nonce="${nonce}">
      localStorage.clear()
      for (const [key, value] of Object.entries(${seed})) localStorage.setItem(key, value)
    </script>
    <script type="module" nonce="${nonce}" src="${asset('index.js')}"></script>
  </body>
</html>`
  }
  render()
  const theme = vscode.window.onDidChangeActiveColorTheme(render)

  webview.onDidReceiveMessage(async (m) => {
    if ('store' in m) {
      if (!SESSION.has(m.store)) await store(m.store, m.value)
      return
    }
    try {
      const value =
        m.cmd === 'initial_repos' ? repos.map((r) => r.root)
        : m.cmd === 'pick_folder' ? ((await vscode.window.showOpenDialog({ canSelectFolders: true, canSelectFiles: false, title: m.args.title }))?.[0].fsPath ?? null)
        : await engine.call(m.cmd, m.args)
      webview.postMessage({ id: m.id, ok: true, value })
      // 界面读完引用说明已经起来了，这时再让它跳到指定提交
      if (reveal && m.cmd === 'refs') {
        webview.postMessage({ reveal })
        reveal = undefined
      }
    } catch (e) {
      webview.postMessage({ id: m.id, ok: false, value: String(e) })
    }
    // 面板里的会话与扩展主机的不是同一个，改动后全部刷新
    if (WRITES.has(m.cmd)) for (const repo of repos) refresh(repo)
  })
  current.onDidDispose(() => {
    theme.dispose()
    current = undefined
    hook.post = undefined
  })
}
