import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { test } from 'node:test'
import { compileModule } from 'svelte/compiler'
import ts from 'typescript'
import { authorKey } from '../src/lib/author.ts'

test('author styles stay independent and survive reloads without inheriting global flags', async () => {
  const storage = new Map([['prefs', JSON.stringify({ authorBorder: true, authorFill: true })]])
  Object.defineProperty(globalThis, 'localStorage', { configurable: true, value: {
    getItem: (key) => storage.get(key) ?? null,
    setItem: (key, value) => storage.set(key, value),
  } })
  const source = readFileSync(new URL('../src/lib/prefs.svelte.ts', import.meta.url), 'utf8')
  const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext } }).outputText
  const compiled = compileModule(js, { filename: 'prefs.svelte.js', generate: 'client' }).js.code
    .replace(/(['"])svelte\/internal\/client\1/g, JSON.stringify(import.meta.resolve('svelte/internal/client')))
  let generation = 0
  const load = () => import(`data:text/javascript;base64,${Buffer.from(compiled).toString('base64')}#${generation++}`)
  const alice = authorKey({ author: '同名作者', author_email: 'alice@example.com' })
  const bob = authorKey({ author: '同名作者', author_email: 'bob@example.com' })
  assert.notEqual(alice, bob)
  assert.equal(alice, authorKey({ author: '改过显示名', author_email: 'alice@example.com' }))
  assert.notEqual(authorKey({ author: 'Alice', author_email: '' }), authorKey({ author: 'Bob', author_email: '' }))
  assert.notEqual(authorKey({ author: 'alice@example.com', author_email: '' }), alice)

  let { prefs, setAuthorStyle } = await load()
  assert.deepEqual(prefs.authorStyles, {})
  setAuthorStyle(alice, { border: true, fill: false })
  assert.equal(prefs.authorStyles[bob], undefined)
  setAuthorStyle(bob, { border: false, fill: true })
  setAuthorStyle(alice, { border: true, fill: true })
  assert.deepEqual(prefs.authorStyles[bob], { border: false, fill: true })
  ;({ prefs, setAuthorStyle } = await load())
  assert.deepEqual(prefs.authorStyles[alice], { border: true, fill: true })
  assert.deepEqual(prefs.authorStyles[bob], { border: false, fill: true })
  setAuthorStyle(alice, { border: false, fill: false })
  assert.equal(prefs.authorStyles[alice], undefined)
  assert.deepEqual((await load()).prefs.authorStyles, { [bob]: { border: false, fill: true } })

  storage.set('prefs', '{broken')
  assert.deepEqual((await load()).prefs.authorStyles, {})
  delete globalThis.localStorage
})
