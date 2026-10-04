<script lang="ts">
  import type { Conflict, Side } from './api'
  import { t } from './zh'

  let {
    conflict,
    onresolve,
    ontake,
  }: {
    conflict: Conflict | null
    /** 每个冲突块选了哪一边，按出现顺序 */
    onresolve: (choices: Side[]) => void
    /** 整个文件取一边 */
    ontake: (theirs: boolean) => void
  } = $props()

  let choices = $state<(Side | null)[]>([])
  $effect(() => {
    choices = (conflict?.blocks ?? []).filter((b) => b.kind === 'conflict').map(() => null)
  })

  // 冲突块在 choices 里的下标
  const numbered = $derived.by(() => {
    let n = 0
    return (conflict?.blocks ?? []).map((b) => ({ block: b, index: b.kind === 'conflict' ? n++ : -1 }))
  })
  const done = $derived(choices.length > 0 && choices.every((c) => c !== null))
</script>

<div class="conflict">
  {#if !conflict}
    <p class="note">{t.pickFile}</p>
  {:else}
    <header>
      <span>{conflict.binary ? t.conflictBinary : t.conflictHint(choices.length)}</span>
      <button onclick={() => ontake(false)}>{t.takeOurs}</button>
      <button onclick={() => ontake(true)}>{t.takeTheirs}</button>
      {#if !conflict.binary}
        <button class="primary" disabled={!done} onclick={() => onresolve(choices as Side[])}>{t.resolve}</button>
      {/if}
    </header>
    <div class="body">
      {#each numbered as { block, index }, i (i)}
        {#if block.kind === 'text'}
          <pre class="same">{block.text}</pre>
        {:else}
          {@const choice = choices[index]}
          <section>
            <div class="side" class:picked={choice === 'ours' || choice === 'both'} class:dropped={choice === 'theirs'}>
              <div class="bar">
                <span>{t.ours}<small>{block.ours_label}</small></span>
                <button onclick={() => (choices[index] = 'ours')}>{t.useThis}</button>
              </div>
              <pre>{block.ours || t.emptySide}</pre>
            </div>
            <div class="side theirs" class:picked={choice === 'theirs' || choice === 'both'} class:dropped={choice === 'ours'}>
              <div class="bar">
                <span>{t.theirs}<small>{block.theirs_label}</small></span>
                <button onclick={() => (choices[index] = 'theirs')}>{t.useThis}</button>
                <button onclick={() => (choices[index] = 'both')}>{t.useBoth}</button>
              </div>
              <pre>{block.theirs || t.emptySide}</pre>
            </div>
          </section>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .conflict {
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg);
  }
  .note {
    margin: 0;
    padding: 24px;
    text-align: center;
    color: var(--muted);
  }
  header {
    flex: none;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--panel);
  }
  header span {
    flex: 1;
    color: var(--yellow);
  }
  .body {
    flex: 1;
    overflow: auto;
    user-select: text;
  }
  pre {
    margin: 0;
    padding: 4px 14px;
    font: 12px/20px var(--mono);
    white-space: pre;
    tab-size: 4;
  }
  .same {
    color: var(--muted);
  }
  section {
    margin: 6px 0;
    border-block: 1px solid var(--border);
  }
  .side {
    border-left: 3px solid var(--accent);
    background: color-mix(in srgb, var(--accent) 8%, transparent);
  }
  .side.theirs {
    border-left-color: var(--green);
    background: color-mix(in srgb, var(--green) 8%, transparent);
  }
  .side.picked {
    box-shadow: inset 0 0 0 1px currentColor;
  }
  .side.dropped {
    opacity: 0.35;
  }
  .side.dropped pre {
    text-decoration: line-through;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 10px;
    font-size: 12px;
  }
  .bar span {
    flex: 1;
    font-weight: 600;
  }
  small {
    margin-left: 8px;
    font-weight: 400;
    color: var(--muted);
  }
  button {
    padding: 2px 10px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--panel);
    font-size: 12px;
    cursor: pointer;
  }
  button:hover:enabled {
    border-color: var(--accent);
    color: var(--accent);
  }
  .primary {
    border-color: var(--accent);
    background: var(--accent);
    color: #fff;
  }
  .primary:hover:enabled {
    color: #fff;
    filter: brightness(1.1);
  }
  button:disabled {
    opacity: 0.45;
    cursor: default;
  }
</style>
