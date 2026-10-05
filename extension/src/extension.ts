import { existsSync } from 'node:fs'
import { join } from 'node:path'
import * as vscode from 'vscode'
import { registerBlame } from './blame'
import { activeRepo, engine, log, onPrefsChange, onRepoChange, openRepos, repos, v } from './core'
import { registerHistory } from './history'
import { registerGraphView } from './graph-view'
import { openPanel } from './panel'
import { offerTakeover, registerScm, repoArg } from './scm'
import { checkout, fetch, inProgress, pull, push } from './sync'

export async function activate(context: vscode.ExtensionContext) {
  const exe = join(context.extensionPath, 'bin', process.platform === 'win32' ? 'pushright-engine.exe' : 'pushright-engine')
  // 引擎按平台打包，目前只有 Windows x64
  if (!existsSync(exe)) return void vscode.window.showErrorMessage(v.unsupported)
  await openRepos(context, exe)
  await vscode.commands.executeCommand('setContext', 'pushright.hasRepo', repos.length > 0)

  const command = (id: string, fn: (...args: any[]) => unknown) => vscode.commands.registerCommand(`pushright.${id}`, fn)
  // 参数是状态栏传来的仓库或面板标题栏传来的 SourceControl；从命令面板执行时没有参数，取当前文件所在的仓库
  const onRepo = (fn: (repo: NonNullable<ReturnType<typeof activeRepo>>) => unknown) => (arg: unknown) => {
    const repo = repoArg(arg) ?? activeRepo()
    return repo && fn(repo)
  }
  context.subscriptions.push(
    log, onRepoChange, onPrefsChange,
    { dispose: () => engine.dispose() },
    command('openGraph', () => openPanel(context)),
    command('revealCommit', (id: string, root?: string) => openPanel(context, { id, root: root ?? activeRepo()?.root })),
    command('pull', onRepo(pull)),
    command('push', onRepo(push)),
    command('fetch', onRepo(fetch)),
    command('checkout', onRepo(checkout)),
    command('inProgress', onRepo(inProgress)),
    command('showLog', () => log.show()),
  )
  registerScm(context)
  registerGraphView(context)
  registerBlame(context)
  registerHistory(context)
  if (repos.length) offerTakeover(context)
  // 供 tests/vscode-smoke-host.cjs 使用
  return { engine, repos }
}
