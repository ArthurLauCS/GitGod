<script lang="ts">
  import type { Worktree } from './api'
  import { explain } from './explain'
  import { fmtTime, t } from './zh'

  let {
    worktrees,
    onopen,
    onremove,
    onnew,
  }: { worktrees: Worktree[]; onopen: (path: string) => void; onremove: (w: Worktree) => void; onnew: () => void } = $props()

  const name = (p: string) => p.split(/[\\/]/).filter(Boolean).pop() ?? p
</script>

<div class="overview">
  <header>
    <div>
      <h2>{t.worktrees}</h2>
      <p>{explain.worktree.what}</p>
    </div>
    <button class="primary" onclick={onnew}>{t.newWorktree}</button>
  </header>
  <div class="table">
    <div class="row head">
      <span>{t.wtFolder}</span><span>{t.wtBranch}</span><span>{t.wtChanges}</span><span>{t.wtSync}</span><span>{t.wtLast}</span><span></span>
    </div>
    {#each worktrees as w, i (w.path)}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="row" class:current={w.current} ondblclick={() => w.current || onopen(w.path)}>
        <span class="folder" title={w.path}><strong>{name(w.path)}</strong><small>{w.path}</small></span>
        <span>
          <span class="branch">{w.branch ? w.branch.replace('refs/heads/', '') : t.detached}</span>
          {#if w.current}<small>{t.wtCurrent}</small>{/if}
        </span>
        <span class:dirty={w.changes > 0}>{w.changes ? t.wtDirty(w.changes) : t.wtClean}</span>
        <span>
          {#if !w.ahead_behind}<small>{t.wtNoUpstream}</small>
          {:else if w.ahead_behind[0] + w.ahead_behind[1] === 0}{t.wtSynced}
          {:else}{t.toPush(w.ahead_behind[0])} · {t.toPull(w.ahead_behind[1])}{/if}
        </span>
        <span class="last" title={w.subject}>{w.subject}<small>{w.time ? fmtTime(w.time) : ''}</small></span>
        <span class="actions">
          {#if !w.current}
            <button onclick={() => onopen(w.path)}>{t.openInTab}</button>
            <!-- 第一个是主工作树，git 不允许删除 -->
            {#if i > 0}<button class="danger" onclick={() => onremove(w)}>{t.wtRemove}</button>{/if}
          {/if}
        </span>
      </div>
    {/each}
  </div>
</div>

<style>
  .overview {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 20px 24px;
  }
  header {
    display: flex;
    align-items: flex-start;
    gap: 24px;
    margin-bottom: 16px;
  }
  header div {
    flex: 1;
  }
  h2 {
    margin: 0 0 6px;
    font-size: 16px;
    font-weight: 600;
  }
  p {
    margin: 0;
    max-width: 720px;
    color: var(--muted);
    line-height: 1.6;
  }
  .table {
    border: 1px solid var(--border);
    border-radius: 7px;
    background: var(--panel);
    overflow: hidden;
  }
  .row {
    display: grid;
    grid-template-columns: minmax(160px, 1.3fr) minmax(120px, 1fr) 110px 150px minmax(160px, 1.4fr) 150px;
    gap: 14px;
    align-items: center;
    min-height: 52px;
    padding: 8px 14px;
    border-top: 1px solid var(--border);
  }
  .row:first-child {
    border-top: 0;
  }
  .row.head {
    min-height: 32px;
    color: var(--muted);
    font-size: 12px;
  }
  .row:not(.head):hover {
    background: var(--hover);
  }
  .row.current {
    background: var(--accent-soft);
  }
  .row > span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  small {
    color: var(--muted);
    font-size: 12px;
  }
  .folder small,
  .last small {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .branch {
    padding: 1px 8px;
    border-radius: 4px;
    color: var(--accent);
    background: var(--accent-soft);
    font-size: 12px;
  }
  .dirty {
    color: var(--yellow);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
  }
  button {
    padding: 3px 10px;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--raised);
    cursor: pointer;
  }
  button:hover {
    border-color: var(--muted);
  }
  .danger:hover {
    border-color: var(--red);
    color: var(--red);
  }
  .primary {
    flex: none;
    padding: 6px 16px;
    border-color: var(--accent);
    background: var(--accent);
    color: #fff;
  }
</style>
