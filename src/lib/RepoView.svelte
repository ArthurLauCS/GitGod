<script lang="ts">
  import { untrack } from 'svelte'
  import * as api from './api'
  import Detail from './Detail.svelte'
  import Graph from './Graph.svelte'
  import Sidebar from './Sidebar.svelte'
  import WorkingCopy from './WorkingCopy.svelte'
  import { t } from './zh'

  let { tab, initialCount, active }: { tab: number; initialCount: number; active: boolean } = $props()

  let count = $state(untrack(() => initialCount))
  let version = $state(0)
  let loadingAll = $state(false)
  let error = $state('')
  let refs = $state.raw<api.Refs>({ head: null, head_id: null, refs: [] })
  let stashes = $state.raw<api.Stash[]>([])
  let worktrees = $state.raw<api.Worktree[]>([])
  let entries = $state.raw<api.Entry[]>([])
  let view = $state<'history' | 'changes'>('history')
  let selectedRow = $state<number | null>(null)
  let detail = $state.raw<api.Detail | null>(null)
  let graph = $state<Graph>()
  let pendingJump: string | null = null

  const badges = $derived.by(() => {
    const map = new Map<string, api.Ref[]>()
    for (const r of refs.refs) map.set(r.id, [...(map.get(r.id) ?? []), r])
    return map
  })
  const headLabel = $derived(refs.head?.startsWith('refs/heads/') ? refs.head.slice(11) : t.detached)

  async function guard<T>(p: Promise<T>): Promise<T | undefined> {
    try {
      return await p
    } catch (e) {
      error = String(e)
    }
  }

  async function loadStatus() {
    entries = (await guard(api.status(tab))) ?? entries
  }

  async function loadSidebar() {
    const r = await guard(Promise.all([api.refs(tab), api.stashes(tab), api.worktrees(tab), loadStatus()]))
    if (r) [refs, stashes, worktrees] = r
  }

  async function loadAll() {
    loadingAll = true
    const n = await guard(api.loadGraph(tab, true))
    loadingAll = false
    if (n === undefined) return
    count = n
    version++
    if (pendingJump) jump(pendingJump)
    pendingJump = null
  }

  async function select(row: number, id: string) {
    selectedRow = row
    const d = await guard(api.detail(tab, id))
    if (d && selectedRow === row) detail = d
  }

  async function jump(id: string) {
    const row = await guard(api.rowOf(tab, id))
    if (row == null) {
      // 完整历史还没加载完，目标提交不在首批里：等加载完再跳
      if (loadingAll) pendingJump = id
      return
    }
    view = 'history'
    graph?.scrollToRow(row, true)
    select(row, id)
  }

  // 回到窗口或切回本页签时，引用有变化才重新加载提交图
  async function refresh() {
    if (loadingAll) return
    const before = JSON.stringify(refs)
    await loadSidebar()
    if (JSON.stringify(refs) === before) return
    selectedRow = null
    detail = null
    await loadAll()
  }

  let started = false
  $effect(() => {
    if (!active) return
    if (started) {
      untrack(refresh)
      return
    }
    // 首次激活才加载完整历史，恢复多个页签时不会同时全量加载
    started = true
    untrack(async () => {
      await loadSidebar()
      if (refs.head_id) jump(refs.head_id)
      await loadAll()
    })
  })
</script>

<svelte:window onfocus={() => active && started && refresh()} />

<div class="repo" class:hidden={!active}>
  <header>
    <span class="branch">{headLabel}</span>
    <span class="muted">{loadingAll ? t.loadingHistory : t.commits(count)}</span>
  </header>
  {#if error}<p class="error">{error}</p>{/if}
  <main>
    <Sidebar {refs} {stashes} {worktrees} {view} changes={entries.length} onview={(v) => (view = v)} onjump={jump} />
    <div class="content">
      <div class="pane" class:hidden={view !== 'history'}>
        {#if count}
          <Graph
            bind:this={graph}
            fetchRows={(start, n) => api.rows(tab, start, n)}
            {count}
            {version}
            {badges}
            headId={refs.head_id}
            {selectedRow}
            onselect={select}
          />
        {:else}
          <p class="none">{t.noCommits}</p>
        {/if}
        <Detail {detail} onjump={jump} fetchDiff={(id, path) => api.diffCommit(tab, id, path)} />
      </div>
      <div class="pane" class:hidden={view !== 'changes'}>
        <WorkingCopy {tab} {entries} reload={loadStatus} oncommitted={refresh} />
      </div>
    </div>
  </main>
</div>

<style>
  .repo {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  /* 不用 display: none：那样会丢掉提交图的滚动位置 */
  .hidden {
    position: absolute;
    inset: 0;
    visibility: hidden;
  }
  header {
    flex: none;
    display: flex;
    align-items: center;
    gap: 12px;
    height: 36px;
    padding: 0 14px;
    background: var(--panel);
    border-bottom: 1px solid var(--border);
  }
  .branch {
    padding: 1px 8px;
    border-radius: 4px;
    color: var(--accent);
    background: var(--accent-soft);
    font-size: 12px;
  }
  .muted {
    color: var(--muted);
    font-size: 12px;
  }
  main {
    flex: 1;
    min-height: 0;
    display: flex;
  }
  .content {
    flex: 1;
    min-width: 0;
    position: relative;
    display: flex;
  }
  .pane {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .none {
    flex: 1;
    display: grid;
    place-items: center;
    margin: 0;
    color: var(--muted);
  }
  .error {
    margin: 0;
    padding: 6px 14px;
    color: var(--red);
    background: color-mix(in srgb, var(--red) 12%, transparent);
    user-select: text;
  }
</style>
