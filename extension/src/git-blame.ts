import { execFile, spawn } from 'node:child_process'
import { createInterface } from 'node:readline'

// blame 不走 sidecar：Linux 仓库 1.1 万行的文件整份要 3 秒，流式前 200 行只要 140 毫秒，
// 所以边读边显示，文件一改就杀掉进程重来。数据见 docs/PERFORMANCE.md。

export interface BlameCommit {
  id: string
  author: string
  author_email: string
  time: number
  subject: string
  /** 这一行在该提交里所属的文件（重命名之前是旧名字） */
  path: string
  /** 该提交之前的版本：`<提交> <路径>`；文件在这个提交里新建时没有 */
  previous?: string
}

export interface Blame {
  /** 下标是行号减一；还没算到的行为 undefined */
  lines: (BlameCommit | undefined)[]
  done: boolean
}

/** 尚未提交的行，git 用全零的提交号表示 */
export const isUncommitted = (c: BlameCommit) => /^0+$/.test(c.id)

/** 逐行喂入 `git blame --incremental` 的输出。 */
export function blameParser(blame: Blame) {
  const commits = new Map<string, BlameCommit>()
  let current: BlameCommit | undefined
  let start = 0
  let count = 0
  return (line: string) => {
    const head = /^([0-9a-f]{40}) \d+ (\d+) (\d+)$/.exec(line)
    if (head) {
      current = commits.get(head[1])
      if (!current) commits.set(head[1], (current = { id: head[1], author: '', author_email: '', time: 0, subject: '', path: '' }))
      start = +head[2] - 1
      count = +head[3]
      return
    }
    if (!current) return
    const space = line.indexOf(' ')
    const value = line.slice(space + 1)
    switch (line.slice(0, space)) {
      case 'author': current.author = value; break
      case 'author-mail': current.author_email = value.replace(/^<|>$/g, ''); break
      case 'author-time': current.time = +value; break
      case 'summary': current.subject = value; break
      case 'previous': current.previous = value; break
      case 'filename':
        // 每个区段以 filename 结尾
        current.path ||= value
        for (let i = start; i < start + count; i++) blame.lines[i] = current
    }
  }
}

/**
 * `rev` 指定历史版本；不传时使用 `contents`（编辑器/暂存区快照）或工作区文件。
 * 每解析完一批区段调用一次 `onProgress`。返回的函数用来中途取消。
 */
export function runBlame(root: string, path: string, contents: string | undefined, onProgress: (blame: Blame) => void, rev?: string, onError: (message: string) => void = () => {}): () => void {
  if (rev?.startsWith('-')) return () => {}
  const args = ['-C', root, '-c', 'core.quotepath=false', 'blame', '--incremental']
  if (rev) args.push(rev)
  else if (contents !== undefined) args.push('--contents', '-')
  let stopped = false, cancel = () => {}
  function start(file: string, retry: boolean) {
    const blame: Blame = { lines: [], done: false }
    const proc = spawn('git', [...args, '--', file], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] })
    let stderr = ''
    proc.stderr.setEncoding('utf8').on('data', (chunk: string) => { stderr = (stderr + chunk).slice(0, 8192) })
    proc.on('error', (error) => { stderr = error.message })
    proc.stdin.on('error', () => {})
    proc.stdin.end(contents ?? '')
    const feed = blameParser(blame)
    let timer: ReturnType<typeof setTimeout> | undefined
    const reader = createInterface(proc.stdout).on('line', (line) => {
      feed(line)
      // 合并成每 50 毫秒最多刷新一次界面
      timer ??= setTimeout(() => ((timer = undefined), onProgress(blame)), 50)
    })
    cancel = () => {
      reader.removeAllListeners('line')
      reader.close()
      clearTimeout(timer)
      proc.kill()
    }
    proc.on('close', (code) => {
      clearTimeout(timer)
      if (stopped) return
      if (code === 0) {
        blame.done = true
        onProgress(blame)
        return
      }
      const failure = `git blame (${rev || 'working tree'}) -- ${file}: ${stderr.trim() || `exit ${code}`}`
      if (!retry || process.platform !== 'win32' || !stderr.includes('no such path')) return onError(failure)
      // Windows 编辑器可能保留改名前的大小写；历史 diff 必须查对应提交，不能查磁盘。
      const lookup = rev ? ['ls-tree', '-r', '--name-only', '-z', rev] : ['ls-files', '--cached', '-z', '--', `:(icase,literal)${file}`]
      const child = execFile('git', ['-C', root, ...lookup], { windowsHide: true, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 }, (error, stdout) => {
        if (stopped) return
        if (error) return onError(`${failure}\n${error.message}`)
        const matches = [...new Set(stdout.split('\0').filter((name) => name.toLowerCase() === file.toLowerCase()))]
        if (matches.length === 1 && matches[0] !== file) start(matches[0], false)
        else onError(failure)
      })
      cancel = () => { child.kill() }
    })
  }
  start(path, true)
  return () => { stopped = true; cancel() }
}
