<script lang="ts">
  import type { Ref, Row } from './api'
  import { fmtTime, t } from './zh'

  const ROW_H = 26
  const LANE_W = 14
  const PAD = 12
  const CHUNK = 128
  // ponytail: 超过 MAX_LANES 的泳道不画（内核图宽 572）；需要时给图列加横向滚动
  const MAX_LANES = 24
  // 浏览器单个元素的高度上限约 3300 万像素，超出后把滚动位置按比例映射到行号
  const MAX_H = 16_000_000
  const COLORS = ['#5c9dff', '#5fc27e', '#e0b252', '#c58af9', '#ef6b73', '#4fc4cf', '#f08d49', '#9aa5b8']

  let {
    fetchRows,
    count,
    version,
    badges,
    headId,
    selectedRow,
    onselect,
  }: {
    fetchRows: (start: number, count: number) => Promise<Row[]>
    count: number
    /** 提交图每次重新加载后加一，用来作废已缓存的行 */
    version: number
    badges: Map<string, Ref[]>
    headId: string | null
    selectedRow: number | null
    onselect: (row: number, id: string) => void
  } = $props()

  let viewport: HTMLDivElement
  let canvas = $state<HTMLCanvasElement>()
  let scrollTop = $state(0)
  let height = $state(0)
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
      ctx.fillStyle = row.id === headId ? getComputedStyle(canvas!).getPropertyValue('--panel') : color
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

<div class="head" style:padding-left="{graphW}px">
  <span>{t.colSubject}</span><span>{t.colAuthor}</span><span>{t.colDate}</span><span>{t.colCommit}</span>
</div>
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div class="viewport" bind:this={viewport} bind:clientHeight={height} onscroll={() => (scrollTop = viewport.scrollTop)} {onkeydown} tabindex="0" role="grid">
  <div style:height="{virt}px">
    <div class="window" style:transform="translateY({scrollTop - (top % ROW_H)}px)">
      <canvas bind:this={canvas} style:width="{graphW}px" style:height="{windowRows.length * ROW_H}px"></canvas>
      {#each windowRows as row, i (first + i)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          class="row"
          class:selected={first + i === selectedRow}
          style:padding-left="{graphW}px"
          role="row"
          tabindex="-1"
          onclick={() => row && onselect(first + i, row.id)}
        >
          {#if row}
            <span class="subject">
              {#each badges.get(row.id) ?? [] as ref (ref.name)}
                <span class="badge {kind(ref.name)}">{short(ref.name)}</span>
              {/each}
              {row.subject}
            </span>
            <span class="muted">{row.author}</span>
            <span class="muted">{fmtTime(row.time)}</span>
            <span class="muted mono">{row.id.slice(0, 8)}</span>
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
    grid-template-columns: minmax(0, 1fr) 150px 128px 76px;
    gap: 12px;
    align-items: center;
    padding-right: 12px;
  }
  .head {
    height: 28px;
    color: var(--muted);
    font-size: 12px;
    border-bottom: 1px solid var(--border);
    flex: none;
  }
  .viewport {
    flex: 1;
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
    height: 26px;
    cursor: default;
  }
  .row:hover {
    background: var(--hover);
  }
  .row.selected {
    background: var(--accent-soft);
  }
  .row span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .muted {
    color: var(--muted);
    font-size: 12px;
  }
  .mono {
    font-family: var(--mono);
  }
  .badge {
    margin-right: 6px;
    padding: 1px 6px;
    border-radius: 3px;
    font-size: 11px;
    border: 1px solid;
  }
  .badge.branch {
    color: var(--accent);
    background: var(--accent-soft);
    border-color: transparent;
  }
  .badge.remote {
    color: var(--muted);
    border-color: var(--border);
  }
  .badge.tag {
    color: var(--yellow);
    border-color: color-mix(in srgb, var(--yellow) 40%, transparent);
  }
</style>
