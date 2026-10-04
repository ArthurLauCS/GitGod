import assert from 'node:assert/strict'
import { test } from 'node:test'
import { graphKey } from '../src/lib/graph-key.ts'

test('graph refresh follows reachable tips rather than names, order or checked-out branch', () => {
  const refs = [{ name: 'main', id: 'a' }, { name: 'feature', id: 'b' }]
  const initial = graphKey({ refs, head_id: 'a' })
  assert.equal(graphKey({ refs, head_id: 'b' }), initial)
  assert.equal(graphKey({ refs: [{ name: 'renamed', id: 'b' }, refs[0], { name: 'alias', id: 'a' }], head_id: 'a' }), initial)
  assert.notEqual(graphKey({ refs, head_id: 'new-detached-commit' }), initial)
  assert.notEqual(graphKey({ refs: [refs[0]], head_id: 'a' }), initial)
  assert.notEqual(graphKey({ refs: [refs[0], { name: 'feature', id: 'new-commit' }], head_id: 'a' }), initial)
})
