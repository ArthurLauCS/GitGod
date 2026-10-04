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
    gap: 16px;
    line-height: var(--lh-body);
  }
  p,
  ul {
    max-width: 36em;
    margin: 0;
  }
  ul {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding-left: 16px;
    color: var(--muted);
  }
  .undo {
    padding: 8px 12px;
    border-left: 2px solid var(--border-strong);
    color: var(--muted);
    font-size: var(--fs-sm);
  }
  strong {
    color: var(--text);
    font-weight: 600;
  }
</style>
