<script lang="ts">
  import type { Ref, Row } from './api'
  import { authorColor, authorKey } from './author'
  import Dialog from './Dialog.svelte'
  import { layout } from './layout.svelte'
  import { prefs, setAuthorStyle } from './prefs.svelte'
  import Splitter from './Splitter.svelte'
  import { theme } from './theme.svelte'
  import { fmtTime, t } from './i18n.svelte'

  const ROW_H = 28
  const LANE_W = 14
  const PAD = 12
  const CHUNK = 128
  // ponytail: 超过 MAX_LANES 的泳道不画（内核图宽 572）；需要时给图列加横向滚动
  const LANE_CAP = 24
  // 浏览器单个元素的高度上限约 3300 万像素，超出后把滚动位置按比例映射到行号
  const MAX_H = 16_000_000
  const LANES = 8

  let {
    fetchRows,
    count,
    version,
    badges,
    headId,
    selectedRow,
    onselect,
    onmenu,
  }: {
    fetchRows: (start: number, count: number) => Promise<Row[]>
    count: number
    /** 提交图每次重新加载后加一，用来作废已缓存的行 */
    version: number
    badges: Map<string, Ref[]>
    headId: string | null
    selectedRow: number | null
    onselect: (row: number, id: string) => void
    onmenu: (e: MouseEvent, id: string) => void
  } = $props()

  let viewport: HTMLDivElement
  let canvas = $state<HTMLCanvasElement>()
  /** 只高亮这个作者的提交，其余变淡 */
  let focusAuthor = $state<{ key: string; name: string } | null>(null)
  let authorDialog = $state<Dialog>()

  async function styleAuthor(row: Row) {
    const key = authorKey(row)
    const style = prefs.authorStyles[key]
    const values = await authorDialog?.ask({
      title: t.authorStyle(row.author),
      message: row.author_email,
      fields: [
        { key: 'border', label: t.authorBorder, type: 'checkbox', value: style?.border ?? false },
        { key: 'fill', label: t.authorFill, type: 'checkbox', value: style?.fill ?? false },
      ],
    })
    if (values) setAuthorStyle(key, { border: values.border as boolean, fill: values.fill as boolean })
  }
  const cols = $derived(`minmax(0, 1fr) min(${layout.author}px, 18vw) min(${layout.date}px, 13vw) 72px`)
  let scrollTop = $state(0)
  let height = $state(0)
  let width = $state(0)
  /** 泳道列最多占提交图宽度的 28%，窄窗口下给说明列留地方 */
  const MAX_LANES = $derived(Math.max(4, Math.min(LANE_CAP, Math.floor((width * 0.28 - PAD * 2) / LANE_W))))
  let loaded = $state(0)

  const chunks = new Map<number, Row[]>()
  const pending = new Set<number>()
  let gen = 0
  let seenVersion = -1

  const total = $derived(count * ROW_H)
  const virt = $derived(Math.min(total, MAX_H))
  const scale = $derived(virt > height ? (total - height) / (virt - height) : 1)
  const top = $derived(scrollTop * scale)
  const first = $derived(Math.floor(top / ROW_H))
  const visible = $derived(Math.ceil(height / ROW_H) + 1)

  const rowAt = (r: number) => chunks.get(Math.floor(r / CHUNK))?.[r % CHUNK]

  const windowRows = $derived.by(() => {
    loaded
    const out: (Row | undefined)[] = []
    for (let r = first; r < Math.min(first + visible, count); r++) out.push(rowAt(r))
    return out
  })

  const lanes = $derived(
    Math.min(MAX_LANES, 1 + Math.max(0, ...windowRows.map((r) => (r ? Math.max(r.lane, ...r.through, ...r.out, ...r.incoming) : 0)))),
  )
  const graphW = $derived(PAD * 2 + lanes * LANE_W)

  $effect(() => {
    if (version !== seenVersion) {
      seenVersion = version
      gen++
      chunks.clear()
      pending.clear()
    }
    const g = gen
    for (let c = Math.floor(first / CHUNK); c * CHUNK < Math.min(first + visible, count); c++) {
      if (chunks.has(c) || pending.has(c)) continue
      pending.add(c)
      fetchRows(c * CHUNK, CHUNK).then((rows) => {
        if (g !== gen) return
        pending.delete(c)
        if (chunks.size > 256) chunks.clear()
        chunks.set(c, rows)
        loaded++
      })
    }
  })

  $effect(() => {
    if (!canvas) return
    // 泳道色跟主题走，切主题时重画
    theme.current
    const css = getComputedStyle(canvas)
    const COLORS = Array.from({ length: LANES }, (_, i) => css.getPropertyValue(`--lane-${i}`).trim())
    const dpr = window.devicePixelRatio || 1
    const h = windowRows.length * ROW_H
    canvas.width = graphW * dpr
    canvas.height = h * dpr
    const ctx = canvas.getContext('2d')!
    ctx.scale(dpr, dpr)
    ctx.lineWidth = 1.6
    const x = (lane: number) => PAD + lane * LANE_W + LANE_W / 2
    const link = (lane: number, x0: number, y0: number, x1: number, y1: number) => {
      ctx.strokeStyle = COLORS[lane % COLORS.length]
      ctx.beginPath()
      ctx.moveTo(x0, y0)
      // 竖直方向出入、中间平滑过渡
      ctx.bezierCurveTo(x0, (y0 + y1) / 2, x1, (y0 + y1) / 2, x1, y1)
      ctx.stroke()
    }
    windowRows.forEach((row, i) => {
      if (!row) return
      const y = i * ROW_H
      const cy = y + ROW_H / 2
      const cx = x(row.lane)
      for (const l of row.through) if (l < MAX_LANES) link(l, x(l), y, x(l), y + ROW_H)
      for (const l of row.incoming) if (l < MAX_LANES) link(l, x(l), y, cx, cy)
      for (const l of row.out) if (l < MAX_LANES) link(l, cx, cy, x(l), y + ROW_H)
      if (row.lane >= MAX_LANES) return
      const color = COLORS[row.lane % COLORS.length]
      ctx.beginPath()
      ctx.arc(cx, cy, 4, 0, Math.PI * 2)
      ctx.fillStyle = row.id === headId ? css.getPropertyValue('--bg') : color
      ctx.fill()
      ctx.strokeStyle = color
      ctx.lineWidth = 2
      ctx.stroke()
      ctx.lineWidth = 1.6
    })
  })

  export function scrollToRow(row: number, center = false) {
    const y = row * ROW_H
    if (center) viewport.scrollTop = (y - height / 2 + ROW_H / 2) / scale
    else if (y < top) viewport.scrollTop = y / scale
    else if (y + ROW_H > top + height) viewport.scrollTop = (y + ROW_H - height) / scale
  }

  function onkeydown(e: KeyboardEvent) {
    const step = { ArrowDown: 1, ArrowUp: -1, PageDown: visible - 2, PageUp: 2 - visible }[e.key]
    if (!step || selectedRow === null) return
    e.preventDefault()
    const next = Math.max(0, Math.min(count - 1, selectedRow + step))
    const row = rowAt(next)
    if (!row) return
    onselect(next, row.id)
    scrollToRow(next)
  }

  const kind = (name: string) => (name.startsWith('refs/heads/') ? 'branch' : name.startsWith('refs/tags/') ? 'tag' : 'remote')
  const short = (name: string) => name.replace(/^refs\/(heads|tags|remotes)\//, '')
</script>

<Dialog bind:this={authorDialog} />

<div class="head" style:padding-left="{graphW}px" style:grid-template-columns={cols}>
  <span>
    {t.colSubject}
    {#if focusAuthor}
      <button class="chip" style:color={authorColor(focusAuthor.name)} onclick={() => (focusAuthor = null)}>{t.onlyAuthor(focusAuthor.name)} ×</button>
    {/if}
  </span>
  <span class="cell">
    <span class="grip" title={t.dragHint}><Splitter key="author" min={80} max={360} invert /></span>
    <span title={t.authorHint}>{t.colAuthor}</span>
  </span>
  <span class="cell"><span class="grip" title={t.dragHint}><Splitter key="date" min={96} max={220} invert /></span>{t.colDate}</span>
  <span>{t.colCommit}</span>
</div>
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div class="viewport" bind:this={viewport} bind:clientHeight={height} bind:clientWidth={width} onscroll={() => (scrollTop = viewport.scrollTop)} {onkeydown} tabindex="0" role="grid">
  <div style:height="{virt}px">
    <div class="window" style:transform="translateY({scrollTop - (top % ROW_H)}px)">
      <canvas bind:this={canvas} style:width="{graphW}px" style:height="{windowRows.length * ROW_H}px"></canvas>
      {#each windowRows as row, i (first + i)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          class="row"
          class:selected={first + i === selectedRow}
          class:dim={focusAuthor !== null && (!row || authorKey(row) !== focusAuthor.key)}
          style:padding-left="{graphW}px"
          style:grid-template-columns={cols}
          role="row"
          tabindex="-1"
          onclick={() => row && onselect(first + i, row.id)}
          oncontextmenu={(e) => row && (onselect(first + i, row.id), onmenu(e, row.id))}
        >
          {#if row}
            <span class="subject">
              {#if badges.has(row.id)}
                <span class="badges">
                  {#each badges.get(row.id) ?? [] as ref (ref.name)}
                    <span class="badge {kind(ref.name)}">{short(ref.name)}</span>
                  {/each}
                </span>
              {/if}
              <span>{row.subject}</span>
            </span>
            <button
              class="author"
              class:boxed={prefs.authorStyles[authorKey(row)]?.border}
              class:filled={prefs.authorStyles[authorKey(row)]?.fill}
              style:color={authorColor(row.author)}
              title={t.authorHint}
              onclick={(e) => (e.stopPropagation(), (focusAuthor = focusAuthor?.key === authorKey(row) ? null : { key: authorKey(row), name: row.author }))}
              oncontextmenu={(e) => { e.preventDefault(); e.stopPropagation(); styleAuthor(row) }}
              onkeydown={(e) => {
                if (e.key === 'ContextMenu' || (e.shiftKey && e.key === 'F10')) {
                  e.preventDefault()
                  e.stopPropagation()
                  styleAuthor(row)
                }
              }}
            >
              {row.author}
            </button>
            <span class="muted">{fmtTime(row.time)}</span>
            <span class="muted mono">{row.id.slice(0, 7)}</span>
          {/if}
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .head,
  .row {
    display: grid;
    gap: 12px;
    align-items: center;
    padding-right: 12px;
  }
  .head {
    position: relative;
    z-index: 3;
    flex: none;
    height: var(--row);
    color: var(--muted);
    font-size: var(--fs-sm);
    border-bottom: 1px solid var(--border);
  }
  .cell {
    position: relative;
  }
  /* 列分隔条贴在列的左边缘 */
  .grip {
    position: absolute;
    left: -6px;
    top: 0;
    bottom: 0;
    display: flex;
  }
  .chip {
    margin-left: 8px;
    padding: 0 8px;
    border: 1px solid currentColor;
    border-radius: var(--r-lg);
    background: none;
    font-size: var(--fs-sm);
    cursor: pointer;
  }
  .viewport {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    outline: none;
  }
  .window {
    position: relative;
    will-change: transform;
  }
  canvas {
    position: absolute;
    left: 0;
    top: 0;
    pointer-events: none;
  }
  .row {
    height: var(--row);
    cursor: default;
  }
  .row:hover {
    background: var(--hover);
  }
  .row.selected {
    background: var(--accent-soft);
    box-shadow: inset 2px 0 var(--accent);
  }
  /* 高亮某个作者时，其他人的提交退后 */
  .row.dim > :global(*) {
    opacity: 0.3;
  }
  .row span,
  .author {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .author {
    padding: 0;
    border: 0;
    background: none;
    font-size: var(--fs-sm);
    text-align: left;
    cursor: pointer;
  }
  .author:hover {
    text-decoration: underline;
  }
  /* 可选：给作者名加边框和（或）底色，颜色都跟着作者色走 */
  .author.boxed,
  .author.filled {
    justify-self: start;
    max-width: 100%;
    padding: 0 8px;
    border: 1px solid transparent;
    border-radius: var(--r-lg);
    line-height: 20px;
  }
  .author.boxed {
    border-color: currentColor;
  }
  .author.filled {
    background: color-mix(in srgb, currentColor 18%, transparent);
  }
  .muted {
    color: var(--muted);
    font-size: var(--fs-sm);
  }
  .mono {
    font-family: var(--mono);
  }
  /* 引用标签最多占说明列的一半，剩下的留给提交标题 */
  .subject {
    display: flex;
    align-items: center;
  }
  .badges {
    flex: 0 1 auto;
    max-width: 50%;
  }
  .subject > span:last-child {
    flex: 1;
    min-width: 0;
  }
  .badge {
    margin-right: 8px;
    padding: 2px 8px;
    border-radius: var(--r-sm);
    font-size: var(--fs-sm);
    font-weight: 600;
    border: 1px solid;
  }
  .badge.branch {
    color: var(--accent);
    background: var(--accent-soft);
    border-color: transparent;
  }
  .badge.remote {
    color: var(--muted);
    border-color: var(--border-strong);
    font-weight: 400;
  }
  .badge.tag {
    color: var(--yellow);
    border-color: color-mix(in srgb, var(--yellow) 45%, transparent);
  }
</style>
