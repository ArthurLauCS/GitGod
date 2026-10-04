<script lang="ts">
  import type { Explain } from './explain'
  import OpDiagram from './OpDiagram.svelte'
  import { t } from './zh'

  let { explain }: { explain: Explain } = $props()
</script>

<div class="explain">
  <p>{explain.what}</p>
  {#if explain.before && explain.after}
    <OpDiagram before={explain.before} after={explain.after} />
  {/if}
  <ul>
    {#each explain.result as r (r)}<li>{r}</li>{/each}
  </ul>
  {#if explain.undo}<p class="undo"><strong>{t.ifWrong}</strong>{explain.undo}</p>{/if}
</div>

<style>
  .explain {
    display: flex;
    flex-direction: column;
    gap: 10px;
    line-height: 1.6;
  }
  p,
  ul {
    margin: 0;
  }
  ul {
    padding-left: 18px;
    color: var(--muted);
  }
  .undo {
    color: var(--muted);
    font-size: 12px;
  }
  strong {
    color: var(--text);
    font-weight: 600;
  }
</style>
