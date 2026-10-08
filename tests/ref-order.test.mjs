import assert from 'node:assert/strict'
import { test } from 'node:test'
import { orderRefs, readRecentRefs, rememberRef, writeRecentRefs } from '../src/lib/ref-order.ts'

test('reference recency is repository-scoped and preserves unseen order', () => {
  const refs = ['refs/heads/a', 'refs/heads/b', 'refs/heads/c'].map((name) => ({ name }))
  assert.deepEqual(orderRefs(refs, ['refs/heads/c'], 'refs/heads/b').map((r) => r.name), ['refs/heads/b', 'refs/heads/c', 'refs/heads/a'])
  assert.deepEqual(rememberRef(['b', 'a'], 'a'), ['a', 'b'])
  const value = writeRecentRefs(null, 'one', ['a'])
  assert.deepEqual(readRecentRefs(value, 'one'), ['a'])
  assert.deepEqual(readRecentRefs(value, 'two'), [])
  assert.deepEqual(readRecentRefs('not json', 'one'), [])
})
