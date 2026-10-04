<script lang="ts">
  import { untrack } from 'svelte'
  import * as api from './api'
  import ContextMenu, { type Item } from './ContextMenu.svelte'
  import Detail from './Detail.svelte'
  import Dialog from './Dialog.svelte'
  import { explain, type ExplainKey } from './explain'
  import Graph from './Graph.svelte'
  import Help from './Help.svelte'
  import Sidebar from './Sidebar.svelte'
  import WorkingCopy from './WorkingCopy.svelte'
  import { t } from './zh'

  let { tab, initialCount, active }: { tab: number; initialCount: number; active: boolean } = $props()

  let count = $state(untrack(() => initialCount))
  let version = $state(0)
  let loadingAll = $state(false)
  let busy = $state(false)
  let error = $state('')
  let refs = $state.raw<api.Refs>({ head: null, head_id: null, ahead_behind: null, in_progress: null, refs: [] })
  let stashes = $state.raw<api.Stash[]>([])
  let worktrees = $state.raw<api.Worktree[]>([])
  let entries = $state.raw<api.Entry[]>([])
  let logs = $state.raw<api.Log[]>([])
  let showLog = $state(false)
  let view = $state<'history' | 'changes'>('history')
  let selectedRow = $state<number | null>(null)
  let detail = $state.raw<api.Detail | null>(null)
  let graph = $state<Graph>()
  let dialog = $state<Dialog>()
  let menu = $state<ContextMenu>()
  let help = $state<Help>()
  let pendingJump: string | null = null

  const badges = $derived.by(() => {
    const map = new Map<string, api.Ref[]>()
    for (const r of refs.refs) map.set(r.id, [...(map.get(r.id) ?? []), r])
    return map
  })
  /** 当前分支的短名；游离 HEAD 时为 null */
  const head = $derived(refs.head?.startsWith('refs/heads/') ? refs.head.slice(11) : null)
  const short = (name: string) => name.replace(/^refs\/(heads|tags|remotes)\//, '')

  async function guard<T>(p: Promise<T>): Promise<T | undefined> {
    try {
      return await p
    } catch (e) {
      error = String(e)
    }
  }

  async function loadStatus() {
    entries = (await guard(api.status(tab))) ?? entries
  }

  async function loadSidebar() {
    const r = await guard(Promise.all([api.refs(tab), api.stashes(tab), api.worktrees(tab), loadStatus()]))
    if (r) [refs, stashes, worktrees] = r
  }

  async function loadAll() {
    loadingAll = true
    const n = await guard(api.loadGraph(tab, true))
    loadingAll = false
    if (n === undefined) return
    count = n
    version++
    if (pendingJump) jump(pendingJump)
    pendingJump = null
  }

  async function select(row: number, id: string) {
    selectedRow = row
    const d = await guard(api.detail(tab, id))
    if (d && selectedRow === row) detail = d
  }

  async function jump(id: string) {
    const row = await guard(api.rowOf(tab, id))
    if (row == null) {
      // 完整历史还没加载完，目标提交不在首批里：等加载完再跳
      if (loadingAll) pendingJump = id
      return
    }
    view = 'history'
    graph?.scrollToRow(row, true)
    select(row, id)
  }

  /** 提交图只取决于各引用和 HEAD 指向哪些提交 */
  const tips = () => JSON.stringify([refs.refs.map((r) => r.name + r.id), refs.head_id])

  // 回到窗口、切回本页签或执行完操作后：引用指向有变化才重新加载提交图
  async function refresh() {
    if (loadingAll) return
    const before = tips()
    await loadSidebar()
    if (tips() === before) return
    selectedRow = null
    detail = null
    await loadAll()
    if (refs.head_id) jump(refs.head_id)
  }

  /** 执行一个写操作，记入命令日志；失败时展开日志。返回是否成功。 */
  async function exec(op: api.Op): Promise<boolean> {
    busy = true
    error = ''
    const log = await guard(api.op(tab, op))
    busy = false
    if (log) {
      logs = [...logs, log]
      if (!log.ok) showLog = true
    }
    await refresh()
    return log?.ok ?? false
  }

  const ask = (spec: Parameters<Dialog['ask']>[0]) => dialog!.ask(spec)

  /** 先用白话和示意图说明这个操作会发生什么，确认后再执行；勾过「不再显示」的直接执行 */
  async function explained(key: ExplainKey, title: string, op: api.Op) {
    if (localStorage.getItem(`skip:${key}`) || (await ask({ title, explain: explain[key], skipKey: key }))) exec(op)
  }

  async function newBranch(start: string) {
    const v = await ask({
      title: t.newBranch,
      explain: explain.create_branch,
      fields: [
        { key: 'name', label: t.branchName, type: 'text' },
        { key: 'checkout', label: t.checkoutAfterCreate, type: 'checkbox', value: true },
      ],
    })
    if (v) exec({ op: 'create_branch', name: v.name as string, start, checkout: v.checkout as boolean })
  }

  async function stash() {
    const v = await ask({
      title: t.stashTitle,
      explain: explain.stash,
      fields: [
        { key: 'message', label: t.stashMessage, type: 'text', optional: true },
        { key: 'untracked', label: t.includeUntracked, type: 'checkbox', value: true },
      ],
    })
    if (v) exec({ op: 'stash_push', message: v.message as string, include_untracked: v.untracked as boolean })
  }

  async function push(branch: string | null) {
    if (!branch) return void (error = t.noBranch)
    const remotes = (await guard(api.remotes(tab))) ?? []
    if (!remotes.length) return void (error = t.noRemote)
    const upstream = refs.refs.find((r) => r.name === `refs/heads/${branch}`)?.upstream
    // 远程名可以带斜杠，用已知的远程列表来拆 refs/remotes/<远程>/<分支>
    const remote = remotes.find((r) => upstream?.startsWith(`refs/remotes/${r}/`))
    const upBranch = remote && upstream!.slice(`refs/remotes/${remote}/`.length)
    const force = { key: 'force', label: t.forcePush, type: 'checkbox' as const }

    if (remote && upBranch === branch) {
      const v = await ask({
        title: t.pushTitle(branch),
        explain: explain.push,
        message: t.pushTo(`${remote}/${branch}`),
        confirm: t.push,
        fields: [force],
      })
      if (v) exec({ op: 'push', remote, branch, remote_branch: branch, force: v.force as boolean, set_upstream: false })
    } else if (remote && upBranch) {
      // 上游与本地分支不同名：不给默认值，必须明确选一个目标
      const v = await ask({
        title: t.pushTitle(branch),
        warning: t.pushMismatch(branch, `${remote}/${upBranch}`),
        confirm: t.push,
        fields: [
          {
            key: 'target',
            label: t.pushTarget,
            type: 'radio',
            options: [
              { value: 'upstream', label: t.pushToUpstream(`${remote}/${upBranch}`) },
              { value: 'same', label: t.pushToSameName(`${remote}/${branch}`) },
            ],
          },
          force,
        ],
      })
      if (!v) return
      const same = v.target === 'same'
      exec({ op: 'push', remote, branch, remote_branch: same ? branch : upBranch, force: v.force as boolean, set_upstream: same })
    } else {
      const v = await ask({
        title: t.pushTitle(branch),
        explain: explain.push,
        message: t.pushNew(`${remotes.length === 1 ? remotes[0] : '<' + t.remote + '>'}/${branch}`),
        confirm: t.push,
        fields: [
          ...(remotes.length > 1
            ? [{ key: 'remote', label: t.remote, type: 'radio' as const, value: remotes[0], options: remotes.map((r) => ({ value: r, label: r })) }]
            : []),
          force,
        ],
      })
      if (v) exec({ op: 'push', remote: (v.remote as string) ?? remotes[0], branch, remote_branch: branch, force: v.force as boolean, set_upstream: true })
    }
  }

  async function reset(target: string) {
    const v = await ask({
      title: t.resetTitle(head ?? 'HEAD'),
      explain: explain.reset,
      danger: true,
      fields: [
        {
          key: 'mode',
          label: t.resetMode,
          type: 'radio',
          value: 'mixed',
          options: [
            { value: 'soft', label: t.resetSoft },
            { value: 'mixed', label: t.resetMixed },
            { value: 'hard', label: t.resetHard },
          ],
        },
      ],
    })
    if (v) exec({ op: 'reset', target, mode: v.mode as 'soft' | 'mixed' | 'hard' })
  }

  type Kind = 'branch' | 'remote' | 'tag' | 'stash'

  function activate(kind: Kind, name: string) {
    if (kind === 'branch') exec({ op: 'checkout', target: short(name) })
    else if (kind === 'tag') explained('checkout_commit', short(name), { op: 'checkout', target: short(name) })
    else if (kind === 'remote') explained('track', short(name), { op: 'track', remote_branch: short(name) })
    else exec({ op: 'stash_apply', name, pop: false })
  }

  function refMenu(e: MouseEvent, kind: Kind, name: string) {
    const s = short(name)
    const current = name === refs.head
    const onto = head ?? 'HEAD'
    const items: Item[] = []
    if (kind === 'stash') {
      items.push(
        { label: t.stashApply, hint: '把收起来的改动放回工作区，贮藏保留', action: () => exec({ op: 'stash_apply', name, pop: false }) },
        { label: t.stashPop, hint: '放回工作区，并从贮藏列表里移除', action: () => exec({ op: 'stash_apply', name, pop: true }) },
        null,
        {
          label: t.stashDrop,
          hint: explain.stash_drop.short,
          danger: true,
          action: async () => {
            if (await ask({ title: t.dropStashTitle, warning: t.dropStashWarning, danger: true })) exec({ op: 'stash_drop', name })
          },
        },
      )
    } else {
      if (!current) {
        items.push(
          {
            label: kind === 'remote' ? t.checkoutRemote : kind === 'tag' ? t.checkoutCommit : t.checkout,
            hint: explain[kind === 'remote' ? 'track' : kind === 'tag' ? 'checkout_commit' : 'checkout'].short,
            action: () => activate(kind, name),
          },
          { label: t.mergeInto(onto), hint: explain.merge.short, action: () => explained('merge', t.mergeInto(onto), { op: 'merge', target: s }) },
          { label: t.rebaseOnto(onto), hint: explain.rebase.short, action: () => explained('rebase', t.rebaseOnto(onto), { op: 'rebase', onto: s }) },
          null,
        )
      }
      if (kind === 'branch') {
        items.push(
          { label: t.pushBranch, hint: explain.push.short, action: () => push(s) },
          {
            label: t.rename,
            hint: '只改本地分支的名字',
            action: async () => {
              const v = await ask({ title: t.rename, fields: [{ key: 'name', label: t.newName, type: 'text', value: s }] })
              if (v) exec({ op: 'rename_branch', old: s, new: v.name as string })
            },
          },
        )
        if (!current) {
          items.push(null, {
            label: t.delete,
            hint: explain.delete_branch.short,
            danger: true,
            action: async () => {
              const v = await ask({ title: t.deleteBranchTitle(s), explain: explain.delete_branch, danger: true, fields: [{ key: 'force', label: t.forceDelete, type: 'checkbox' }] })
              if (v) exec({ op: 'delete_branch', name: s, force: v.force as boolean })
            },
          })
        }
      } else if (kind === 'tag') {
        items.push({
          label: t.deleteTag,
          danger: true,
          action: async () => {
            if (await ask({ title: t.deleteTagTitle(s), danger: true })) exec({ op: 'delete_tag', name: s })
          },
        })
      } else {
        items.pop()
      }
    }
    menu!.show(e, items)
  }

  function commitMenu(e: MouseEvent, id: string) {
    const onto = head ?? 'HEAD'
    menu!.show(e, [
      {
        label: t.checkoutCommit,
        hint: explain.checkout_commit.short,
        action: () => explained('checkout_commit', t.checkoutCommit, { op: 'checkout', target: id }),
      },
      { label: t.newBranchHere, hint: explain.create_branch.short, action: () => newBranch(id) },
      {
        label: t.newTagHere,
        hint: explain.create_tag.short,
        action: async () => {
          const v = await ask({
            title: t.newTagHere,
            explain: explain.create_tag,
            fields: [
              { key: 'name', label: t.tagName, type: 'text' },
              { key: 'message', label: t.tagMessage, type: 'text', optional: true },
            ],
          })
          if (v) exec({ op: 'create_tag', name: v.name as string, target: id, message: v.message as string })
        },
      },
      null,
      { label: t.cherryPick, hint: explain.cherry_pick.short, action: () => explained('cherry_pick', t.cherryPick, { op: 'cherry_pick', id }) },
      { label: t.revertCommit, hint: explain.revert.short, action: () => explained('revert', t.revertCommit, { op: 'revert', id }) },
      { label: t.resetTo(onto), hint: explain.reset.short, danger: true, action: () => reset(id) },
      null,
      { label: t.copyId, action: () => navigator.clipboard.writeText(id) },
    ])
  }

  let started = false
  $effect(() => {
    if (!active) return
    if (started) {
      untrack(refresh)
      return
    }
    // 首次激活才加载完整历史，恢复多个页签时不会同时全量加载
    started = true
    untrack(async () => {
      await loadSidebar()
      if (refs.head_id) jump(refs.head_id)
      await loadAll()
    })
  })
</script>

<svelte:window onfocus={() => active && started && !busy && refresh()} />

<div class="repo" class:hidden={!active}>
  <header>
    <span class="branch">{head ?? t.detached}</span>
    {#if refs.ahead_behind}
      <span class="muted" title={t.aheadBehindHint}>{t.toPush(refs.ahead_behind[0])} · {t.toPull(refs.ahead_behind[1])}</span>
    {/if}
    <span class="muted">{busy ? t.working : loadingAll ? t.loadingHistory : t.commits(count)}</span>
    <span class="spacer"></span>
    <button disabled={busy} title={explain.fetch.short} onclick={() => exec({ op: 'fetch' })}>{t.fetch}</button>
    <button disabled={busy} title={explain.pull.short} onclick={() => explained('pull', t.pull, { op: 'pull' })}>{t.pull}</button>
    <button disabled={busy} title={explain.push.short} onclick={() => push(head)}>{t.push}</button>
    <button disabled={busy} title={explain.stash.short} onclick={stash}>{t.stash}</button>
    <button disabled={busy} title={explain.create_branch.short} onclick={() => newBranch('HEAD')}>{t.newBranch}</button>
    <button class:on={showLog} title={t.logHint} onclick={() => (showLog = !showLog)}>{t.log}</button>
    <button class="help" onclick={() => help!.open()}>? {t.help}</button>
  </header>
  {#if error}<p class="error">{error}</p>{/if}
  {#if refs.in_progress}
    {@const what = refs.in_progress}
    <p class="progress">
      <strong>{t.inProgress[what]}</strong>
      <span>{t.inProgressHint}</span>
      <button disabled={busy} onclick={() => exec({ op: 'continue', what })}>{t.continue}</button>
      <button disabled={busy} onclick={() => exec({ op: 'abort', what })}>{t.abort}</button>
    </p>
  {/if}
  <main>
    <Sidebar
      {refs}
      {stashes}
      {worktrees}
      {view}
      changes={entries.length}
      onview={(v) => (view = v)}
      onjump={jump}
      onactivate={activate}
      onmenu={(e, kind, name) => refMenu(e, kind, name)}
    />
    <div class="content">
      <div class="pane" class:hidden={view !== 'history'}>
        {#if count}
          <Graph
            bind:this={graph}
            fetchRows={(start, n) => api.rows(tab, start, n)}
            {count}
            {version}
            {badges}
            headId={refs.head_id}
            {selectedRow}
            onselect={select}
            onmenu={commitMenu}
          />
        {:else}
          <p class="none">{t.noCommits}</p>
        {/if}
        <Detail {detail} onjump={jump} fetchDiff={(id, path) => api.diffCommit(tab, id, path)} />
      </div>
      <div class="pane" class:hidden={view !== 'changes'}>
        <WorkingCopy {tab} {entries} reload={loadStatus} oncommitted={refresh} />
      </div>
    </div>
  </main>
  {#if showLog}
    <div class="log">
      {#each logs as log, i (i)}
        <div class:failed={!log.ok}>
          <code>$ {log.command}</code>
          {#if log.output}<pre>{log.output}</pre>{/if}
        </div>
      {:else}
        <p class="muted">{t.logEmpty}</p>
      {/each}
    </div>
  {/if}
</div>

<Dialog bind:this={dialog} />
<ContextMenu bind:this={menu} />
<Help bind:this={help} />

<style>
  .repo {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  /* 不用 display: none：那样会丢掉提交图的滚动位置 */
  .hidden {
    position: absolute;
    inset: 0;
    visibility: hidden;
  }
  header {
    flex: none;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 40px;
    padding: 0 10px 0 14px;
    background: var(--panel);
    border-bottom: 1px solid var(--border);
  }
  .branch {
    padding: 1px 8px;
    border-radius: 4px;
    color: var(--accent);
    background: var(--accent-soft);
    font-size: 12px;
  }
  .muted {
    color: var(--muted);
    font-size: 12px;
  }
  .spacer {
    flex: 1;
  }
  header button,
  .progress button {
    padding: 3px 12px;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--raised);
    cursor: pointer;
  }
  header button:hover:enabled,
  .progress button:hover:enabled {
    border-color: var(--muted);
  }
  header button.on,
  header button.help {
    border-color: var(--accent);
    color: var(--accent);
  }
  button:disabled {
    opacity: 0.45;
    cursor: default;
  }
  main {
    flex: 1;
    min-height: 0;
    display: flex;
  }
  .content {
    flex: 1;
    min-width: 0;
    position: relative;
    display: flex;
  }
  .pane {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .none {
    flex: 1;
    display: grid;
    place-items: center;
    margin: 0;
    color: var(--muted);
  }
  .error {
    margin: 0;
    padding: 6px 14px;
    color: var(--red);
    background: color-mix(in srgb, var(--red) 12%, transparent);
    user-select: text;
  }
  .progress {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 0;
    padding: 6px 14px;
    color: var(--yellow);
    background: color-mix(in srgb, var(--yellow) 12%, transparent);
  }
  .progress span {
    flex: 1;
    color: var(--text);
  }
  .progress strong {
    flex: none;
  }
  .log {
    flex: none;
    height: 180px;
    overflow: auto;
    padding: 8px 14px;
    border-top: 1px solid var(--border);
    background: var(--bg);
    font: 12px/1.6 var(--mono);
    user-select: text;
  }
  .log div {
    margin-bottom: 8px;
  }
  .log code {
    color: var(--accent);
  }
  .log pre {
    margin: 0;
    font: inherit;
    color: var(--muted);
    white-space: pre-wrap;
  }
  .log .failed code,
  .log .failed pre {
    color: var(--red);
  }
</style>
