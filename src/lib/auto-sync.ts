export interface AutoSync { refresh: boolean; fetch: boolean }

export function loadAutoSync(storage: Pick<Storage, 'getItem'>): AutoSync {
  try {
    const saved = JSON.parse(storage.getItem('autoSync') ?? '{}')
    return { refresh: saved?.refresh !== false, fetch: saved?.fetch !== false }
  } catch { return { refresh: true, fetch: true } }
}

/** 每个已打开的桌面仓库一个调度器；不依赖焦点，不补跑休眠期间错过的任务。 */
export function startAutoSync(actions: {
  settings: () => AutoSync
  ready: () => boolean
  refresh: () => Promise<void>
  fetch: () => Promise<void>
  error: (error: unknown) => void
}) {
  let stopped = false, running = false, lastFetch = -Infinity, fetchEnabled = false
  async function tick() {
    if (stopped || running || !actions.ready()) return
    const settings = actions.settings()
    if (settings.fetch && !fetchEnabled) lastFetch = -Infinity
    fetchEnabled = settings.fetch
    const fetch = settings.fetch && Date.now() - lastFetch >= 600_000
    if (!settings.refresh && !fetch) return
    running = true
    try {
      if (fetch) {
        lastFetch = Date.now()
        try { await actions.fetch() } catch (e) { actions.error(e) }
      }
      if (!stopped) await actions.refresh()
    } catch (e) { actions.error(e) }
    finally { running = false }
  }
  const timer = setInterval(() => void tick(), 60_000)
  queueMicrotask(() => void tick())
  return { tick, stop: () => { stopped = true; clearInterval(timer) } }
}
