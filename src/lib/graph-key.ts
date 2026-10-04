import type { Refs } from './api'

/** 图只依赖提交起点集合；引用改名、已有分支间切换只更新标记和选中行。 */
export function graphKey(refs: Pick<Refs, 'refs' | 'head_id'>): string {
  return JSON.stringify([...new Set([...refs.refs.map((r) => r.id), ...(refs.head_id ? [refs.head_id] : [])])].sort())
}
