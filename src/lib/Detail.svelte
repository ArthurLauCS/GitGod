<script lang="ts">
  import type { Detail, Diff } from './api'
  import DiffView from './DiffView.svelte'
  import { fmtTime, t } from './zh'

  let {
    detail,
    onjump,
    fetchDiff,
  }: { detail: Detail | null; onjump: (id: string) => void; fetchDiff: (id: string, path: string) => Promise<Diff> } = $props()

  /** 正在看 diff 的文件；null 时右侧显示提交信息 */
  let file = $state<string | null>(null)
  let diff = $state.raw<Diff | null>(null)

  $effect(() => {
    detail
    file = null
  })
  $effect(() => {
    const f = file
    diff = null
    if (!f || !detail) return
    let stale = false
    fetchDiff(detail.id, f).then(
      (d) => stale || (diff = d),
      () => {},
    )
    return () => (stale = true)
  })
</script>

<section>
  {#if detail}
    <div class="files">
      <h3>{t.files(detail.files.length)}</h3>
      {#each detail.files as f (f.path)}
        <button class="file" class:selected={file === f.path} title={f.path} onclick={() => (file = file === f.path ? null : f.path)}>
          <span class="status s{f.status}" title={t.status[f.status] ?? f.status}>{f.status}</span>
          <span class="path">{#if f.old_path}<span class="muted">{f.old_path} → </span>{/if}{f.path}</span>
        </button>
      {/each}
    </div>
    {#if file}
      <DiffView {diff} />
    {:else}
      <div class="info">
        <pre class="message">{detail.message}</pre>
        <dl>
          <dt>{t.colCommit}</dt>
          <dd class="mono">{detail.id}</dd>
          <dt>{t.author}</dt>
          <dd>{detail.author} <span class="muted">&lt;{detail.author_email}&gt; · {fmtTime(detail.author_time)}</span></dd>
          {#if detail.committer !== detail.author || detail.committer_time !== detail.author_time}
            <dt>{t.committer}</dt>
            <dd>{detail.committer} <span class="muted">· {fmtTime(detail.committer_time)}</span></dd>
          {/if}
          {#if detail.parents.length}
            <dt>{t.parents}</dt>
            <dd>
              {#each detail.parents as p (p)}
                <button class="mono" onclick={() => onjump(p)}>{p.slice(0, 8)}</button>
              {/each}
            </dd>
          {/if}
        </dl>
      </div>
    {/if}
  {:else}
    <p class="empty">{t.noSelection}</p>
  {/if}
</section>

<style>
  section {
    height: 36%;
    flex: none;
    display: flex;
    border-top: 1px solid var(--border);
    background: var(--panel);
    user-select: text;
  }
  .info {
    flex: 1;
    min-width: 0;
    overflow: auto;
    padding: 12px 16px;
  }
  .files {
    width: 360px;
    flex: none;
    overflow: auto;
    padding: 10px 0;
    border-right: 1px solid var(--border);
    user-select: none;
  }
  .message {
    margin: 0 0 12px;
    font: inherit;
    font-size: 14px;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 4px 14px;
    margin: 0;
    font-size: 12px;
  }
  dt {
    color: var(--muted);
  }
  dd {
    margin: 0;
    overflow-wrap: anywhere;
  }
  dd button {
    margin-right: 8px;
    padding: 0;
    border: 0;
    background: none;
    color: var(--accent);
    cursor: pointer;
  }
  dd button:hover {
    text-decoration: underline;
  }
  h3 {
    margin: 0 12px 6px;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
  }
  .file {
    display: flex;
    gap: 8px;
    width: 100%;
    padding: 1px 12px;
    border: 0;
    background: none;
    text-align: left;
  }
  .file:hover {
    background: var(--hover);
  }
  .file.selected {
    background: var(--accent-soft);
  }
  .path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .status {
    flex: none;
    width: 16px;
    text-align: center;
    font: 600 12px/20px var(--mono);
    color: var(--yellow);
  }
  .sA {
    color: var(--green);
  }
  .sD {
    color: var(--red);
  }
  .sR,
  .sC {
    color: var(--accent);
  }
  .muted {
    color: var(--muted);
  }
  .mono {
    font-family: var(--mono);
  }
  .empty {
    margin: auto;
    color: var(--muted);
  }
</style>
