<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog'
  import * as api from './lib/api'
  import Detail from './lib/Detail.svelte'
  import Graph from './lib/Graph.svelte'
  import Sidebar from './lib/Sidebar.svelte'
  import { t } from './lib/zh'

  let recent = $state<string[]>(JSON.parse(localStorage.getItem('recent') ?? '[]'))
  let repoPath = $state<string | null>(null)
  let count = $state(0)
  let version = $state(0)
  let loadingAll = $state(false)
  let error = $state('')
  let refs = $state.raw<api.Refs>({ head: null, head_id: null, refs: [] })
  let stashes = $state.raw<api.Stash[]>([])
  let worktrees = $state.raw<api.Worktree[]>([])
  let selectedRow = $state<number | null>(null)
  let detail = $state.raw<api.Detail | null>(null)
  let graph = $state<Graph>()
  let pendingJump: string | null = null

  const badges = $derived.by(() => {
    const map = new Map<string, api.Ref[]>()
    for (const r of refs.refs) map.set(r.id, [...(map.get(r.id) ?? []), r])
    return map
  })
  const repoName = $derived(repoPath?.split(/[\\/]/).filter(Boolean).pop() ?? '')
  const headLabel = $derived(refs.head?.startsWith('refs/heads/') ? refs.head.slice(11) : t.detached)

  async function guard<T>(p: Promise<T>): Promise<T | undefined> {
    try {
      return await p
    } catch (e) {
      error = String(e)
    }
  }

  async function loadSidebar() {
    const r = await guard(Promise.all([api.refs(), api.stashes(), api.worktrees()]))
    if (r) [refs, stashes, worktrees] = r
  }

  async function loadAll() {
    loadingAll = true
    const n = await guard(api.loadGraph(true))
    loadingAll = false
    if (n === undefined) return
    count = n
    version++
    if (pendingJump) jump(pendingJump)
    pendingJump = null
  }

  async function openRepo(path: string) {
    error = ''
    const r = await guard(api.openRepo(path))
    if (!r) return
    ;[repoPath, count] = r
    version++
    selectedRow = null
    detail = null
    recent = [repoPath, ...recent.filter((p) => p !== repoPath)].slice(0, 10)
    localStorage.setItem('recent', JSON.stringify(recent))
    await loadSidebar()
    if (refs.head_id) jump(refs.head_id)
    await loadAll()
  }

  async function pick() {
    const dir = await open({ directory: true })
    if (dir) openRepo(dir)
  }

  async function select(row: number, id: string) {
    selectedRow = row
    const d = await guard(api.detail(id))
    if (d && selectedRow === row) detail = d
  }

  async function jump(id: string) {
    const row = await guard(api.rowOf(id))
    if (row == null) {
      // 完整历史还没加载完，目标提交不在首批里：等加载完再跳
      if (loadingAll) pendingJump = id
      return
    }
    graph?.scrollToRow(row, true)
    select(row, id)
  }

  api.initialRepo().then((p) => {
    if (p) openRepo(p)
  })

  // 回到窗口时，引用有变化才重新加载提交图
  async function onfocus() {
    if (!repoPath || loadingAll) return
    const before = JSON.stringify(refs)
    await loadSidebar()
    if (JSON.stringify(refs) === before) return
    selectedRow = null
    detail = null
    await loadAll()
  }
</script>

<svelte:window {onfocus} />

{#if repoPath}
  <header>
    <strong>{repoName}</strong>
    <span class="branch">{headLabel}</span>
    <span class="muted">{loadingAll ? t.loadingHistory : t.commits(count)}</span>
    <span class="spacer"></span>
    <button onclick={pick}>{t.openRepo}</button>
  </header>
  {#if error}<p class="error">{error}</p>{/if}
  <main>
    <Sidebar {refs} {stashes} {worktrees} onjump={jump} />
    <div class="center">
      {#if count}
        <Graph bind:this={graph} {count} {version} {badges} headId={refs.head_id} {selectedRow} onselect={select} />
      {:else}
        <p class="none">{t.noCommits}</p>
      {/if}
      <Detail {detail} onjump={jump} />
    </div>
  </main>
{:else}
  <div class="welcome">
    <h1>{t.appName}</h1>
    <p class="muted">{t.openRepoHint}</p>
    <button class="primary" onclick={pick}>{t.openRepo}</button>
    {#if error}<p class="error">{error}</p>{/if}
    {#if recent.length}
      <h2>{t.recent}</h2>
      {#each recent as path (path)}
        <button class="recent" onclick={() => openRepo(path)}>{path}</button>
      {/each}
    {/if}
  </div>
{/if}

<style>
  :global(#app) {
    display: flex;
    flex-direction: column;
  }
  header {
    flex: none;
    display: flex;
    align-items: center;
    gap: 12px;
    height: 44px;
    padding: 0 14px;
    background: var(--panel);
    border-bottom: 1px solid var(--border);
  }
  strong {
    font-size: 14px;
  }
  .branch {
    padding: 1px 8px;
    border-radius: 4px;
    color: var(--accent);
    background: var(--accent-soft);
    font-size: 12px;
  }
  .spacer {
    flex: 1;
  }
  .muted {
    color: var(--muted);
    font-size: 12px;
  }
  button {
    padding: 5px 12px;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--raised);
    cursor: pointer;
  }
  button:hover {
    border-color: var(--muted);
  }
  main {
    flex: 1;
    min-height: 0;
    display: flex;
  }
  .center {
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
  .welcome {
    margin: auto;
    width: 440px;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
  h1 {
    margin: 0;
    font-size: 30px;
    font-weight: 600;
    letter-spacing: -0.5px;
  }
  h2 {
    margin: 24px 0 2px;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
  }
  .welcome .muted {
    margin: 0 0 12px;
    font-size: 13px;
  }
  .primary {
    padding: 8px 20px;
    border-color: var(--accent);
    background: var(--accent);
    color: #fff;
  }
  .primary:hover {
    border-color: var(--accent);
    filter: brightness(1.1);
  }
  .recent {
    width: 100%;
    padding: 6px 10px;
    border-color: transparent;
    background: none;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .recent:hover {
    border-color: transparent;
    background: var(--hover);
  }
</style>
