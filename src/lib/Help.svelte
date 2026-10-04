<script lang="ts">
  import { explain } from './explain'
  import Explainer from './Explainer.svelte'
  import { t } from './zh'

  let dialog: HTMLDialogElement
  let reset = $state(false)

  export function open() {
    reset = false
    dialog.showModal()
  }

  /** 让所有勾过「下次不再显示」的说明重新出现 */
  function showAgain() {
    for (const key of Object.keys(explain)) localStorage.removeItem(`skip:${key}`)
    reset = true
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
<dialog bind:this={dialog} onclick={(e) => e.target === dialog && dialog.close()}>
  <header>
    <h2>{t.help}</h2>
    <button disabled={reset} onclick={showAgain}>{reset ? t.helpResetDone : t.helpReset}</button>
    <button onclick={() => dialog.close()}>{t.close}</button>
  </header>
  <p class="legend">{t.helpLegend}</p>
  <div class="list">
    {#each Object.values(explain) as e (e.name)}
      <section>
        <h3>{e.name}</h3>
        <Explainer explain={e} />
      </section>
    {/each}
  </div>
</dialog>

<style>
  dialog {
    width: 640px;
    max-height: 84vh;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 9px;
    background: var(--panel);
    color: var(--text);
    box-shadow: 0 18px 50px rgba(0, 0, 0, 0.45);
  }
  dialog[open] {
    display: flex;
    flex-direction: column;
  }
  dialog::backdrop {
    background: rgba(0, 0, 0, 0.4);
  }
  header {
    flex: none;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 14px 20px 8px;
  }
  h2 {
    flex: 1;
    margin: 0;
    font-size: 15px;
    font-weight: 600;
  }
  .legend {
    flex: none;
    margin: 0;
    padding: 0 20px 12px;
    color: var(--muted);
    font-size: 12px;
    border-bottom: 1px solid var(--border);
  }
  .list {
    overflow-y: auto;
    padding: 4px 20px 20px;
    user-select: text;
  }
  section {
    padding: 16px 0;
    border-bottom: 1px solid var(--border);
  }
  section:last-child {
    border-bottom: 0;
  }
  h3 {
    margin: 0 0 8px;
    font-size: 14px;
    font-weight: 600;
    color: var(--accent);
  }
  button {
    padding: 4px 12px;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--raised);
    cursor: pointer;
  }
  button:hover:enabled {
    border-color: var(--muted);
  }
  button:disabled {
    opacity: 0.6;
    cursor: default;
  }
</style>
