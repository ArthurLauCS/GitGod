// 由 tests/vscode-smoke.mjs 启动，在 VS Code 扩展主机里执行。
const assert = require('node:assert/strict')
const { execFileSync } = require('node:child_process')
const { writeFileSync } = require('node:fs')
const { join } = require('node:path')
const vscode = require('vscode')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
async function until(what, check, timeout = 20000) {
  for (const start = Date.now(); Date.now() - start < timeout; await sleep(200)) {
    const value = await check()
    if (value) return value
  }
  throw new Error(`timed out waiting for ${what}`)
}

exports.run = async () => {
  const passed = []
  let error
  try {
    const root = vscode.workspace.workspaceFolders[0].uri.fsPath
    const git = (...args) => execFileSync('git', ['-C', root, ...args], { encoding: 'utf8' }).trim()
    const file = vscode.Uri.file(join(root, 'src.ts'))
    const step = (name) => passed.push(name)

    const extension = vscode.extensions.getExtension('pushright.pushright')
    const api = await extension.activate()
    assert.equal(api.repos.length, 1)
    assert.equal(api.repos[0].refs.head, 'refs/heads/feature/x')
    step('activates and opens the workspace repository')

    const registered = new Set(await vscode.commands.getCommands())
    const missing = extension.packageJSON.contributes.commands.map((c) => c.command).filter((c) => !registered.has(c))
    assert.deepEqual(missing, [])
    step(`registers all ${extension.packageJSON.contributes.commands.length} contributed commands`)

    // Pull：选择框里「Rebase instead of merge」默认勾选，直接确认应当是变基
    const pulling = vscode.commands.executeCommand('pushright.pull')
    await sleep(1500)
    await vscode.commands.executeCommand('workbench.action.acceptSelectedQuickOpenItem')
    await pulling
    assert.equal(git('log', '--format=%s', '-3'), 'local commit\ncolleague commit\nraise value10')
    assert.equal(git('log', '--merges', '--format=%s'), '')
    step('pull with the default choice rebases instead of merging')

    const head = await vscode.workspace.openTextDocument(file.with({ scheme: 'pushright-rev', query: JSON.stringify({ rev: 'HEAD' }) }))
    assert.equal(head.getText(), git('show', 'HEAD:src.ts') + '\n')
    step('serves file contents at a revision')

    const editor = await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(file))
    const lens = await until('blame code lens', async () => {
      const lenses = await vscode.commands.executeCommand('vscode.executeCodeLensProvider', file)
      return lenses?.find((l) => l.command?.command === 'pushright.fileHistory' && /^bob, /.test(l.command.title))
    })
    assert.match(lens.command.title, /^bob, /)
    step('blames the file and shows the latest author as a code lens')

    // 改两处，只选中第一处暂存
    const text = editor.document.getText().split('\n')
    text[0] = 'export const value0 = -1'
    text[20] = 'export const value20 = -20'
    writeFileSync(file.fsPath, text.join('\n'))
    await until('editor reload', () => editor.document.lineAt(0).text.endsWith('-1'))
    editor.selection = new vscode.Selection(0, 0, 0, 5)
    await vscode.commands.executeCommand('pushright.stageSelection')
    assert.match(git('diff', '--cached'), /\+export const value0 = -1/)
    assert.doesNotMatch(git('diff', '--cached'), /value20 = -20/)
    assert.match(git('diff'), /\+export const value20 = -20/)
    step('stages only the selected lines')

    await until('status refresh', () => api.repos[0].status.some((e) => e.staged))
    await vscode.commands.executeCommand('pushright.unstageAll')
    assert.equal(git('diff', '--cached', '--name-only'), '')
    await vscode.commands.executeCommand('pushright.stageAll')
    assert.equal(git('diff', '--cached', '--name-only'), 'src.ts')
    await vscode.commands.executeCommand('pushright.commitAmend')
    assert.equal(git('status', '--porcelain'), '')
    assert.equal(git('log', '--format=%s', '-1'), 'local commit')
    step('stages, unstages and amends through the source control commands')

    await vscode.commands.executeCommand('pushright.previousRevision')
    const diffOf = () => vscode.window.tabGroups.activeTabGroup.activeTab?.input
    await until('diff with HEAD', () => diffOf() instanceof vscode.TabInputTextDiff && diffOf().modified.scheme === 'file')
    const latest = JSON.parse(diffOf().original.query).rev
    await vscode.commands.executeCommand('pushright.previousRevision')
    await until('diff of the latest commit', () => diffOf() instanceof vscode.TabInputTextDiff && JSON.parse(diffOf().modified.query || '{}').rev === latest)
    await vscode.commands.executeCommand('pushright.previousRevision')
    await until('diff of an older commit', () => diffOf() instanceof vscode.TabInputTextDiff && ![latest, undefined].includes(JSON.parse(diffOf().modified.query || '{}').rev))
    step('steps back through the revisions of a file')

    // 面板里的界面起来后会自己在引擎里打开仓库，会话号紧接主机的 0
    await vscode.commands.executeCommand('pushright.openGraph')
    await until('commit graph panel', () => api.engine.call('refs', { tab: 1 }).then(() => true, () => false))
    assert.ok(vscode.window.tabGroups.all.some((g) => g.tabs.some((tab) => tab.input instanceof vscode.TabInputWebview)))
    step('commit graph panel loads and talks to the engine')

    if (process.env.PUSHRIGHT_SMOKE_SHOT) {
      await sleep(3000)
      execFileSync('powershell', ['-NoProfile', '-Command', `Add-Type -AssemblyName System.Windows.Forms,System.Drawing; $b=[System.Windows.Forms.Screen]::PrimaryScreen.Bounds; $i=New-Object System.Drawing.Bitmap $b.Width,$b.Height; [System.Drawing.Graphics]::FromImage($i).CopyFromScreen($b.Location,[System.Drawing.Point]::Empty,$b.Size); $i.Save('${process.env.PUSHRIGHT_SMOKE_SHOT}')`])
    }
  } catch (e) {
    error = e.stack ?? String(e)
  }
  writeFileSync(process.env.PUSHRIGHT_SMOKE_RESULT, JSON.stringify({ passed, error }))
  if (error) throw new Error(error)
}
