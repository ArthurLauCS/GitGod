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

export const initialRepo = () => invoke<string | null>('initial_repo')
export const openRepo = (path: string) => invoke<[string, number]>('open_repo', { path })
export const loadGraph = (full: boolean) => invoke<number>('load_graph', { full })
export const rows = (start: number, count: number) => invoke<Row[]>('rows', { start, count })
export const rowOf = (id: string) => invoke<number | null>('row_of', { id })
export const refs = () => invoke<Refs>('refs')
export const stashes = () => invoke<Stash[]>('stashes')
export const worktrees = () => invoke<Worktree[]>('worktrees')
export const detail = (id: string) => invoke<Detail>('detail', { id })
