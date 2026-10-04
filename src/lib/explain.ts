// 每个操作的白话说明和操作前后的示意图。菜单提示、确认对话框、「操作说明」页都从这里取。
// 新增操作入口时在这里补一条。

export interface Node {
  id: string
  x: number
  y: number
  /** 0 蓝（当前这条线）1 绿（另一条线）2 黄（远程） */
  color: number
  /** new：这次操作新产生的；ghost：操作后从分支上消失的 */
  kind?: 'new' | 'ghost'
}
export interface Label {
  at: string
  text: string
  color: number
  /** 你当前所在的分支，画成实心 */
  here: boolean
}
export interface Graph {
  nodes: Node[]
  edges: [string, string][]
  labels: Label[]
}
export interface Explain {
  name: string
  /** 一行以内，用在菜单和按钮提示里 */
  short: string
  what: string
  result: string[]
  /** 做错了怎么办 */
  undo?: string
  before?: Graph
  after?: Graph
}

/**
 * 示意图的紧凑写法：
 * 节点 `id:x,y[:颜色[:new|ghost]]`，连线 `父>子`，标签 `节点=文字[:颜色][*]`（* 表示当前分支）
 */
function g(nodes: string, edges: string, labels: string): Graph {
  const words = (s: string) => s.split(' ').filter(Boolean)
  return {
    nodes: words(nodes).map((n) => {
      const [id, pos, color, kind] = n.split(':')
      const [x, y] = pos.split(',').map(Number)
      return { id, x, y, color: +(color || 0), kind: kind as Node['kind'] }
    }),
    edges: words(edges).map((e) => e.split('>') as [string, string]),
    labels: words(labels).map((l) => {
      const here = l.endsWith('*')
      const [at, rest] = l.replace('*', '').split('=')
      const [text, color] = rest.split(':')
      return { at, text, color: +(color || 0), here }
    }),
  }
}

// 两条分支的公共起点：main 上 A-B-C，feat 从 B 分出 D-E
const FORK = 'A:0,0 B:1,0 C:2,0 D:2,1:1 E:3,1:1'
const FORK_EDGES = 'A>B B>C B>D D>E'
const LINE = 'A:0,0 B:1,0 C:2,0'

export const explain = {
  stage: {
    name: '暂存',
    short: '选出哪些改动要进入下一次提交',
    what: '改完文件后，先挑出这次想提交的改动放进「已暂存」。没暂存的改动留在本地，不会被提交。',
    result: ['可以只暂存一个文件、一个区块，甚至几行', '暂存不会改变文件内容，只是打了个「要提交」的记号'],
    undo: '点「取消暂存」就退回去，改动本身不会丢。',
  },
  commit: {
    name: '提交',
    short: '把已暂存的改动存成一个历史记录点',
    what: '把「已暂存」里的改动保存成一个记录点，并写一句话说明改了什么。提交只保存在你自己电脑上。',
    result: ['历史里多出一个新的提交', '别人还看不到，要「推送」之后才会上传'],
    undo: '还没推送时可以勾选「修补上次提交」重新提交，或用「重置」退回。',
    before: g(LINE, 'A>B B>C', 'C=main*'),
    after: g(LINE + ' D:3,0::new', 'A>B B>C C>D', 'D=main*'),
  },
  checkout: {
    name: '切换分支（检出）',
    short: '把文件换成这个分支的版本，之后在它上面工作',
    what: '把工作区的文件换成另一个分支的版本。之后你做的提交会记在那个分支上。',
    result: ['文件内容变成目标分支的样子', '所有分支的提交历史都不变'],
    undo: '随时可以再切回来。',
    before: g(FORK, FORK_EDGES, 'C=main* E=feat:1'),
    after: g(FORK, FORK_EDGES, 'C=main E=feat:1*'),
  },
  checkout_commit: {
    name: '查看历史提交',
    short: '把文件换成这个提交时的样子',
    what: '把文件换成那次提交时的样子，用来查看或测试旧版本。',
    result: ['此时你不在任何分支上（称为「游离状态」）', '在这个状态下做的新提交不属于任何分支，切走之后很难找回；想保留请先「在此新建分支」'],
    undo: '双击任意分支就回去了。',
  },
  track: {
    name: '检出远程分支',
    short: '在本地建一个同名分支并跟它关联',
    what: '远程分支不能直接修改。这个操作会在本地建一个同名分支，并和远程分支关联起来。',
    result: ['本地多出一个同名分支，并切换过去', '之后可以直接「拉取」和「推送」'],
  },
  create_branch: {
    name: '新建分支',
    short: '从这里分出一条独立的开发线',
    what: '从当前位置分出一条新的开发线。在新分支上的提交不会影响原来的分支，做完再合并回去。',
    result: ['多出一个分支名，指向同一个提交', '文件内容不变'],
    undo: '不想要了可以直接删除分支。',
    before: g(LINE, 'A>B B>C', 'C=main*'),
    after: g(LINE, 'A>B B>C', 'C=main C=新分支:1*'),
  },
  delete_branch: {
    name: '删除分支',
    short: '只删除分支这个名字',
    what: '删除的只是分支这个名字。已经合并到其他分支的提交不会丢。',
    result: ['侧栏里不再有这个分支', '如果它上面有没合并过的提交，默认会拒绝删除；勾选「强制删除」才会删，那些提交就找不到了'],
    undo: '只删本地，远程的同名分支不受影响。',
  },
  merge: {
    name: '合并',
    short: '把它的改动并入当前分支',
    what: '把另一个分支的改动并入你当前所在的分支。两边都有新提交时，会生成一个「合并提交」把两条线接在一起。',
    result: ['当前分支得到对方的全部改动', '对方分支不变', '两边改了同一处时会产生冲突，需要你决定保留哪边'],
    undo: '可以用「重置」退回合并前。推送之前都不会影响别人。',
    before: g(FORK, FORK_EDGES, 'C=main* E=feat:1'),
    after: g(FORK + ' M:4,0::new', FORK_EDGES + ' C>M E>M', 'M=main* E=feat:1'),
  },
  rebase: {
    name: '变基',
    short: '把当前分支的提交挪到它后面，历史变成一条直线',
    what: '把当前分支上你自己的提交，挪到目标分支最新的提交后面重新做一遍。结果和合并一样拿到了对方的改动，但历史是一条直线，没有合并提交。',
    result: ['你的提交内容不变，但都变成了新的提交（ID 会变）', '目标分支不变', '如果这些提交之前已经推送过，之后必须强制推送，会影响同样在用这个分支的人'],
    undo: '只对还没推送的提交使用最安全。多人共用的分支不要变基。',
    before: g(FORK, FORK_EDGES, 'C=main E=feat:1*'),
    after: g("A:0,0 B:1,0 C:2,0 D:2,1:1:ghost E:3,1:1:ghost D':3,0:1:new E':4,0:1:new", "A>B B>C B>D D>E C>D' D'>E'", "C=main E'=feat:1*"),
  },
  cherry_pick: {
    name: '摘取提交（Cherry-pick）',
    short: '把这一个提交的改动复制到当前分支',
    what: '只把选中的这一个提交的改动复制一份到当前分支，不带上那个分支的其他提交。',
    result: ['当前分支多出一个内容相同的新提交', '原来的提交和它所在的分支不变'],
    undo: '可以用「撤销此提交」或「重置」去掉。',
    before: g(FORK, FORK_EDGES, 'C=main* E=feat:1'),
    after: g(FORK + " E':3,0:1:new", FORK_EDGES + " C>E'", "E'=main* E=feat:1"),
  },
  revert: {
    name: '撤销提交（Revert）',
    short: '新增一个提交，抵消这次提交的改动',
    what: '生成一个新的提交，内容正好抵消选中提交的改动。原来的提交仍然留在历史里。',
    result: ['文件回到那次提交之前的样子', '历史只增不减，对已经推送的提交也安全'],
    undo: '对这个新提交再撤销一次，就恢复了。',
    before: g(LINE, 'A>B B>C', 'C=main*'),
    after: g(LINE + ' -C:3,0::new', 'A>B B>C C>-C', '-C=main*'),
  },
  reset: {
    name: '重置',
    short: '把当前分支退回到这个提交',
    what: '把当前分支直接挪回到选中的提交，这之后的提交会从分支上消失。三种方式的区别在于怎么处理那些提交里的改动。',
    result: ['软重置：改动保留，并且是已暂存状态', '混合重置：改动保留，变成未暂存', '硬重置：改动全部丢弃，文件回到那个提交的样子'],
    undo: '硬重置会丢掉还没提交的改动，无法恢复。已经推送过的提交不要重置，改用「撤销提交」。',
    before: g(LINE, 'A>B B>C', 'C=main*'),
    after: g('A:0,0 B:1,0 C:2,0::ghost', 'A>B B>C', 'B=main*'),
  },
  fetch: {
    name: '获取',
    short: '看看远程有什么新提交，不改动你的文件',
    what: '从远程下载最新的提交记录，只更新「远程分支」的位置。你本地的分支和文件完全不动。',
    result: ['能看到别人推送了什么', '你的文件和本地分支不变'],
    undo: '非常安全，随时可以点。',
    before: g('A:0,0 B:1,0', 'A>B', 'B=main* B=origin/main:2'),
    after: g('A:0,0 B:1,0 C:2,0:2:new', 'A>B B>C', 'B=main* C=origin/main:2'),
  },
  pull: {
    name: '拉取',
    short: '下载远程的新提交并合并进当前分支',
    what: '先「获取」远程的新提交，再把它们合并进当前分支。开始工作前拉取一次，可以拿到别人最新的改动。',
    result: ['当前分支和文件更新到远程的最新状态', '你本地有还没推送的提交时，可能生成合并提交或产生冲突'],
    before: g('A:0,0 B:1,0', 'A>B', 'B=main* B=origin/main:2'),
    after: g('A:0,0 B:1,0 C:2,0::new', 'A>B B>C', 'C=main* C=origin/main:2'),
  },
  push: {
    name: '推送',
    short: '把本地的新提交上传到远程',
    what: '把本地分支上新的提交上传到远程。推送之后，别人拉取就能看到你的改动。',
    result: ['远程分支更新到和本地一致', '如果远程有你本地没有的提交，推送会被拒绝，需要先「拉取」'],
    undo: '推送后别人可能已经拉取，这些提交就不要再重置或变基了。',
    before: g(LINE, 'A>B B>C', 'C=main* B=origin/main:2'),
    after: g(LINE, 'A>B B>C', 'C=main* C=origin/main:2'),
  },
  stash: {
    name: '贮藏',
    short: '把没提交的改动暂时收起来',
    what: '把还没提交的改动暂时收起来，工作区恢复干净。适合手头的活没做完、又要临时切到别的分支的时候。',
    result: ['文件回到上次提交的状态', '改动保存在侧栏的「贮藏」列表里，换到任何分支都能取出来'],
    undo: '在侧栏「贮藏」里双击，改动就回来了。',
  },
  stash_drop: {
    name: '删除贮藏',
    short: '永久丢弃这份收起来的改动',
    what: '把这份贮藏永久删除。',
    result: ['里面的改动无法恢复'],
  },
  worktree: {
    name: '工作树',
    short: '给同一个仓库再开一个文件夹，同时处理另一个分支',
    what: '工作树是同一个仓库的另一个文件夹，里面打开的是另一个分支。几个文件夹互不干扰：这边改到一半，可以直接去另一个文件夹处理别的分支，不用来回切换，也不用贮藏。',
    result: ['磁盘上多出一个文件夹，里面是那个分支的文件', '所有工作树共用同一份提交历史，在任何一个里提交，其他的都能看到', '同一个分支同时只能在一个工作树里打开'],
    undo: '用完可以删除工作树。删的只是那个文件夹，分支和提交都还在。',
  },
  switch_dirty: {
    name: '带着未提交的改动切换分支',
    short: '切换前先决定没提交的改动怎么办',
    what: '你有改动还没提交。切换分支会把文件换成另一个分支的版本，所以要先决定这些改动怎么处理。',
    result: [
      '暂时收起再放回：改动原样带到新分支。放回时如果和新分支冲突，改动会留在「贮藏」里，不会丢',
      '直接带过去：改动跟着走。如果它和目标分支改了同一个文件，切换会失败，什么都不会变',
      '在新工作树里打开：改动留在这里不动，另开一个文件夹处理那个分支',
    ],
  },
  create_tag: {
    name: '标签',
    short: '给这个提交起一个固定的名字',
    what: '给某个提交起一个固定的名字，常用来标记版本号（比如 v1.2）。和分支不同，标签不会随着新提交移动。',
    result: ['提交图上多出一个标签', '只在本地，需要单独推送'],
    undo: '可以随时删除标签，不影响提交。',
  },
} satisfies Record<string, Explain>

export type ExplainKey = keyof typeof explain
