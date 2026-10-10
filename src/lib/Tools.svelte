<script lang="ts">
  import { untrack } from 'svelte'
  import * as api from './api'
  import Detail from './Detail.svelte'
  import RevisionPicker from './RevisionPicker.svelte'
  import type { Spec, Values } from './Dialog.svelte'
  import { t, errorText } from './i18n.svelte'

  let { tab, path, refs, busy, left: initialLeft = 'HEAD~1', right: initialRight = 'HEAD', exec, rebase, ask }: {
    tab: number; path: string; refs: api.Refs; busy: boolean; left?: string; right?: string
    exec: (op: api.Op) => Promise<boolean>; rebase: (plan: api.RebasePlan) => Promise<boolean>
    ask: (spec: Spec) => Promise<Values | null>
  } = $props()
  let section = $state<'compare' | 'remotes' | 'rebase' | 'reflog'>('compare')
  let left = $state(untrack(() => initialLeft))
  let right = $state(untrack(() => initialRight))
  let commonBase = $state(false)
  let comparison = $state<api.Comparison | null>(null)
  let error = $state('')
  let loading = $state(false)
  let remotes = $state<api.Remote[]>([])
  let logs = $state<api.Reflog[]>([])
  let moreLogs = $state(false)
  let base = $state('HEAD~1')
  let plan = $state<api.RebasePlan | null>(null)
  const disabled = $derived(busy || loading)
  const compareDetail = $derived<api.Detail | null>(comparison ? {
    id: comparison.right, parents: [comparison.left], author: '', author_email: '', author_time: 0,
    committer: '', committer_time: 0, message: `${comparison.left.slice(0, 12)} → ${comparison.right.slice(0, 12)}`, files: comparison.files,
  } : null)

  async function run(action: () => Promise<unknown>) {
    loading = true
    error = ''
    try { await action() } catch (e) { error = errorText(e) } finally { loading = false }
  }
  async function choose(value: typeof section) {
    section = value
    if (value === 'remotes') await run(async () => { remotes = await api.remoteDetails(tab) })
    if (value === 'reflog') await loadLogs(false)
  }
  async function loadLogs(append: boolean) {
    await run(async () => {
      const next = await api.reflog(tab, append ? logs.length : 0)
      moreLogs = next.length > 200
      logs = [...(append ? logs : []), ...next.slice(0, 200)]
    })
  }
  async function remote(action: 'add' | 'fetch' | 'push' | 'rename' | 'remove', item?: api.Remote) {
    const title = action === 'add' ? t.tools.addRemote : action === 'remove' ? t.tools.deleteRemote : action === 'rename' ? t.tools.renameRemote : action === 'push' ? t.tools.editPush : t.tools.editFetch
    const values = await ask({ title, message: item?.name, warning: action === 'remove' ? t.tools.removeRemote : undefined, danger: action === 'remove', fields:
      action === 'remove' ? [] : action === 'rename' ? [{ key: 'name', label: t.tools.remoteName, type: 'text', value: item!.name }]
      : [...(action === 'add' ? [{ key: 'name', label: t.tools.remoteName, type: 'text' as const, value: 'origin' }] : []), { key: 'url', label: t.tools.url, type: 'text', value: item?.[action === 'push' ? 'push' : 'fetch'] ?? '' }],
    })
    if (!values) return
    await run(async () => {
      const op: api.Op = action === 'add' ? { op: 'remote_add', name: String(values.name), url: String(values.url) }
        : action === 'remove' ? { op: 'remote_remove', name: item!.name }
        : action === 'rename' ? { op: 'remote_rename', old: item!.name, new: String(values.name) }
        : { op: 'remote_url', name: item!.name, url: String(values.url), push: action === 'push' }
      await exec(op)
      remotes = await api.remoteDetails(tab)
    })
  }
  async function recover(entry: api.Reflog) {
    const values = await ask({ title: t.tools.recover, message: `${entry.id}\n${entry.subject}`, fields: [{ key: 'name', label: t.newName, type: 'text', value: `recovered/${entry.id.slice(0, 8)}` }] })
    if (values) await run(() => exec({ op: 'create_branch', name: String(values.name), start: entry.id, checkout: false }))
  }
  function move(index: number, delta: number) {
    if (!plan) return
    const next = [...plan.steps]
    ;[next[index], next[index + delta]] = [next[index + delta], next[index]]
    plan.steps = next
  }
  async function apply() {
    if (!plan || !await ask({ title: t.tools.applyPlan, warning: t.tools.rebaseWarning, danger: true })) return
    await run(async () => { if (await rebase($state.snapshot(plan!))) plan = null })
  }
</script>

<div class="toolbox">
  <nav aria-label={t.tools.nav}>
    {#each ['compare', 'remotes', 'rebase', 'reflog'] as key}
      <button class="btn" class:on={section === key} disabled={disabled} onclick={() => choose(key as typeof section)}>{t.tools[key as typeof section]}</button>
    {/each}
  </nav>
  {#if error}<p class="error">{error}</p>{/if}
  {#if section === 'compare'}
    <form onsubmit={(e) => { e.preventDefault(); run(async () => { comparison = await api.compare(tab, left, right, commonBase) }) }}>
      <RevisionPicker {refs} label={t.tools.left} bind:value={left} />
      <RevisionPicker {refs} label={t.tools.right} bind:value={right} />
      <label class="check"><input type="checkbox" bind:checked={commonBase} />{t.tools.commonBase}</label>
      <button class="btn primary" disabled={disabled}>{t.tools.compare}</button>
    </form>
    {#if comparison}
      <div class="comparison">
        <Detail detail={compareDetail} comparison onjump={() => {}}
          reviewKey={`review:${JSON.stringify([path, comparison.left, comparison.right])}`}
          fetchDiff={(_, file) => api.diffBetween(tab, comparison!.left, comparison!.right, file, comparison!.files.find((f) => f.path === file)?.old_path ?? null)}
          fetchPage={(_, file, skip) => api.diffPage(tab, 'compare', comparison!.left, comparison!.right, file, skip, comparison!.files.find((f) => f.path === file)?.old_path ?? null)} />
      </div>
    {/if}
  {:else if section === 'remotes'}
    <button class="btn" disabled={disabled} onclick={() => remote('add')}>{t.tools.addRemote}</button>
    {#each remotes as item (item.name)}
      <article><h3>{item.name}</h3><p>Fetch: {item.fetch}</p><p>Push: {item.push}</p>
        <div class="actions">{#each ['fetch', 'push', 'rename', 'remove'] as action}
          <button class="btn small" disabled={disabled} onclick={() => remote(action as 'fetch' | 'push' | 'rename' | 'remove', item)}>{action === 'fetch' ? t.tools.editFetch : action === 'push' ? t.tools.editPush : action === 'rename' ? t.tools.renameRemote : t.tools.deleteRemote}</button>
        {/each}</div>
      </article>
    {/each}
  {:else if section === 'reflog'}
    <button class="btn" disabled={disabled} onclick={() => loadLogs(false)}>{t.tools.refresh}</button>
    {#each logs as entry, i (i)}
      <article class="reflog"><code title={entry.id}>{entry.id.slice(0, 12)}</code><span>{entry.selector} · {entry.subject}</span><button class="btn small" disabled={disabled} onclick={() => recover(entry)}>{t.tools.recover}</button></article>
    {:else}<p>{t.tools.noReflog}</p>{/each}
    {#if moreLogs}<button class="btn" disabled={disabled} onclick={() => loadLogs(true)}>{t.tools.loadMore}</button>{/if}
  {:else}
    <p>{t.tools.rebaseWarning}</p>
    <form onsubmit={(e) => { e.preventDefault(); run(async () => { plan = await api.rebasePlan(tab, base) }) }}>
      <RevisionPicker {refs} label={t.tools.base} bind:value={base} />
      <button class="btn" disabled={disabled || !!refs.in_progress}>{t.tools.loadPlan}</button>
    </form>
    {#if plan}
      <p><code>{plan.base.slice(0, 12)} → {plan.head.slice(0, 12)}</code></p>
      {#each plan.steps as step, index (step.id)}
        <article>
          <div class="actions">
            <button class="btn small" title={t.tools.up} aria-label={t.tools.up} disabled={disabled || index === 0} onclick={() => move(index, -1)}>↑</button>
            <button class="btn small" title={t.tools.down} aria-label={t.tools.down} disabled={disabled || index === plan!.steps.length - 1} onclick={() => move(index, 1)}>↓</button>
            <select class="field" aria-label={t.tools.action} bind:value={step.action} disabled={disabled}>{#each ['pick', 'reword', 'edit', 'squash', 'fixup', 'drop'] as action}<option value={action}>{t.tools.rebaseActions[action]}</option>{/each}</select>
            <code>{step.id.slice(0, 8)}</code><span>{step.subject}</span>
          </div>
          {#if step.action === 'reword' || step.action === 'squash'}<label>{t.tools.rebaseMessage}<textarea class="field" bind:value={step.message} disabled={disabled}></textarea></label>{/if}
        </article>
      {/each}
      <button class="btn primary" disabled={disabled || !plan.steps.length || !!refs.in_progress || plan.head !== refs.head_id} onclick={apply}>{t.tools.applyPlan}</button>
    {/if}
  {/if}
</div>

<style>
  .toolbox { flex: 1; min-height: 0; overflow: auto; padding: 16px; display: flex; flex-direction: column; gap: 12px; }
  nav, .actions, form { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; }
  form label { flex: 1; min-width: 200px; }
  label { display: flex; flex-direction: column; gap: 4px; color: var(--muted); }
  .check { flex-direction: row; align-items: center; }
  article { padding: 12px; background: var(--panel); border: 1px solid var(--border); border-radius: var(--r-md); }
  article p { overflow-wrap: anywhere; user-select: text; }
  .reflog { display: flex; gap: 12px; align-items: center; }
  .reflog span { flex: 1; overflow-wrap: anywhere; }
  h3, p { margin: 0 0 8px; }
  .error { color: var(--red); white-space: pre-wrap; }
  .comparison { flex: 1; min-height: 300px; display: flex; flex-direction: column; }
  .comparison :global(section) { height: 100% !important; flex: 1; min-height: 0; }
  .actions select { width: auto; }
  textarea { min-height: 64px; }
  code { user-select: text; }
</style>
