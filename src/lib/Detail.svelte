<script lang="ts">
  import type { Detail } from './api'
  import { fmtTime, t } from './zh'

  let { detail, onjump }: { detail: Detail | null; onjump: (id: string) => void } = $props()
</script>

<section>
  {#if detail}
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
    <div class="files">
      <h3>{t.files(detail.files.length)}</h3>
      {#each detail.files as f (f.path)}
        <div class="file" title={t.status[f.status] ?? f.status}>
          <span class="status s{f.status}">{f.status}</span>
          <span class="path">{#if f.old_path}<span class="muted">{f.old_path} → </span>{/if}{f.path}</span>
        </div>
      {/each}
    </div>
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
  .info,
  .files {
    flex: 1;
    min-width: 0;
    overflow: auto;
    padding: 12px 16px;
  }
  .files {
    border-left: 1px solid var(--border);
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
    margin: 0 0 8px;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
  }
  .file {
    display: flex;
    gap: 8px;
    padding: 1px 0;
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
