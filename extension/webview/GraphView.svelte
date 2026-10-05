<script lang="ts">
  import { onMount } from 'svelte'
  import type { Ref, Refs, Row } from '../../src/lib/api'
  import Graph from '../../src/lib/Graph.svelte'
  import { graphKey } from '../../src/lib/graph-key'
  import { errorText, t } from '../../src/lib/i18n.svelte'
  import { invoke } from './tauri'

  type Repository = { tab: number; root: string; name: string; refs: Refs }
  let repos = $state.raw<Repository[]>([])
  let tab = $state<number>()
  let repo = $state.raw<Repository>()
  let count = $state(0)
  let version = $state(0)
  let selectedRow = $state<number | null>(null)
  let loading = $state(true)
  let error = $state('')
  let loadedKey = ''
  let running = false
  let again = false
  let disposed = false
  const badges = $derived.by(() => {
    const result = new Map<string, Ref[]>()
    for (const ref of repo?.refs.refs ?? []) result.set(ref.id, [...(result.get(ref.id) ?? []), ref])
    return result
  })

  // 一次只加载一份图；切仓库或刷新时合并请求，避免旧加载覆盖引擎里的新图。
  async function refresh() {
    again = true
    if (running) return
    running = true
    do {
      again = false
      try {
        repos = await invoke<Repository[]>('graph_repos')
        const next = repos.find((r) => r.tab === tab) ?? repos[0]
        if (!next || disposed) break
        tab ??= next.tab
        const key = `${next.tab}:${graphKey(next.refs)}`
        repo = next
        if (key === loadedKey) continue
        loading = true
        error = ''
        count = 0
        selectedRow = null
        count = await invoke<number>('load_graph', { tab: next.tab, full: false })
        version++
        if (count >= 2000) {
          count = await invoke<number>('load_graph', { tab: next.tab, full: true })
          version++
        }
        loadedKey = key
      } catch (e) {
        error = errorText(String(e))
      } finally {
        loading = false
      }
    } while (again && !disposed)
    running = false
  }

  async function select(row: number, id: string) {
    selectedRow = row
    try { await invoke('reveal_commit', { tab: repo!.tab, id }) }
    catch (e) { error = errorText(String(e)) }
  }
  async function fetchRows(start: number, count: number) {
    try { return await invoke<Row[]>('rows', { tab: repo!.tab, start, count }) }
    catch (e) { loadedKey = ''; error = errorText(String(e)); return [] }
  }
  onMount(() => {
    refresh()
    addEventListener('focus', refresh)
    return () => { disposed = true; removeEventListener('focus', refresh) }
  })
</script>

<main>
  <div class="toolbar">
    <select aria-label={t.vscode.graphRepository} bind:value={tab} onchange={refresh} title={repo?.root}>
      {#each repos as r (r.tab)}<option value={r.tab}>{r.name}</option>{/each}
    </select>
    <span title={repo?.refs.head ?? ''}>{repo?.refs.head?.replace(/^refs\/heads\//, '') ?? repo?.refs.head_id?.slice(0, 7) ?? ''}</span>
  </div>
  {#if error}<p role="alert">{error} <button onclick={refresh}>{t.vscode.graphRetry}</button></p>{/if}
  {#if loading}<p role="status">{t.loadingHistory}</p>{/if}
  {#if repo && count}
    {#key repo.tab}
      <Graph compact {fetchRows} {count} {version} {badges} headId={repo.refs.head_id} {selectedRow} onselect={select} onmenu={(e) => e.preventDefault()} />
    {/key}
  {:else if !loading && !error}
    <p>{repo ? t.noCommits : t.vscode.noRepo}</p>
  {/if}
</main>

<style>
  main {
    height: 100vh;
    display: flex;
    flex-direction: column;
    --bg: var(--vscode-sideBar-background);
    --text: var(--vscode-foreground);
    --muted: var(--vscode-descriptionForeground);
    --hover: var(--vscode-list-hoverBackground);
    --accent: var(--vscode-focusBorder);
    --accent-soft: var(--vscode-list-inactiveSelectionBackground);
    --ref-branch: var(--vscode-textLink-foreground);
    --border-strong: var(--vscode-contrastBorder, var(--vscode-widget-border, var(--vscode-descriptionForeground)));
    --fs-md: 12px;
    --fs-sm: 11px;
    --font: var(--vscode-font-family);
    background: var(--bg);
    color: var(--text);
    font-family: var(--font);
    font-size: var(--fs-md);
  }
  .toolbar { display: flex; gap: 4px; align-items: center; padding: 2px 4px; }
  select { min-width: 0; flex: 1; height: 22px; padding: 0 2px; font: inherit; background: var(--vscode-dropdown-background); color: var(--vscode-dropdown-foreground); border: 1px solid var(--vscode-dropdown-border); }
  .toolbar span { max-width: 40%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: var(--fs-sm); color: var(--muted); }
  p { margin: 8px; font-size: 12px; }
</style>
