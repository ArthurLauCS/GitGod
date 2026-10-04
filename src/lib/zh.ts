// 全部界面文案集中在此
export const t = {
  appName: 'GitGod',
  openRepo: '打开仓库',
  openRepoHint: '选择一个 Git 仓库的文件夹开始',
  recent: '最近打开',
  loadingHistory: '正在加载完整历史…',
  commits: (n: number) => `${n.toLocaleString('zh-CN')} 个提交`,
  filter: '筛选分支、标签…',
  branches: '本地分支',
  remotes: '远程分支',
  tags: '标签',
  stashes: '贮藏',
  worktrees: '工作树',
  detached: '游离 HEAD',
  colSubject: '说明',
  colAuthor: '作者',
  colDate: '日期',
  colCommit: '提交',
  noSelection: '选择一个提交查看详情',
  author: '作者',
  committer: '提交者',
  parents: '父提交',
  files: (n: number) => `改动文件（${n}）`,
  noCommits: '这个仓库还没有提交',
  status: { A: '新增', M: '修改', D: '删除', R: '重命名', C: '复制', T: '类型变更' } as Record<string, string>,
}

export function fmtTime(sec: number): string {
  const d = new Date(sec * 1000)
  const p = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`
}
