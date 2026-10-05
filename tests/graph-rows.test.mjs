import assert from 'node:assert/strict'
import { test } from 'node:test'
import { expansionRanges, graphIndex, graphPosition } from '../src/lib/graph-rows.ts'

test('expanded commit files map both ways without shifting neighboring commits', () => {
  const ranges = expansionRanges([[3, 1], [0, 2], [2, 3]])
  const expected = [[0, -1], [0, 0], [0, 1], [1, -1], [2, -1], [2, 0], [2, 1], [2, 2], [3, -1], [3, 0], [4, -1]]
  expected.forEach(([row, child], index) => {
    assert.deepEqual(graphPosition(index, ranges), { row, child })
    if (child === -1) assert.equal(graphIndex(row, ranges), index)
  })
  assert.deepEqual(graphPosition(4, expansionRanges([[0, 2]])), { row: 2, child: -1 })
  assert.deepEqual(graphPosition(4, []), { row: 4, child: -1 })
})

test('large histories and huge commits keep the index proportional to expanded commits', () => {
  const ranges = expansionRanges([[0, 100_000], [1_000_000, 2]])
  assert.equal(ranges.length, 2)
  assert.deepEqual(graphPosition(50_000, ranges), { row: 0, child: 49_999 })
  assert.deepEqual(graphPosition(1_100_002, ranges), { row: 1_000_000, child: 1 })
  assert.equal(graphIndex(1_500_000, ranges), 1_600_002)
})
