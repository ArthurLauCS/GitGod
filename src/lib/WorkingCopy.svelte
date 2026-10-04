<script lang="ts">
  import * as api from './api'
  import DiffView from './DiffView.svelte'
  import { t } from './zh'

  let {
    tab,
    entries,
    reload,
    oncommitted,
  }: { tab: number; entries: api.Entry[]; reload: () => Promise<void>; oncommitted: () => void } = $props()

  let sel = $state<{ path: string; staged: boolean } | null>(null)
  let diff = $state.raw<api.Diff | null>(null)
  let message = $state('')
  let amend = $state(false)
  let error = $state('')
  let busy = $state(false)

  const unstaged = $derived(entries.filter((e) => e.unstaged))
  const staged = $derived(entries.filter((e) => e.staged))
  const canCommit = $derived(!busy && message.trim() !== '' && (staged.length > 0 || amend))

  $effect(() => {
    const s = sel
    if (!s) {
      diff = null
      return
    }
    const entry = entries.find((e) => e.path === s.path && (s.staged ? e.staged : e.unstaged))
    if (!entry) {
      sel = null
      return
    }
    let stale = false
    api.diffWorktree(tab, s.path, s.staged, !s.staged && entry.unstaged === '?').then(
      (d) => stale || (diff = d),
      (e) => stale || (error = String(e)),
    )
    return () => (stale = true)
  })

  async function run(op: Promise<unknown>) {
    busy = true
    error = ''
    try {
      await op
    } catch (e) {
      error = String(e)
    }
    await reload()
    busy = false
  }

  const stage = (list: api.Entry[]) => run(api.stage(tab, list.map((e) => e.path)))
  const unstage = (list: api.Entry[]) => run(api.unstage(tab, list.map((e) => e.path)))

  async function commit() {
    if (!canCommit) return
    await run(api.commit(tab, message, amend))
    if (error) return
    message = ''
    amend = false
    oncommitted()
  }

  async function toggleAmend() {
    if (amend && !message.trim()) message = await api.lastMessage(tab).catch(() => '')
  }

  const dir = (p: string) => p.slice(0, p.lastIndexOf('/') + 1)
  const base = (p: string) => p.slice(p.lastIndexOf('/') + 1)
</script>

{#snippet list(title: string, items: api.Entry[], isStaged: boolean)}
  <div class="group">
    <div class="title">
      <span>{title}<span class="count">{items.length}</span></span>
      <button disabled={busy || !items.length} onclick={() => (isStaged ? unstage(items) : stage(items))}>
        {isStaged ? t.unstageAll : t.stageAll}
      </button>
    </div>
    <div class="items">
      {#each items as e (e.path)}
        {@const s = (isStaged ? e.staged : e.unstaged) ?? ''}
        <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
        <div
          class="item"
          class:selected={sel?.path === e.path && sel.staged === isStaged}
          title={e.path}
          onclick={() => (sel = { path: e.path, staged: isStaged })}
          ondblclick={() => (isStaged ? unstage([e]) : stage([e]))}
        >
          <span class="status s{s === '?' ? 'A' : s}" title={t.status[s] ?? s}>{s}</span>
          <span class="path">{base(e.path) || e.path}<span class="dir">{dir(e.path.replace(/\/$/, ''))}</span></span>
          <button
            disabled={busy}
            title={isStaged ? t.unstageFile : t.stageFile}
            onclick={(ev) => (ev.stopPropagation(), isStaged ? unstage([e]) : stage([e]))}
          >
            {isStaged ? '−' : '+'}
          </button>
        </div>
      {/each}
    </div>
  </div>
{/snippet}

<div class="working">
  <div class="left">
    {#if !entries.length}<p class="clean">{t.clean}</p>{/if}
    {@render list(t.unstaged, unstaged, false)}
    {@render list(t.staged, staged, true)}
    <div class="commit">
      {#if error}<p class="error">{error}</p>{/if}
      <textarea
        placeholder={t.commitPlaceholder}
        bind:value={message}
        onkeydown={(e) => e.ctrlKey && e.key === 'Enter' && commit()}
      ></textarea>
      <div class="actions">
        <label><input type="checkbox" bind:checked={amend} onchange={toggleAmend} />{t.amend}</label>
        <button class="primary" disabled={!canCommit} onclick={commit}>{t.commit}</button>
      </div>
    </div>
  </div>
  <DiffView
    {diff}
    mode={sel?.staged ? 'staged' : 'unstaged'}
    onapply={(hunk, header, lines) => sel && run(api.applyLines(tab, sel.path, sel.staged, hunk, header, lines))}
  />
</div>

<style>
  .working {
    flex: 1;
    min-height: 0;
    display: flex;
  }
  .left {
    position: relative;
    width: 360px;
    flex: none;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border);
    background: var(--panel);
  }
  .clean {
    position: absolute;
    inset: 40% 0 auto;
    margin: 0;
    text-align: center;
    color: var(--muted);
    pointer-events: none;
  }
  .group {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    border-bottom: 1px solid var(--border);
  }
  .title {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 32px;
    padding: 0 8px 0 12px;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
  }
  .count {
    margin-left: 6px;
    font-weight: 400;
    opacity: 0.7;
  }
  .title button {
    padding: 1px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: none;
    font-size: 12px;
    cursor: pointer;
  }
  .title button:hover:enabled {
    border-color: var(--accent);
    color: var(--accent);
  }
  button:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .items {
    flex: 1;
    overflow-y: auto;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 24px;
    padding: 0 6px 0 12px;
    cursor: default;
  }
  .item:hover {
    background: var(--hover);
  }
  .item.selected {
    background: var(--accent-soft);
  }
  .path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dir {
    margin-left: 8px;
    color: var(--muted);
    font-size: 12px;
  }
  .item button {
    flex: none;
    visibility: hidden;
    width: 20px;
    height: 20px;
    padding: 0;
    border: 0;
    border-radius: 4px;
    background: var(--raised);
    font-size: 15px;
    line-height: 1;
    cursor: pointer;
  }
  .item:hover button {
    visibility: visible;
  }
  .item button:hover:enabled {
    background: var(--accent);
    color: #fff;
  }
  .status {
    flex: none;
    width: 14px;
    text-align: center;
    font: 600 12px var(--mono);
    color: var(--yellow);
  }
  .sA {
    color: var(--green);
  }
  .sD,
  .sU {
    color: var(--red);
  }
  .sR,
  .sC {
    color: var(--accent);
  }
  .commit {
    flex: none;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px;
  }
  textarea {
    height: 92px;
    padding: 7px 9px;
    resize: none;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 5px;
    color: inherit;
    font: inherit;
    outline: none;
    user-select: text;
  }
  textarea:focus {
    border-color: var(--accent);
  }
  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  label {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--muted);
    font-size: 12px;
  }
  .primary {
    padding: 5px 22px;
    border: 0;
    border-radius: 5px;
    background: var(--accent);
    color: #fff;
    cursor: pointer;
  }
  .primary:hover:enabled {
    filter: brightness(1.1);
  }
  .error {
    margin: 0;
    max-height: 96px;
    overflow: auto;
    color: var(--red);
    font-size: 12px;
    white-space: pre-wrap;
    user-select: text;
  }
</style>
