<script lang="ts">
  import type { Graph } from './explain'
  import { t } from './zh'

  let { before, after }: { before: Graph; after: Graph } = $props()

  // ponytail: 示意图样式待重做（用户反馈不美观），留到统一打磨界面时处理
  const DX = 38
  const DY = 44
  const TOP = 58
  const R = 11
  const COLORS = ['var(--accent)', 'var(--green)', 'var(--yellow)']

  // 两张图用同一个尺寸，前后对比时节点位置不跳
  const all = $derived([...before.nodes, ...after.nodes])
  const width = $derived(Math.max(...all.map((n) => n.x)) * DX + 68)
  const rows = $derived(Math.max(...all.map((n) => n.y)) + 1)
  const height = $derived(TOP + (rows - 1) * DY + (rows > 1 ? 44 : 22))

  const x = (n: { x: number }) => 34 + n.x * DX
  const y = (n: { y: number }) => TOP + n.y * DY

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
        return { ...l, x: x(n), y: n.y === 0 ? y(n) - 22 - i * 18 : y(n) + 28 + i * 18 }
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
        <line
          x1={x(e.a)}
          y1={y(e.a)}
          x2={x(e.b)}
          y2={y(e.b)}
          stroke={COLORS[e.b.color]}
          class:ghost={e.a.kind === 'ghost' || e.b.kind === 'ghost'}
        />
      {/each}
      {#each g.nodes as n (n.id)}
        <circle cx={x(n)} cy={y(n)} r={R} fill={COLORS[n.color]} class:ghost={n.kind === 'ghost'} class:fresh={n.kind === 'new'} />
        <text x={x(n)} y={y(n) + 4} class="id" class:ghost={n.kind === 'ghost'}>{n.id}</text>
      {/each}
      {#each g.labels as l (l.at + l.text)}
        <text x={l.x} y={l.y} class="label" class:here={l.here} fill={COLORS[l.color]}>{l.here ? '▸ ' : ''}{l.text}</text>
      {/each}
    </svg>
  </figure>
{/snippet}

<div class="diagram">
  {@render panel(t.before, before)}
  <span class="arrow">→</span>
  {@render panel(t.after, after)}
</div>

<style>
  .diagram {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 8px;
    border: 1px solid var(--border);
    border-radius: 7px;
    background: var(--bg);
  }
  figure {
    margin: 0;
    text-align: center;
  }
  figcaption {
    color: var(--muted);
    font-size: 11px;
  }
  .arrow {
    color: var(--muted);
    font-size: 20px;
  }
  line {
    stroke-width: 2.5;
  }
  .ghost {
    opacity: 0.3;
    stroke-dasharray: 3 3;
  }
  circle.fresh {
    stroke: var(--text);
    stroke-width: 2.5;
  }
  .id {
    fill: #fff;
    font: 600 10px var(--mono);
    text-anchor: middle;
    stroke: none;
  }
  .label {
    font-size: 11px;
    text-anchor: middle;
  }
  .label.here {
    font-weight: 700;
  }
</style>
