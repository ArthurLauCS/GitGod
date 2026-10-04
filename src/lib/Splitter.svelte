<script lang="ts">
  import { DEFAULTS, layout, saveLayout, type LayoutKey } from './layout.svelte'

  let {
    key,
    dir = 'x',
    min,
    max,
    invert = false,
  }: {
    /** 拖动时改 layout 里的哪一项 */
    key: LayoutKey
    /** x：左右拖改宽度；y：上下拖改高度 */
    dir?: 'x' | 'y'
    min: number
    max: number
    /** 被调整的栏位在分隔条的右侧或下方时，拖动方向与尺寸变化相反 */
    invert?: boolean
  } = $props()

  let dragging = $state(false)

  function down(e: PointerEvent) {
    const el = e.currentTarget as HTMLElement
    const pos = (ev: PointerEvent) => (dir === 'x' ? ev.clientX : ev.clientY)
    const start = pos(e)
    const from = layout[key]
    el.setPointerCapture(e.pointerId)
    dragging = true
    const move = (ev: PointerEvent) => {
      const delta = (pos(ev) - start) * (invert ? -1 : 1)
      layout[key] = Math.round(Math.min(max, Math.max(min, from + delta)))
    }
    el.addEventListener('pointermove', move)
    el.addEventListener(
      'pointerup',
      () => {
        el.removeEventListener('pointermove', move)
        dragging = false
        saveLayout()
      },
      { once: true },
    )
  }

  /** 双击恢复默认尺寸 */
  function reset() {
    layout[key] = DEFAULTS[key]
    saveLayout()
  }
</script>

<div class="splitter {dir}" class:dragging onpointerdown={down} ondblclick={reset} role="separator" aria-orientation={dir === 'x' ? 'vertical' : 'horizontal'}></div>

<style>
  /* 看得见的是 1px 的线，能抓住的是它两侧各 4px */
  .splitter {
    position: relative;
    flex: none;
    background: var(--border);
    z-index: 2;
  }
  .splitter::after {
    content: '';
    position: absolute;
  }
  .x {
    width: 1px;
    cursor: col-resize;
  }
  .x::after {
    inset: 0 -4px;
  }
  .y {
    height: 1px;
    cursor: row-resize;
  }
  .y::after {
    inset: -4px 0;
  }
  .splitter:hover,
  .dragging {
    background: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
  }
</style>
