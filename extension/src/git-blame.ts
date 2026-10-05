import { spawn } from 'node:child_process'
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
 * 在 `root` 里对 `path` 做 blame。`contents` 是编辑器里尚未保存的内容，不给则用磁盘上的文件。
 * 每解析完一批区段调用一次 `onProgress`。返回的函数用来中途取消。
 */
export function runBlame(root: string, path: string, contents: string | undefined, onProgress: (blame: Blame) => void): () => void {
  const blame: Blame = { lines: [], done: false }
  const args = ['-C', root, 'blame', '--incremental']
  if (contents !== undefined) args.push('--contents', '-')
  const proc = spawn('git', [...args, '--', path], { windowsHide: true, stdio: ['pipe', 'pipe', 'ignore'] })
  proc.on('error', () => {})
  proc.stdin.on('error', () => {})
  proc.stdin.end(contents ?? '')
  const feed = blameParser(blame)
  let timer: ReturnType<typeof setTimeout> | undefined
  createInterface(proc.stdout).on('line', (line) => {
    feed(line)
    // 合并成每 50 毫秒最多刷新一次界面
    timer ??= setTimeout(() => ((timer = undefined), onProgress(blame)), 50)
  })
  proc.on('close', (code) => {
    clearTimeout(timer)
    if (code !== 0) return
    blame.done = true
    onProgress(blame)
  })
  return () => {
    clearTimeout(timer)
    proc.removeAllListeners('close')
    proc.kill()
  }
}
