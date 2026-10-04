<script lang="ts">
  import type { Refs, Stash, Track, Worktree } from './api'
  import { t } from './zh'

  type Kind = 'branch' | 'remote' | 'tag' | 'stash'
  type View = 'history' | 'changes' | 'worktrees'

  let {
    refs,
    stashes,
    worktrees,
    tracks,
    view,
    changes,
    onview,
    onjump,
    onactivate,
    onmenu,
    onworktree,
  }: {
    refs: Refs
    stashes: Stash[]
    worktrees: Worktree[]
    /** 各本地分支与上游的同步状态，键是完整引用名 */
    tracks: Map<string, Track>
    view: View
    /** 有改动的文件数 */
    changes: number
    onview: (view: View) => void
    onjump: (id: string) => void
    /** 双击：name 是引用全名或贮藏名 */
    onactivate: (kind: Kind, name: string) => void
    onmenu: (e: MouseEvent, kind: Kind, name: string, id: string) => void
    /** 双击工作树：在页签中打开 */
    onworktree: (path: string) => void
  } = $props()

  /** 在别的工作树里打开着的分支 */
  const elsewhere = $derived(new Set(worktrees.filter((w) => !w.current && w.branch).map((w) => w.branch)))

  let filter = $state('')

  const group = (prefix: string) =>
    refs.refs
      .filter((r) => r.name.startsWith(prefix) && r.name.toLowerCase().includes(filter.toLowerCase()))
      .map((r) => ({ ...r, label: r.name.slice(prefix.length) }))
      // 当前分支排最前
      .sort((a, b) => +(b.name === refs.head) - +(a.name === refs.head))
  const branches = $derived(group('refs/heads/'))
  const remotes = $derived(group('refs/remotes/'))
  const tags = $derived(group('refs/tags/'))
  const basename = (p: string) => p.split(/[\\/]/).filter(Boolean).pop() ?? p
</script>

<aside>
  <nav>
    <button class:on={view === 'changes'} onclick={() => onview('changes')}>
      {t.changes}{#if changes}<span class="pill">{changes}</span>{/if}
    </button>
    <button class:on={view === 'history'} onclick={() => onview('history')}>{t.history}</button>
    <button class:on={view === 'worktrees'} onclick={() => onview('worktrees')}>
      {t.worktreesNav}{#if worktrees.length > 1}<span class="pill quiet">{worktrees.length}</span>{/if}
    </button>
  </nav>
  <input type="search" placeholder={t.filter} bind:value={filter} />
  <div class="scroll">
    {#snippet section(kind: Kind, title: string, items: { name: string; id: string; label: string }[], open: boolean)}
      <details {open}>
        <summary>{title}<span class="count">{items.length}</span></summary>
        {#each items as r (r.name)}
          <button
            class:head={r.name === refs.head}
            title={r.label}
            onclick={() => onjump(r.id)}
            ondblclick={() => onactivate(kind, r.name)}
            oncontextmenu={(e) => onmenu(e, kind, r.name, r.id)}
          >
            {r.label}
            {#if elsewhere.has(r.name)}<span class="sub" title={t.inOtherWorktreeMark}>⧉</span>{/if}
            {#if tracks.get(r.name)}
              {@const tr = tracks.get(r.name)!}
              <span class="sub" title={tr.gone ? t.gone : `${t.toPush(tr.ahead)} · ${t.toPull(tr.behind)}`}>
                {tr.gone ? t.gone : `${tr.ahead ? '↑' + tr.ahead : ''} ${tr.behind ? '↓' + tr.behind : ''}`}
              </span>
            {/if}
          </button>
        {/each}
      </details>
    {/snippet}
    {@render section('branch', t.branches, branches, true)}
    {@render section('remote', t.remotes, remotes, filter !== '')}
    {@render section('tag', t.tags, tags, filter !== '')}
    <details>
      <summary>{t.stashes}<span class="count">{stashes.length}</span></summary>
      {#each stashes as s (s.name)}
        <button title={s.subject} onclick={() => onjump(s.id)} oncontextmenu={(e) => onmenu(e, 'stash', s.name, s.id)}>
          {s.subject}
        </button>
      {/each}
    </details>
    <details open>
      <summary>{t.worktrees}<span class="count">{worktrees.length}</span></summary>
      {#each worktrees as w (w.path)}
        <button class:head={w.current} title={w.path} onclick={() => onjump(w.head)} ondblclick={() => w.current || onworktree(w.path)}>
          {basename(w.path)}
          <span class="sub">{w.branch ? w.branch.replace('refs/heads/', '') : t.detached}{w.changes ? ` · ${t.wtDirty(w.changes)}` : ''}</span>
        </button>
      {/each}
    </details>
  </div>
</aside>

<style>
  aside {
    width: 260px;
    flex: none;
    display: flex;
    flex-direction: column;
    background: var(--panel);
    border-right: 1px solid var(--border);
  }
  nav {
    padding: 8px 8px 0;
  }
  nav button {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 5px 10px;
    border-radius: 5px;
    font-weight: 600;
  }
  nav button.on {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .pill.quiet {
    background: var(--raised);
    color: var(--muted);
  }
  .pill {
    padding: 0 7px;
    border-radius: 9px;
    background: var(--accent);
    color: #fff;
    font-size: 11px;
    font-weight: 600;
  }
  input {
    margin: 10px;
    padding: 5px 9px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 5px;
    outline: none;
  }
  input:focus {
    border-color: var(--accent);
  }
  .scroll {
    flex: 1;
    overflow-y: auto;
    padding-bottom: 10px;
  }
  summary {
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
    cursor: default;
  }
  summary:hover {
    color: var(--text);
  }
  .count {
    margin-left: 6px;
    font-weight: 400;
    opacity: 0.7;
  }
  button {
    display: block;
    width: 100%;
    padding: 3px 12px 3px 26px;
    border: 0;
    background: none;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  button:hover {
    background: var(--hover);
  }
  button.head {
    color: var(--accent);
    font-weight: 600;
  }
  .sub {
    margin-left: 6px;
    color: var(--muted);
    font-size: 12px;
  }
</style>
