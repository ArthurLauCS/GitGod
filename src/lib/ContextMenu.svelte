<script lang="ts" module>
  /** null 表示分隔线 */
  export type Item = { label: string; action: () => void; danger?: boolean } | null
</script>

<script lang="ts">
  let items = $state<Item[]>([])
  let x = $state(0)
  let y = $state(0)
  let menu = $state<HTMLDivElement>()

  export function show(e: MouseEvent, list: Item[]) {
    e.preventDefault()
    items = list
    x = e.clientX
    y = e.clientY
  }

  // 贴近窗口边缘时往回收，保证整个菜单可见
  $effect(() => {
    if (!menu) return
    const r = menu.getBoundingClientRect()
    if (r.right > innerWidth) x = Math.max(0, innerWidth - r.width - 4)
    if (r.bottom > innerHeight) y = Math.max(0, innerHeight - r.height - 4)
  })

  const close = () => (items = [])
</script>

<svelte:window onmousedown={(e) => menu?.contains(e.target as Node) || close()} onkeydown={(e) => e.key === 'Escape' && close()} onblur={close} />

{#if items.length}
  <div class="menu" bind:this={menu} style:left="{x}px" style:top="{y}px" role="menu">
    {#each items as item, i (i)}
      {#if item}
        <button role="menuitem" class:danger={item.danger} onclick={() => (close(), item.action())}>{item.label}</button>
      {:else}
        <hr />
      {/if}
    {/each}
  </div>
{/if}

<style>
  .menu {
    position: fixed;
    z-index: 10;
    min-width: 200px;
    padding: 5px;
    border: 1px solid var(--border);
    border-radius: 7px;
    background: var(--raised);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
  }
  button {
    display: block;
    width: 100%;
    padding: 5px 12px;
    border: 0;
    border-radius: 4px;
    background: none;
    text-align: left;
    white-space: nowrap;
  }
  button:hover {
    background: var(--accent);
    color: #fff;
  }
  .danger {
    color: var(--red);
  }
  hr {
    margin: 5px 0;
    border: 0;
    border-top: 1px solid var(--border);
  }
</style>
