import assert from 'node:assert/strict'
import { test } from 'node:test'
import { revisionOptions } from '../src/lib/revision-options.ts'

const ref = (name, upstream = null) => ({ name, id: 'same-commit', upstream })
const refs = { head: 'refs/heads/feature', refs: [
  ref('refs/tags/season'), ref('refs/heads/z-last'), ref('refs/remotes/origin/season'),
  ref('refs/heads/season'), ref('refs/heads/dev'), ref('refs/heads/main'),
  ref('refs/heads/master'), ref('refs/heads/develop'), ref('refs/heads/a-first'),
  ref('refs/remotes/origin/feature'), ref('refs/heads/feature', 'refs/remotes/origin/feature'),
  ref('refs/heads/feature/dev'), ref('refs/remotes/origin/feature/dev'), ref('refs/notes/commits'),
] }

test('current branch and upstream precede common branches, then local, remote, tags and other refs', () => {
  const before = structuredClone(refs)
  const groups = revisionOptions(refs)
  assert.deepEqual(groups.map(g => g.key), ['current', 'common', 'branches', 'remotes', 'tags', 'other'])
  assert.deepEqual(groups[0].items.map(r => r.name), ['refs/heads/feature', 'refs/remotes/origin/feature'])
  assert.deepEqual(groups[1].items.map(r => r.label), ['dev', 'develop', 'main', 'master', 'season', 'origin/season'])
  assert.deepEqual(groups[2].items.map(r => r.label), ['a-first', 'feature/dev', 'z-last'])
  assert.deepEqual(refs, before)
  assert.equal(new Set(groups.flatMap(g => g.items.map(r => r.name))).size, refs.refs.length)
})

test('typing filters case-insensitively without mixing branches and same-named tags', () => {
  const groups = revisionOptions(refs, ' SEASON ')
  assert.deepEqual(groups.map(g => g.key), ['common', 'tags'])
  assert.deepEqual(groups.flatMap(g => g.items.map(r => r.name)), [
    'refs/heads/season', 'refs/remotes/origin/season', 'refs/tags/season',
  ])
  assert.deepEqual(revisionOptions(refs, 'HEAD~1'), [])
  assert.deepEqual(revisionOptions(refs, 'deadbeef'), [])
})

test('detached, empty, missing-upstream and current-common branches remain usable', () => {
  assert.equal(revisionOptions({ ...refs, head: null })[0].key, 'common')
  assert.deepEqual(revisionOptions({ head: null, refs: [] }), [])
  const groups = revisionOptions({ head: 'refs/heads/season', refs: [ref('refs/heads/season', 'refs/remotes/gone/season')] })
  assert.deepEqual(groups.map(g => g.key), ['current'])
  assert.equal(groups[0].items[0].name, 'refs/heads/season')
})
