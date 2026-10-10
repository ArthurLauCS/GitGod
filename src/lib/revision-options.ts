import type { Refs } from './api'

export function revisionOptions(refs: Refs, query = '') {
  const upstream = refs.refs.find((ref) => ref.name === refs.head)?.upstream
  const groups = ['current', 'common', 'branches', 'remotes', 'tags', 'other'] as const
  const needle = query.trim().toLowerCase()
  const items = refs.refs.filter((ref) => ref.name.toLowerCase().includes(needle)).map((ref) => {
    const local = ref.name.startsWith('refs/heads/')
    const remote = ref.name.startsWith('refs/remotes/')
    const label = ref.name.replace(/^refs\/(heads|remotes|tags)\//, '')
    const branch = remote ? label.slice(label.indexOf('/') + 1) : label
    const group = ref.name === refs.head || ref.name === upstream ? 'current'
      : (local || remote) && ['main', 'master', 'season', 'dev', 'develop'].includes(branch) ? 'common'
      : local ? 'branches' : remote ? 'remotes' : ref.name.startsWith('refs/tags/') ? 'tags' : 'other'
    return { name: ref.name, label, group, local }
  })
  return groups.map((key) => ({ key, items: items.filter((item) => item.group === key).sort((a, b) =>
    Number(b.name === refs.head) - Number(a.name === refs.head)
    || Number(b.local) - Number(a.local) || a.label.localeCompare(b.label)) })).filter((group) => group.items.length)
}
