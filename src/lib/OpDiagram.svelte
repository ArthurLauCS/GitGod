<script lang="ts">
  import type { Graph } from './explain'
  import { t } from './zh'

  let { before, after }: { before: Graph; after: Graph } = $props()

  const DX = 36
  const DY = 46
  const PAD = 32
  const TOP = 60
  const R = 9
  const PILL_H = 18
  const COLORS = ['var(--lane-0)', 'var(--lane-1)', 'var(--lane-2)']

  // 两张图用同一个尺寸，前后对比时节点位置不跳
  const all = $derived([...before.nodes, ...after.nodes])
  const width = $derived(Math.max(...all.map((n) => n.x)) * DX + PAD * 2)
  const rows = $derived(Math.max(...all.map((n) => n.y)) + 1)
  const height = $derived(TOP + (rows - 1) * DY + (rows > 1 ? 52 : 26))

  const x = (n: { x: number }) => PAD + n.x * DX
  const y = (n: { y: number }) => TOP + n.y * DY
  /** 标签药丸的宽度：汉字按两个字符算 */
  const pillW = (text: string) => [...text].reduce((w, ch) => w + (ch.charCodeAt(0) > 255 ? 11 : 6.2), 0) + 14

  function layout(graph: Graph) {
    const at = new Map(graph.nodes.map((n) => [n.id, n]))
    const stacked = new Map<string, number>()
    return {
      nodes: graph.nodes,
      edges: graph.edges.map(([a, b]) => ({ a: at.get(a)!, b: at.get(b)! })),
      // 第一行的标签放在节点上方，其余行放下方；同一个节点上的多个标签向外堆叠
      labels: graph.labels.map((l) => {
        const n = at.get(l.at)!
        const i = stacked.get(l.at) ?? 0
        stacked.set(l.at, i + 1)
        const up = n.y === 0
        const cy = up ? y(n) - 26 - i * (PILL_H + 4) : y(n) + 26 + i * (PILL_H + 4)
        return { ...l, w: pillW(l.text), cx: x(n), cy, stem: i === 0 ? (up ? [cy + PILL_H / 2, y(n) - R - 2] : [y(n) + R + 2, cy - PILL_H / 2]) : null }
      }),
    }
  }
</script>

{#snippet panel(title: string, graph: Graph)}
  {@const g = layout(graph)}
  <figure>
    <figcaption>{title}</figcaption>
    <svg {width} {height}>
      {#each g.edges as e (e.a.id + e.b.id)}
        <!-- 分叉和汇合走曲线，不走斜线 -->
        <path
          d={e.a.y === e.b.y
            ? `M${x(e.a)} ${y(e.a)}H${x(e.b)}`
            : `M${x(e.a)} ${y(e.a)}C${x(e.a) + DX * 0.6} ${y(e.a)} ${x(e.b) - DX * 0.6} ${y(e.b)} ${x(e.b)} ${y(e.b)}`}
          stroke={COLORS[e.b.kind === 'new' && e.a.y !== e.b.y ? e.a.color : e.b.color]}
          class:ghost={e.a.kind === 'ghost' || e.b.kind === 'ghost'}
        />
      {/each}
      {#each g.nodes as n (n.id)}
        {#if n.kind === 'new'}<circle cx={x(n)} cy={y(n)} r={R + 5} class="halo" fill={COLORS[n.color]} />{/if}
        <circle cx={x(n)} cy={y(n)} r={R} fill={COLORS[n.color]} class="node" class:ghost={n.kind === 'ghost'} stroke={COLORS[n.color]} />
        <text x={x(n)} y={y(n) + 3.5} class="id" class:ghost={n.kind === 'ghost'}>{n.id}</text>
      {/each}
      {#each g.labels as l (l.at + l.text)}
        {#if l.stem}<line x1={l.cx} x2={l.cx} y1={l.stem[0]} y2={l.stem[1]} stroke={COLORS[l.color]} class="stem" />{/if}
        <rect
          x={l.cx - l.w / 2}
          y={l.cy - PILL_H / 2}
          width={l.w}
          height={PILL_H}
          rx="4"
          fill={COLORS[l.color]}
          stroke={COLORS[l.color]}
          class="pill"
          class:here={l.here}
        />
        <text x={l.cx} y={l.cy + 4} class="label" class:here={l.here} fill={COLORS[l.color]}>{l.text}</text>
      {/each}
    </svg>
  </figure>
{/snippet}

<div class="diagram">
  {@render panel(t.before, before)}
  <svg class="arrow" width="28" height="16" viewBox="0 0 28 16"><path d="M2 8h22M18 2l6 6-6 6" /></svg>
  {@render panel(t.after, after)}
</div>

<style>
  .diagram {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }
  figure {
    margin: 0;
    padding: 8px 4px 4px;
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    background: var(--bg);
  }
  figcaption {
    padding-left: 8px;
    color: var(--muted);
    font-size: var(--fs-sm);
    letter-spacing: 0.04em;
  }
  svg {
    display: block;
  }
  .arrow {
    flex: none;
  }
  .arrow path {
    fill: none;
    stroke: var(--muted);
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  path {
    fill: none;
    stroke-width: 2;
  }
  path.ghost {
    opacity: 0.4;
    stroke-dasharray: 3 4;
  }
  .node {
    stroke-width: 2;
  }
  /* 消失的提交：只剩虚线轮廓 */
  .node.ghost {
    fill: var(--bg);
    stroke-dasharray: 3 3;
    opacity: 0.6;
  }
  /* 新产生的提交：外面一圈光晕 */
  .halo {
    opacity: 0.25;
  }
  .id {
    fill: var(--bg);
    font: 600 var(--fs-xs) var(--mono);
    text-anchor: middle;
  }
  .id.ghost {
    fill: var(--muted);
  }
  .stem {
    stroke-width: 1;
    opacity: 0.6;
  }
  .pill {
    fill-opacity: 0.14;
    stroke-opacity: 0.5;
    stroke-width: 1;
  }
  /* 当前所在的分支：实心 */
  .pill.here {
    fill-opacity: 1;
    stroke-opacity: 1;
  }
  .label {
    font: 600 var(--fs-xs) var(--mono);
    text-anchor: middle;
  }
  .label.here {
    fill: var(--bg);
  }
</style>
