import { invoke } from '@tauri-apps/api/core'

export interface Row {
  id: string
  lane: number
  incoming: number[]
  through: number[]
  out: number[]
  author: string
  time: number
  subject: string
}
export interface Ref {
  name: string
  id: string
  upstream: string | null
}
export interface Refs {
  head: string | null
  head_id: string | null
  /** 当前分支相对上游的 [领先, 落后] */
  ahead_behind: [number, number] | null
  in_progress: InProgress | null
  refs: Ref[]
}
export type InProgress = 'merge' | 'rebase' | 'cherry-pick' | 'revert'
export interface Log {
  command: string
  output: string
  ok: boolean
}
export type Op =
  | { op: 'checkout'; target: string }
  | { op: 'track'; remote_branch: string }
  | { op: 'create_branch'; name: string; start: string; checkout: boolean }
  | { op: 'delete_branch'; name: string; force: boolean }
  | { op: 'rename_branch'; old: string; new: string }
  | { op: 'merge'; target: string }
  | { op: 'rebase'; onto: string }
  | { op: 'cherry_pick'; id: string }
  | { op: 'revert'; id: string }
  | { op: 'reset'; target: string; mode: 'soft' | 'mixed' | 'hard' }
  | { op: 'stash_push'; message: string; include_untracked: boolean }
  | { op: 'stash_apply'; name: string; pop: boolean }
  | { op: 'stash_drop'; name: string }
  | { op: 'fetch' }
  | { op: 'pull' }
  | { op: 'push'; remote: string; branch: string; remote_branch: string; force: boolean; set_upstream: boolean }
  | { op: 'create_tag'; name: string; target: string; message: string }
  | { op: 'delete_tag'; name: string }
  | { op: 'continue'; what: InProgress }
  | { op: 'abort'; what: InProgress }
export interface Stash {
  name: string
  id: string
  subject: string
}
export interface Worktree {
  path: string
  head: string
  branch: string | null
}
export interface FileChange {
  status: string
  path: string
  old_path: string | null
}
export interface Detail {
  id: string
  parents: string[]
  author: string
  author_email: string
  author_time: number
  committer: string
  committer_time: number
  message: string
  files: FileChange[]
}

export interface Entry {
  path: string
  old_path: string | null
  staged: string | null
  unstaged: string | null
  conflicted: boolean
}
export interface Line {
  kind: string
  text: string
  old_no: number | null
  new_no: number | null
}
export interface Hunk {
  header: string
  lines: Line[]
}
export interface Diff {
  binary: boolean
  too_large: boolean
  hunks: Hunk[]
}

export const initialRepos = () => invoke<string[]>('initial_repos')
/** 返回 [页签 id, 工作区路径, 已加载的提交数] */
export const openRepo = (path: string) => invoke<[number, string, number]>('open_repo', { path })
export const closeRepo = (tab: number) => invoke<void>('close_repo', { tab })
export const loadGraph = (tab: number, full: boolean) => invoke<number>('load_graph', { tab, full })
export const rows = (tab: number, start: number, count: number) => invoke<Row[]>('rows', { tab, start, count })
export const rowOf = (tab: number, id: string) => invoke<number | null>('row_of', { tab, id })
export const refs = (tab: number) => invoke<Refs>('refs', { tab })
export const stashes = (tab: number) => invoke<Stash[]>('stashes', { tab })
export const worktrees = (tab: number) => invoke<Worktree[]>('worktrees', { tab })
export const detail = (tab: number, id: string) => invoke<Detail>('detail', { tab, id })
export const status = (tab: number) => invoke<Entry[]>('status', { tab })
export const stage = (tab: number, paths: string[]) => invoke<void>('stage', { tab, paths })
export const unstage = (tab: number, paths: string[]) => invoke<void>('unstage', { tab, paths })
export const commit = (tab: number, message: string, amend: boolean) => invoke<void>('commit', { tab, message, amend })
export const lastMessage = (tab: number) => invoke<string>('last_message', { tab })
export const diffWorktree = (tab: number, path: string, staged: boolean, untracked: boolean) =>
  invoke<Diff>('diff_worktree', { tab, path, staged, untracked })
export const diffCommit = (tab: number, id: string, path: string) => invoke<Diff>('diff_commit', { tab, id, path })
export const applyLines = (tab: number, path: string, staged: boolean, hunk: number, header: string, lines: number[]) =>
  invoke<void>('apply_lines', { tab, path, staged, hunk, header, lines })
export const op = (tab: number, op: Op) => invoke<Log>('op', { tab, op })
export const remotes = (tab: number) => invoke<string[]>('remotes', { tab })
