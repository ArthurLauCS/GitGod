export type NamedRef = { name: string }

/** 最近使用的引用排前；未使用过的引用保留原始顺序。 */
export function orderRefs<T extends NamedRef>(refs: T[], recent: string[], current?: string | null): T[] {
  const rank = new Map(recent.map((name, index) => [name, index]))
  return refs
    .map((ref, index) => ({ ref, index }))
    .sort((a, b) => {
      const currentOrder = Number(b.ref.name === current) - Number(a.ref.name === current)
      if (currentOrder) return currentOrder
      const ar = rank.get(a.ref.name) ?? Number.MAX_SAFE_INTEGER
      const br = rank.get(b.ref.name) ?? Number.MAX_SAFE_INTEGER
      return ar - br || a.index - b.index
    })
    .map(({ ref }) => ref)
}

export function rememberRef(recent: string[], name: string, limit = 30): string[] {
  return [name, ...recent.filter((ref) => ref !== name)].slice(0, limit)
}

export function readRecentRefs(value: string | null | undefined, repo: string): string[] {
  try {
    const parsed = JSON.parse(value ?? '{}')
    return Array.isArray(parsed?.[repo]) ? parsed[repo].filter((name: unknown) => typeof name === 'string') : []
  } catch {
    return []
  }
}

export function writeRecentRefs(value: string | null | undefined, repo: string, recent: string[]): string {
  let parsed: Record<string, string[]> = {}
  try {
    const candidate = JSON.parse(value ?? '{}')
    if (candidate && typeof candidate === 'object' && !Array.isArray(candidate)) parsed = candidate
  } catch {
    // 损坏的偏好直接重建，不影响分支操作。
  }
  parsed[repo] = recent
  return JSON.stringify(parsed)
}
