<script lang="ts">
  import { onMount } from 'svelte'
  import type { Detail, FileChange, Ref, Refs, Row } from '../../src/lib/api'
  import Graph from '../../src/lib/Graph.svelte'
  import { graphKey } from '../../src/lib/graph-key'
  import { errorText, t } from '../../src/lib/i18n.svelte'
  import { invoke } from './tauri'

  type Repository = { tab: number; root: string; name: string; refs: Refs; scope: string }
  let repos = $state.raw<Repository[]>([])
  let tab = $state<number>()
  let repo = $state.raw<Repository>()
  let count = $state(0)
  let version = $state(0)
  let selectedRow = $state<number | null>(null)
  let loading = $state(true)
  let error = $state('')
  let scope = $state('auto')
  let busy = $state(false)
  const head = $derived(repo?.refs.head?.replace(/^refs\/heads\//, '') ?? repo?.refs.head_id?.slice(0, 7) ?? '')
  const branches = $derived(repo?.refs.refs.filter((r) => /^refs\/(heads|remotes)\//.test(r.name) && !r.name.endsWith('/HEAD')) ?? [])
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
        if (!next || disposed) { repo = undefined; count = 0; break }
        tab = next.tab
        scope = next.scope
        // Auto 随 HEAD/上游切换；所有引用的提交集合没变，也可能需要换一份子图。
        const current = scope === 'auto' ? `${next.refs.head}:${next.refs.refs.find((r) => r.name === next.refs.head)?.upstream}` : ''
        const key = `${next.tab}:${scope}:${current}:${graphKey(next.refs)}`
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

  async function changeScope() {
    busy = true
    try { await invoke('graph_scope', { tab: repo!.tab, scope }); await refresh() }
    catch (e) { error = errorText(String(e)) }
    finally { busy = false }
  }
  async function sync(action: 'fetch' | 'pull' | 'push') {
    busy = true
    try { await invoke('graph_sync', { tab: repo!.tab, action }); await refresh() }
    catch (e) { error = errorText(String(e)) }
    finally { busy = false }
  }

  async function fetchFiles(id: string) {
    try { return (await invoke<Detail>('detail', { tab: repo!.tab, id })).files }
    catch (e) { throw errorText(String(e)) }
  }
  async function openFile(id: string, file: FileChange, preview: boolean) {
    try { await invoke('open_commit_file', { tab: repo!.tab, id, path: file.path, preview }) }
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
    <select aria-label={t.vscode.graphRepository} bind:value={tab} onchange={refresh} title={repo?.root} disabled={busy}>
      {#each repos as r (r.tab)}<option value={r.tab}>{r.name}</option>{/each}
    </select>
    <span title={t.vscode.graphCurrentBranch(head)}>{head}</span>
  </div>
  <div class="toolbar">
    <select aria-label={t.vscode.graphScope} bind:value={scope} onchange={changeScope} disabled={!repo || busy || loading} title={scope === 'auto' ? t.vscode.graphAutoHint : scope}>
      <option value="auto">{t.vscode.graphAuto}</option>
      <option value="all">{t.vscode.graphAll}</option>
      {#each branches as branch (branch.name)}
        <option value={branch.name}>{branch.name.replace(/^refs\/(heads|remotes)\//, '')}</option>
      {/each}
    </select>
    <button class="action" disabled={!repo || busy} title={`${t.fetch} · ${repo?.name ?? ''}`} aria-label={t.fetch} onclick={() => sync('fetch')}>⟳</button>
    <button class="action" disabled={!repo || busy || !repo.refs.head} title={`${t.pull} · ${head}`} aria-label={t.pull} onclick={() => sync('pull')}>↓<small>{repo?.refs.ahead_behind?.[1] || ''}</small></button>
    <button class="action" disabled={!repo || busy || !repo.refs.head} title={t.vscode.graphPushSame(head)} aria-label={t.vscode.graphPushSame(head)} onclick={() => sync('push')}>↑<small>{repo?.refs.ahead_behind?.[0] || ''}</small></button>
  </div>
  {#if error}<p role="alert">{error} <button onclick={refresh}>{t.vscode.graphRetry}</button></p>{/if}
  {#if loading}<p role="status">{t.loadingHistory}</p>{/if}
  {#if repo && count}
    {#key repo.tab}
      <Graph compact {fetchRows} {fetchFiles} onfile={openFile} {count} {version} {badges} headId={repo.refs.head_id} {selectedRow} onselect={(row) => selectedRow = row} onmenu={(e) => e.preventDefault()} />
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
  .action { height: 22px; min-width: 24px; padding: 0 4px; border: 0; border-radius: 3px; font-size: 16px; color: var(--text); background: transparent; cursor: pointer; }
  .action:hover { background: var(--hover); }
  .action:disabled { opacity: .45; cursor: default; }
  .action small { font-size: 10px; margin-left: 2px; }
  select:focus-visible, button:focus-visible { outline: 1px solid var(--accent); outline-offset: -1px; }
  p { margin: 8px; font-size: 12px; }
</style>
