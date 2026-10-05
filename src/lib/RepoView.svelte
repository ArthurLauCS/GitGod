<script lang="ts">
  import { untrack } from 'svelte'
  import * as api from './api'
  import ContextMenu, { type Item } from './ContextMenu.svelte'
  import Detail from './Detail.svelte'
  import Dialog from './Dialog.svelte'
  import type { ExplainKey } from './explain'
  import { explain } from './i18n.svelte'
  import Graph from './Graph.svelte'
  import { graphKey } from './graph-key'
  import Help from './Help.svelte'
  import Icon from './Icon.svelte'
  import { splitUpstream } from './push-target'
  import Splitter from './Splitter.svelte'
  import Sidebar from './Sidebar.svelte'
  import WorkingCopy from './WorkingCopy.svelte'
  import Worktrees from './Worktrees.svelte'
  import { t, errorText } from './i18n.svelte'

  let {
    tab,
    path,
    initialCount,
    active,
    onopen,
  }: {
    tab: number
    /** 这个页签的工作区路径 */
    path: string
    initialCount: number
    active: boolean
    /** 在（新）页签中打开另一个仓库或工作树 */
    onopen: (path: string) => void
  } = $props()

  let count = $state(untrack(() => initialCount))
  let version = $state(0)
  let loadingAll = $state(false)
  let busy = $state(false)
  let error = $state('')
  let refs = $state.raw<api.Refs>({ head: null, head_id: null, ahead_behind: null, in_progress: null, refs: [] })
  let stashes = $state.raw<api.Stash[]>([])
  let worktrees = $state.raw<api.Worktree[]>([])
  let entries = $state.raw<api.Entry[]>([])
  let identity = $state.raw<api.Identity | null>(null)
  let logs = $state.raw<api.Log[]>([])
  let showLog = $state(false)
  let view = $state<'history' | 'changes' | 'worktrees'>('history')
  let tracks = $state.raw(new Map<string, api.Track>())
  let selectedRow = $state<number | null>(null)
  let detail = $state.raw<api.Detail | null>(null)
  let graph = $state<Graph>()
  let dialog = $state<Dialog>()
  let menu = $state<ContextMenu>()
  let help = $state<Help>()
  let pendingJump: string | null = null
  /** 可撤销的操作，最近的在最后。每项是把仓库退回去要执行的操作 */
  let undos = $state.raw<{ title: string; ops: api.Op[] }[]>([])

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
    const r = await guard(Promise.all([api.refs(tab), api.stashes(tab), api.worktrees(tab), loadStatus(), api.commitIdentity(tab)]))
    if (r) [refs, stashes, worktrees, , identity] = r
    // 各分支的同步状态在分支多时要算一两秒，不等它
    api.tracking(tab).then(
      (list) => (tracks = new Map(list.map((x) => [x.name, x]))),
      () => {},
    )
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

  // VS Code 扩展从编辑器跳到某个提交
  $effect(() => {
    const reveal = (e: Event) => active && jump((e as CustomEvent<string>).detail)
    addEventListener('pushright:jump', reveal)
    return () => removeEventListener('pushright:jump', reveal)
  })

  // 回到窗口、切回本页签或执行完操作后：引用指向有变化才重新加载提交图
  async function refresh() {
    if (loadingAll) return
    const before = graphKey(refs)
    const beforeHead = refs.head_id
    await loadSidebar()
    if (graphKey(refs) === before) {
      if (refs.head_id && refs.head_id !== beforeHead && view === 'history') jump(refs.head_id)
      return
    }
    selectedRow = null
    detail = null
    await loadAll()
    // 正在看本地更改或工作树时不把人拽回提交图
    if (refs.head_id && view === 'history') jump(refs.head_id)
  }

  /** 根据操作之前的状态，算出把它退回去要执行什么；不能撤销的返回 null */
  function undoFor(op: api.Op, before: api.Refs): api.Op[] | null {
    const idOf = (name: string) => before.refs.find((r) => r.name === name)?.id
    const back: api.Op | null = before.head ? { op: 'checkout', target: short(before.head) } : before.head_id ? { op: 'checkout', target: before.head_id } : null
    switch (op.op) {
      case 'checkout':
      case 'track':
        return back && [back]
      case 'create_branch':
        return [...(op.checkout && back ? [back] : []), { op: 'delete_branch', name: op.name, force: true }]
      case 'delete_branch': {
        const id = idOf(`refs/heads/${op.name}`)
        return id ? [{ op: 'create_branch', name: op.name, start: id, checkout: false }] : null
      }
      case 'create_tag':
        return [{ op: 'delete_tag', name: op.name }]
      case 'delete_tag': {
        const id = idOf(`refs/tags/${op.name}`)
        return id ? [{ op: 'create_tag', name: op.name, target: id, message: '' }] : null
      }
      case 'merge':
      case 'rebase':
      case 'cherry_pick':
      case 'revert':
      case 'pull':
        return before.head_id ? [{ op: 'reset', target: before.head_id, mode: 'keep' }] : null
      case 'reset':
        // 软/混合重置后改动还在工作区，用同样的方式移回去正好还原；硬重置后工作区是干净的，用 keep
        return before.head_id ? [{ op: 'reset', target: before.head_id, mode: op.mode === 'hard' ? 'keep' : op.mode }] : null
      case 'stash_push':
      case 'discard':
        return [{ op: 'stash_apply', name: 'stash@{0}', pop: true }]
      default:
        return null
    }
  }

  /** 执行一个写操作，记入命令日志；失败时展开日志。`record` 为 false 时不进撤销列表。返回是否成功。 */
  async function exec(op: api.Op, record = true): Promise<boolean> {
    busy = true
    error = ''
    const before = refs
    const log = await guard(api.op(tab, op))
    busy = false
    if (log) {
      logs = [...logs, log]
      if (!log.ok) showLog = true
    }
    const ok = log?.ok ?? false
    const ops = ok && record ? undoFor(op, before) : null
    if (ops) undos = [...undos, { title: op.op, ops }]
    await refresh()
    return ok
  }

  async function undo() {
    const last = undos.at(-1)
    if (!last || !(await ask({ title: t.undoTitle(t.opNames[last.title] ?? last.title), explain: explain.undo, confirm: t.undo }))) return
    undos = undos.slice(0, -1)
    for (const op of last.ops) if (!(await exec(op, false))) break
  }

  /** 提交不走 exec，单独记一条撤销：软重置回去，改动回到已暂存 */
  async function committed() {
    const before = refs.head_id
    await refresh()
    if (before) undos = [...undos, { title: 'commit', ops: [{ op: 'reset', target: before, mode: 'soft' }] }]
  }

  async function discard(paths: string[]) {
    const v = await ask({ title: t.discardTitle(paths.length), explain: explain.discard, danger: true, confirm: t.discardConfirm })
    if (v) exec({ op: 'discard', paths })
  }

  async function discardLines(file: string, hunk: number, header: string, lines: number[]) {
    if (!(await ask({ title: t.discardLinesTitle, explain: explain.discard, danger: true, confirm: t.discardConfirm }))) return
    await guard(api.discardLines(tab, file, hunk, header, lines))
    await refresh()
  }

  const ask = (spec: Parameters<Dialog['ask']>[0]) => dialog!.ask(spec)

  async function editIdentity() {
    identity = (await guard(api.commitIdentity(tab))) ?? identity
    const v = await ask({
      title: t.editIdentity,
      message: t.identityScope,
      fields: [
        { key: 'name', label: t.identityName, type: 'text', value: identity?.name ?? '' },
        { key: 'email', label: t.identityEmail, type: 'text', value: identity?.email ?? '' },
      ],
    })
    if (!v) return
    busy = true
    error = ''
    identity = (await guard(api.setCommitIdentity(tab, v.name as string, v.email as string)))
      ?? (await guard(api.commitIdentity(tab))) ?? identity
    busy = false
    const expected = `${(v.name as string).trim()} <${(v.email as string).trim()}>`
    if (!error && (identity?.author !== expected || identity?.committer !== expected)) error = t.identityOverridden
  }

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
    const { remote, branch: upBranch } = splitUpstream(upstream, remotes) ?? { remote: undefined, branch: undefined }
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
      // 上游与本地分支不同名：先推荐同名分支，仍需明确选一个目标
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
              { value: 'same', label: t.pushToSameName(`${remote}/${branch}`) },
              { value: 'upstream', label: t.pushToUpstream(`${remote}/${upBranch}`) },
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

  async function pull() {
    const upstream = refs.refs.find((r) => r.name === refs.head)?.upstream
    const v = await ask({
      title: t.pull,
      explain: explain.pull,
      message: t.pullFrom(upstream ? short(upstream) : t.noUpstream),
      confirm: t.pull,
      fields: [{ key: 'rebase', label: t.pullRebase, type: 'checkbox', value: true }],
    })
    if (v) exec({ op: 'pull', rebase: v.rebase as boolean })
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

  /** 切换到本地分支；有未提交的改动或分支已在别处打开时先问清楚 */
  async function switchTo(branch: string) {
    const other = worktrees.find((w) => !w.current && w.branch === `refs/heads/${branch}`)
    if (other) {
      const v = await ask({ title: t.switchDirtyTitle(branch), message: t.inOtherWorktree(branch, other.path), confirm: t.openInTab })
      if (v) onopen(other.path)
      return
    }
    // 未跟踪的文件不受切换影响
    if (!entries.some((e) => e.unstaged !== '?')) return void exec({ op: 'checkout', target: branch })
    const v = await ask({
      title: t.switchDirtyTitle(branch),
      explain: explain.switch_dirty,
      fields: [
        {
          key: 'how',
          label: t.switchHow,
          type: 'radio',
          value: 'stash',
          options: [
            { value: 'stash', label: t.switchStash },
            { value: 'carry', label: t.switchCarry },
            { value: 'worktree', label: t.switchWorktree },
          ],
        },
      ],
    })
    if (!v) return
    if (v.how === 'carry') exec({ op: 'checkout', target: branch })
    else if (v.how === 'worktree') addWorktree(branch, true)
    else if (await exec({ op: 'stash_push', message: t.autoStash(branch), include_untracked: false }, false)) {
      // 切换失败时改动也要放回来，所以不看 checkout 的结果
      await exec({ op: 'checkout', target: branch })
      await exec({ op: 'stash_apply', name: 'stash@{0}', pop: true }, false)
    }
  }

  /** 新建工作树并在页签中打开。`existing` 为 true 时直接检出 `start` 这个分支，否则从 `start` 新建分支 */
  async function addWorktree(start: string, existing: boolean) {
    const v = await ask({
      title: t.newWorktree,
      explain: explain.worktree,
      fields: [
        ...(existing ? [] : [{ key: 'branch', label: t.branchName, type: 'text' as const }]),
        { key: 'path', label: t.worktreePath, type: 'text', value: `${path}-${existing ? start.replace(/[\\/]/g, '-') : 'worktree'}` },
      ],
    })
    if (!v) return
    const dir = v.path as string
    if (await exec({ op: 'worktree_add', path: dir, start, new_branch: existing ? null : (v.branch as string) })) onopen(dir)
  }

  async function removeWorktree(w: api.Worktree) {
    const v = await ask({
      title: t.wtRemoveTitle(w.path.split(/[\\/]/).filter(Boolean).pop() ?? w.path),
      message: explain.worktree.undo,
      warning: w.changes ? t.wtRemoveWarning(w.changes) : undefined,
      danger: true,
    })
    if (v) exec({ op: 'worktree_remove', path: w.path, force: w.changes > 0 })
  }

  type Kind = 'branch' | 'remote' | 'tag' | 'stash'

  function activate(kind: Kind, name: string) {
    if (kind === 'branch') switchTo(short(name))
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
        { label: t.stashApply, hint: t.stashApplyHint, action: () => exec({ op: 'stash_apply', name, pop: false }) },
        { label: t.stashPop, hint: t.stashPopHint, action: () => exec({ op: 'stash_apply', name, pop: true }) },
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
          ...(current ? [] : [{ label: t.openInWorktree, hint: explain.worktree.short, action: () => addWorktree(s, true) }]),
          {
            label: t.rename,
            hint: t.renameHint,
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
    <div class="tools">
      <button class="btn quiet" disabled={busy || !undos.length} title={undos.length ? t.undoTitle(t.opNames[undos.at(-1)!.title] ?? undos.at(-1)!.title) : t.undoNothing} onclick={undo}>
        <Icon name="undo" /><span>{t.undo}</span>
      </button>
      <span class="sep"></span>
      <button class="btn quiet" disabled={busy} title={explain.fetch.short} onclick={() => exec({ op: 'fetch' })}>
        <Icon name="fetch" /><span>{t.fetch}</span>
      </button>
      <button class="btn quiet" disabled={busy || !!refs.in_progress} title={explain.pull.short} onclick={pull}>
        <Icon name="pull" /><span>{t.pull}</span>
      </button>
      <button class="btn quiet" disabled={busy} title={explain.push.short} onclick={() => push(head)}>
        <Icon name="push" /><span>{t.push}</span>
      </button>
    </div>
    <!-- 当前分支放在正中：这是整个界面里最需要随时确认的一件事 -->
    <div class="current" class:detached={!head} title={t.currentBranch}>
      <Icon name="branch" size={18} />
      <strong>{head ?? t.detached}</strong>
      <span class="state" title={t.aheadBehindHint}>
        {#if busy}{t.working}
        {:else if loadingAll}{t.loadingHistory}
        {:else if refs.ahead_behind && refs.ahead_behind[0] + refs.ahead_behind[1]}{t.toPush(refs.ahead_behind[0])} · {t.toPull(refs.ahead_behind[1])}
        {:else if refs.ahead_behind}{t.wtSynced}
        {:else}{t.commits(count)}{/if}
      </span>
    </div>
    <div class="tools end">
      <button class="btn quiet" disabled={busy} title={explain.stash.short} onclick={stash}><Icon name="stash" /><span>{t.stash}</span></button>
      <button class="btn quiet" disabled={busy} title={explain.create_branch.short} onclick={() => newBranch('HEAD')}>
        <Icon name="branch" /><span>{t.newBranch}</span>
      </button>
      <span class="sep"></span>
      <button class="btn quiet" class:on={showLog} title={t.logHint} onclick={() => (showLog = !showLog)}><Icon name="log" /><span>{t.log}</span></button>
      <button class="btn quiet" title={t.help} onclick={() => help!.open()}><Icon name="help" /><span>{t.help}</span></button>
    </div>
  </header>
  {#if error}<p class="error">{errorText(error)}</p>{/if}
  {#if refs.in_progress}
    {@const what = refs.in_progress}
    <p class="progress">
      <strong>{t.inProgress[what]}</strong>
      <span>{t.inProgressHint}</span>
      <button class="btn small primary" disabled={busy} onclick={() => exec({ op: 'continue', what })}>{t.continue}</button>
      <button class="btn small" disabled={busy} onclick={() => exec({ op: 'abort', what })}>{t.abort}</button>
    </p>
  {/if}
  <main>
    <Sidebar
      {refs}
      {stashes}
      {worktrees}
      {tracks}
      {view}
      changes={entries.length}
      onview={(v) => (view = v)}
      onjump={jump}
      onactivate={activate}
      onmenu={(e, kind, name) => refMenu(e, kind, name)}
      onworktree={onopen}
    />
    <Splitter key="sidebar" min={180} max={520} />
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
        <Splitter key="detail" dir="y" min={120} max={640} invert />
        <Detail {detail} onjump={jump} fetchDiff={(id, path) => api.diffCommit(tab, id, path)} />
      </div>
      <div class="pane" class:hidden={view !== 'changes'}>
        <WorkingCopy {tab} {entries} {identity} {editIdentity} identityBusy={busy} rebasing={refs.in_progress === 'rebase'} reload={refresh} oncommitted={committed} {discard} {discardLines} />
      </div>
      {#if view === 'worktrees'}
        <div class="pane">
          <Worktrees {worktrees} {onopen} onremove={removeWorktree} onnew={() => addWorktree('HEAD', false)} />
        </div>
      {/if}
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
  /* 左右两组工具等宽，当前分支才能落在正中 */
  header {
    flex: none;
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 16px;
    height: 56px;
    padding: 0 8px;
    background: var(--panel);
    border-bottom: 1px solid var(--border);
    container-type: inline-size;
  }
  .tools {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
  }
  .tools.end {
    justify-content: flex-end;
  }
  /* 窗口窄时工具栏只留图标 */
  @container (max-width: 1100px) {
    .tools .btn span {
      display: none;
    }
  }
  .current {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 40px;
    max-width: 44cqw;
    padding: 0 16px;
    border: 1px solid color-mix(in srgb, var(--accent) 45%, transparent);
    border-radius: var(--r-lg);
    background: var(--accent-soft);
    color: var(--accent);
  }
  .current strong {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--display);
    font-size: var(--fs-lg);
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .current .state {
    flex: none;
    padding-left: 8px;
    border-left: 1px solid color-mix(in srgb, var(--accent) 35%, transparent);
    color: var(--muted);
    font-size: var(--fs-sm);
    white-space: nowrap;
  }
  /* 游离状态不属于任何分支，换成警示色 */
  .current.detached {
    border-color: color-mix(in srgb, var(--yellow) 45%, transparent);
    background: color-mix(in srgb, var(--yellow) 14%, transparent);
    color: var(--yellow);
  }
  .sep {
    width: 1px;
    height: 16px;
    margin: 0 4px;
    background: var(--border);
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
    font-family: var(--prose);
    font-size: var(--fs-lg);
    flex: 1;
    display: grid;
    place-items: center;
    margin: 0;
    color: var(--muted);
  }
  .error {
    margin: 0;
    padding: 8px 16px;
    color: var(--red);
    background: color-mix(in srgb, var(--red) 12%, transparent);
    font-size: var(--fs-sm);
    user-select: text;
  }
  .progress {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 0;
    padding: 8px 16px;
    background: color-mix(in srgb, var(--yellow) 12%, transparent);
    font-size: var(--fs-sm);
  }
  .progress strong {
    flex: none;
    color: var(--yellow);
  }
  .progress span {
    flex: 1;
    font-family: var(--prose);
    font-size: var(--fs-md);
  }
  .log {
    flex: none;
    height: 180px;
    overflow: auto;
    padding: 8px 16px;
    border-top: 1px solid var(--border);
    background: var(--bg);
    font: var(--fs-sm) / 1.6 var(--mono);
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
