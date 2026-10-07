// 在真实的 VS Code 里跑一遍扩展：npm run ext:smoke（先 npm run ext:build）。
// 用临时的用户目录和扩展目录，不碰本机设置。PUSHRIGHT_SMOKE_EXTENSION 指定解包后的 VSIX，PUSHRIGHT_SMOKE_CODE 可指定 Cursor 可执行文件。
import { execFileSync, spawnSync } from 'node:child_process'
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'

const base = mkdtempSync(join(tmpdir(), 'pushright-smoke-'))
const work = join(base, 'work')
const git = (dir, ...args) => execFileSync('git', ['-C', dir, '-c', 'commit.gpgsign=false', '-c', 'core.autocrlf=false', ...args], { stdio: 'pipe' })
const as = (name) => ['-c', `user.name=${name}`, '-c', `user.email=${name}@example.com`]

// 远程 origin、本地 work、同事的 other。本地分支 feature/x 误跟踪 origin/main，并且双方各有一个新提交
git(base, 'init', '-q', '--bare', '-b', 'main', 'origin.git')
git(base, 'clone', '-q', 'origin.git', 'work')
git(work, 'config', 'user.name', 'carol')
git(work, 'config', 'user.email', 'carol@example.com')
const lines = Array.from({ length: 30 }, (_, i) => `export const value${i} = ${i}`)
writeFileSync(join(work, 'src.ts'), lines.join('\n') + '\n')
git(work, 'add', '.')
// 两次提交相隔不到一秒，显式给出作者时间，「最近修改者」才有确定的答案
git(work, ...as('alice'), 'commit', '-q', '--date=2024-01-01T00:00:00', '-m', 'add values')
lines[10] = 'export const value10 = 100'
writeFileSync(join(work, 'src.ts'), lines.join('\n') + '\n')
git(work, ...as('bob'), 'commit', '-q', '--date=2024-06-01T00:00:00', '-am', 'raise value10')
git(work, 'push', '-q', 'origin', 'main')
git(base, 'clone', '-q', 'origin.git', 'other')
writeFileSync(join(base, 'other', 'theirs.txt'), 'from a colleague\n')
git(join(base, 'other'), 'add', '.')
git(join(base, 'other'), ...as('dave'), 'commit', '-q', '-m', 'colleague commit')
git(join(base, 'other'), 'push', '-q', 'origin', 'main')
git(work, 'checkout', '-q', '-b', 'feature/x', '--track', 'origin/main')
writeFileSync(join(work, 'mine.txt'), 'local work\n')
git(work, 'add', '.')
git(work, 'commit', '-q', '-m', 'local commit')

const user = join(base, 'user')
mkdirSync(join(user, 'User'), { recursive: true })
writeFileSync(join(user, 'User', 'settings.json'), JSON.stringify({
  // 测试环境里模态对话框弹不出来，先关掉自带 Git，跳过接管引导
  'git.enabled': false,
  'window.newWindowDimensions': 'maximized',
  'workbench.startupEditor': 'none',
  'security.workspace.trust.enabled': false,
}))

const where = spawnSync(process.platform === 'win32' ? 'where' : 'which', ['code'], { encoding: 'utf8' }).stdout.split(/\r?\n/)[0]
const code = process.env.PUSHRIGHT_SMOKE_CODE ?? (process.platform === 'win32' ? join(dirname(dirname(where)), 'Code.exe') : where)
if (!existsSync(code)) throw new Error('VS Code not found on PATH')

const result = join(base, 'result.json')
const extension = process.env.PUSHRIGHT_SMOKE_EXTENSION ?? join(import.meta.dirname, '../extension')
const run = spawnSync(code, [
  work,
  `--extensionDevelopmentPath=${extension}`,
  `--extensionTestsPath=${join(import.meta.dirname, 'vscode-smoke-host.cjs')}`,
  `--user-data-dir=${user}`,
  `--extensions-dir=${join(base, 'extensions')}`,
  '--disable-extensions', '--disable-updates', '--skip-welcome', '--skip-release-notes', '--new-window',
], { env: { ...process.env, PUSHRIGHT_SMOKE_RESULT: result, PUSHRIGHT_SMOKE_ID: JSON.parse(readFileSync(join(extension, 'package.json'), 'utf8')).publisher + '.pushright' }, stdio: 'inherit' })

const report = existsSync(result) ? JSON.parse(readFileSync(result, 'utf8')) : { error: `VS Code exited with ${run.status} before reporting` }
for (const step of report.passed ?? []) console.log(`ok  ${step}`)
if (report.error) {
  console.error(`FAILED ${report.error}`)
  process.exit(1)
}
