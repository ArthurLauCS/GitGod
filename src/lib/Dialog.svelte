<script lang="ts" module>
  import type { Explain } from './explain'

  export interface Field {
    key: string
    label: string
    type: 'text' | 'checkbox' | 'radio'
    value?: string | boolean
    /** radio 的选项；radio 没有默认值时必须选一个才能确认 */
    options?: { value: string; label: string }[]
    /** 文本框是否允许留空 */
    optional?: boolean
  }
  export interface Spec {
    title: string
    /** 白话说明和操作前后的示意图 */
    explain?: Explain
    /** 给了这个键，对话框会多一个「下次不再显示」的勾选框，勾选后记在 localStorage 的 skip:<键> */
    skipKey?: string
    message?: string
    /** 醒目的警告，显示为红色 */
    warning?: string
    confirm?: string
    danger?: boolean
    fields?: Field[]
  }
  export type Values = Record<string, string | boolean>
</script>

<script lang="ts">
  import { tick } from 'svelte'
  import Explainer from './Explainer.svelte'
  import { t } from './zh'

  let dialog: HTMLDialogElement
  let skip = $state(false)
  let spec = $state<Spec | null>(null)
  let values = $state<Values>({})
  let resolve: (v: Values | null) => void = () => {}

  const ready = $derived(
    (spec?.fields ?? []).every((f) => f.type === 'checkbox' || f.optional || (values[f.key] ?? '') !== ''),
  )

  /** 弹出对话框；确认返回各字段的值，取消返回 null */
  export async function ask(s: Spec): Promise<Values | null> {
    spec = s
    skip = false
    values = Object.fromEntries((s.fields ?? []).map((f) => [f.key, f.value ?? (f.type === 'checkbox' ? false : '')]))
    // 等内容渲染出来再打开，然后把焦点放到第一个输入框；没有输入框时放到确认按钮上，回车即确认。
    // 危险操作不抢焦点，回车落在「取消」上。
    await tick()
    dialog.showModal()
    const target = dialog.querySelector<HTMLElement>('input[type=text]') ?? (s.danger ? null : dialog.querySelector<HTMLElement>('button[type=submit]'))
    target?.focus()
    return new Promise((r) => (resolve = r))
  }

  function close(ok: boolean) {
    dialog.close()
    if (ok && skip && spec?.skipKey) localStorage.setItem(`skip:${spec.skipKey}`, '1')
    resolve(ok ? $state.snapshot(values) : null)
  }
</script>

<dialog bind:this={dialog} oncancel={() => resolve(null)}>
  {#if spec}
    <form
      onsubmit={(e) => {
        e.preventDefault()
        if (ready) close(true)
      }}
    >
      <h2>{spec.title}</h2>
      {#if spec.explain}<Explainer explain={spec.explain} />{/if}
      {#if spec.message}<p>{spec.message}</p>{/if}
      {#if spec.warning}<p class="warning">{spec.warning}</p>{/if}
      {#each spec.fields ?? [] as f (f.key)}
        {#if f.type === 'text'}
          <!-- svelte-ignore a11y_autofocus -->
          <label class="text">{f.label}<input class="field" type="text" bind:value={values[f.key]} autofocus spellcheck="false" /></label>
        {:else if f.type === 'checkbox'}
          <label class="check"><input type="checkbox" bind:checked={values[f.key] as boolean} />{f.label}</label>
        {:else}
          <fieldset>
            <legend>{f.label}</legend>
            {#each f.options ?? [] as o (o.value)}
              <label class="check"><input type="radio" value={o.value} bind:group={values[f.key]} />{o.label}</label>
            {/each}
          </fieldset>
        {/if}
      {/each}
      {#if spec.skipKey}
        <label class="check skip"><input type="checkbox" bind:checked={skip} />{t.dontShowAgain}</label>
      {/if}
      <div class="buttons">
        <button type="button" class="btn" onclick={() => close(false)}>{t.cancel}</button>
        <button type="submit" class="btn primary" class:danger={spec.danger} disabled={!ready}>{spec.confirm ?? t.ok}</button>
      </div>
    </form>
  {/if}
</dialog>

<style>
  dialog {
    padding: 0;
    border: 1px solid var(--border-strong);
    border-radius: var(--r-lg);
    background: var(--panel);
    color: var(--text);
    box-shadow: var(--shadow-pop);
  }
  dialog::backdrop {
    background: var(--backdrop);
  }

  dialog {
    width: 560px;
    max-height: 90vh;
  }
  form {
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 24px;
  }
  h2 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: 600;
  }
  p {
    margin: 0;
    color: var(--muted);
    line-height: var(--lh-body);
    overflow-wrap: anywhere;
  }
  .warning {
    padding: 8px 12px;
    border-radius: var(--r-md);
    color: var(--red);
    background: color-mix(in srgb, var(--red) 12%, transparent);
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 4px;
    color: var(--muted);
    font-size: var(--fs-sm);
  }
  .text input {
    height: 32px;
    color: var(--text);
    font-size: var(--fs-md);
  }
  .check {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .skip {
    color: var(--muted);
    font-size: var(--fs-sm);
  }
  fieldset {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 0;
    padding: 0;
    border: 0;
  }
  legend {
    padding: 0 0 8px;
    color: var(--muted);
    font-size: var(--fs-sm);
  }
  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  .buttons .btn {
    min-width: 80px;
    height: 32px;
  }
</style>
