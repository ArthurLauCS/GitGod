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
}
export interface Refs {
  head: string | null
  head_id: string | null
  refs: Ref[]
}
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
