import assert from 'node:assert/strict'
import { test } from 'node:test'
import { setImmediate } from 'node:timers/promises'
import { loadAutoSync, startAutoSync } from '../src/lib/auto-sync.ts'

test('desktop sync preferences default on and preserve independent opt-outs', () => {
  for (const saved of [null, '{broken', 'null']) assert.deepEqual(loadAutoSync({ getItem: () => saved }), { refresh: true, fetch: true })
  assert.deepEqual(loadAutoSync({ getItem: () => '{"refresh":false,"fetch":true}' }), { refresh: false, fetch: true })
  assert.deepEqual(loadAutoSync({ getItem: () => '{"fetch":false}' }), { refresh: true, fetch: false })
})

test('open repositories refresh every minute and fetch at startup and every ten minutes', async (t) => {
  t.mock.timers.enable({ apis: ['Date', 'setInterval'], now: 1_000 })
  const settings = { refresh: true, fetch: true }, calls = []
  const sync = startAutoSync({ settings: () => settings, ready: () => true,
    fetch: async () => { calls.push('fetch') }, refresh: async () => { calls.push('refresh') }, error: assert.fail })
  await setImmediate()
  assert.deepEqual(calls, ['fetch', 'refresh'])
  for (let minute = 1; minute <= 9; minute++) { t.mock.timers.tick(60_000); await setImmediate() }
  assert.equal(calls.filter((c) => c === 'fetch').length, 1)
  assert.equal(calls.filter((c) => c === 'refresh').length, 10)
  t.mock.timers.tick(60_000); await setImmediate()
  assert.deepEqual(calls.slice(-2), ['fetch', 'refresh'])
  settings.refresh = false
  t.mock.timers.tick(60_000); await setImmediate()
  assert.equal(calls.length, 13, 'fetch-only mode does not refresh each minute')
  settings.fetch = false
  await sync.tick()
  t.mock.timers.tick(600_000); await setImmediate()
  assert.equal(calls.length, 13)
  settings.fetch = true
  await sync.tick()
  assert.deepEqual(calls.slice(-2), ['fetch', 'refresh'], 'enabling fetch starts immediately and refreshes its results')
  sync.stop()
  t.mock.timers.tick(600_000); await setImmediate()
  assert.equal(calls.length, 15, 'closed tabs stop their timers')
})

test('busy repositories defer work, slow fetches never overlap, and close cancels follow-up refresh', async (t) => {
  t.mock.timers.enable({ apis: ['Date', 'setInterval'], now: 1_000 })
  let busy = true, fetches = 0, refreshes = 0, finish
  const pending = new Promise((resolve) => { finish = resolve })
  const sync = startAutoSync({ settings: () => ({ refresh: true, fetch: true }), ready: () => !busy,
    fetch: async () => { fetches++; await pending }, refresh: async () => { refreshes++ }, error: assert.fail })
  await setImmediate()
  assert.equal(fetches, 0)
  busy = false
  t.mock.timers.tick(60_000); await setImmediate()
  assert.equal(fetches, 1)
  t.mock.timers.tick(1_800_000); await setImmediate()
  assert.deepEqual([fetches, refreshes], [1, 0])
  sync.stop()
  finish(); await setImmediate()
  assert.equal(refreshes, 0)
})

test('failed fetches still refresh local changes and wait ten minutes before retrying', async (t) => {
  t.mock.timers.enable({ apis: ['Date', 'setInterval'], now: 1_000 })
  let fetches = 0, refreshes = 0
  const errors = []
  const sync = startAutoSync({ settings: () => ({ refresh: true, fetch: true }), ready: () => true,
    fetch: async () => { fetches++; throw new Error('offline') }, refresh: async () => { refreshes++ }, error: (e) => errors.push(e.message) })
  await setImmediate()
  assert.deepEqual([fetches, refreshes, errors], [1, 1, ['offline']])
  t.mock.timers.tick(60_000); await setImmediate()
  assert.deepEqual([fetches, refreshes], [1, 2])
  t.mock.timers.tick(3_600_000); await setImmediate()
  assert.equal(fetches, 2, 'resuming after sleep does not replay every missed fetch')
  sync.stop()
})
