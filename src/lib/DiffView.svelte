<script lang="ts">
  import type { Diff } from './api'
  import { t } from './zh'

  // ponytail: 超过这么多行就截断显示；需要时换成虚拟滚动
  const MAX_LINES = 5000

  let {
    diff,
    mode = 'readonly',
    onapply,
  }: {
    diff: Diff | null
    /** unstaged / staged 时可以按区块、按行暂存或取消暂存 */
    mode?: 'readonly' | 'unstaged' | 'staged'
    onapply?: (hunk: number, header: string, lines: number[]) => void
  } = $props()

  /** 选中的行，键为「区块下标:行下标」 */
  let selected = $state(new Set<string>())
  $effect(() => {
    diff
    selected = new Set()
  })

  const hunks = $derived.by(() => {
    let budget = MAX_LINES
    return (diff?.hunks ?? []).map((h) => {
      const lines = h.lines.slice(0, Math.max(0, budget))
      budget -= h.lines.length
      return { ...h, lines }
    })
  })
  const truncated = $derived((diff?.hunks ?? []).reduce((n, h) => n + h.lines.length, 0) > MAX_LINES)

  const changed = (kind: string) => kind === '+' || kind === '-'
  const picked = (h: number) => [...selected].filter((k) => k.startsWith(`${h}:`)).map((k) => +k.split(':')[1])

  function toggle(h: number, i: number, kind: string) {
    if (mode === 'readonly' || !changed(kind)) return
    const next = new Set(selected)
    const key = `${h}:${i}`
    if (!next.delete(key)) next.add(key)
    selected = next
  }

  function apply(h: number) {
    const hunk = diff!.hunks[h]
    const lines = picked(h)
    // 没选行就是整个区块
    onapply?.(h, hunk.header, lines.length ? lines : hunk.lines.flatMap((l, i) => (changed(l.kind) ? [i] : [])))
  }

  function label(h: number) {
    const n = picked(h).length
    if (mode === 'unstaged') return n ? t.stageLines(n) : t.stageHunk
    return n ? t.unstageLines(n) : t.unstageHunk
  }
</script>

<div class="diff">
  {#if !diff}
    <p class="note">{t.pickFile}</p>
  {:else if diff.binary}
    <p class="note">{t.binary}</p>
  {:else if diff.too_large}
    <p class="note">{t.tooLarge}</p>
  {:else if !diff.hunks.length}
    <p class="note">{t.noDiff}</p>
  {:else}
    <div class="body">
      {#each hunks as hunk, h (h)}
        <div class="hunk">
          <span>{hunk.header}</span>
          {#if mode !== 'readonly'}<button onclick={() => apply(h)}>{label(h)}</button>{/if}
        </div>
        {#each hunk.lines as line, i (i)}
          <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
          <div
            class="line"
            class:add={line.kind === '+'}
            class:del={line.kind === '-'}
            class:meta={line.kind === '\\'}
            class:pick={mode !== 'readonly' && changed(line.kind)}
            class:selected={selected.has(`${h}:${i}`)}
            onclick={() => toggle(h, i, line.kind)}
          >
            <span class="no">{line.old_no ?? ''}</span><span class="no">{line.new_no ?? ''}</span><span class="sign">{line.kind}</span><span>{line.text}</span>
          </div>
        {/each}
      {/each}
      {#if truncated}<p class="note">{t.truncated(MAX_LINES)}</p>{/if}
    </div>
  {/if}
</div>

<style>
  .diff {
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow: auto;
    /* 区块标题用 100cqw 取本容器宽度，横向滚动时保持不动 */
    container-type: inline-size;
    background: var(--bg);
    font: 12px/20px var(--mono);
    user-select: text;
  }
  .body {
    width: max-content;
    min-width: 100%;
  }
  .note {
    margin: 0;
    padding: 24px;
    text-align: center;
    color: var(--muted);
    font-family: var(--font);
    font-size: 13px;
  }
  .hunk {
    position: sticky;
    left: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    width: 100cqw;
    height: 28px;
    padding: 0 12px;
    color: var(--muted);
    background: var(--raised);
    border-block: 1px solid var(--border);
  }
  .hunk span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .hunk button {
    flex: none;
    padding: 1px 10px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--panel);
    font: 12px var(--font);
    cursor: pointer;
  }
  .hunk button:hover {
    border-color: var(--accent);
    color: var(--accent);
  }
  .line {
    display: flex;
    white-space: pre;
    tab-size: 4;
  }
  .no {
    flex: none;
    width: 48px;
    padding-right: 8px;
    text-align: right;
    color: var(--muted);
    opacity: 0.6;
    user-select: none;
  }
  .sign {
    flex: none;
    width: 20px;
    text-align: center;
    color: var(--muted);
    user-select: none;
  }
  .add {
    background: color-mix(in srgb, var(--green) 14%, transparent);
  }
  .del {
    background: color-mix(in srgb, var(--red) 14%, transparent);
  }
  .meta {
    color: var(--muted);
  }
  .pick {
    cursor: pointer;
  }
  .pick:hover {
    filter: brightness(1.25);
  }
  .selected {
    box-shadow: inset 3px 0 var(--accent);
    background: var(--accent-soft);
  }
</style>
