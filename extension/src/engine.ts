import { spawn } from 'node:child_process'
import { createInterface } from 'node:readline'

export interface Engine {
  /** 命令名和参数与 src/lib/api.ts 的 invoke 一致 */
  call<T>(cmd: string, args?: object): Promise<T>
  dispose(): void
}

/** 启动 sidecar（crates/sidecar）：每行一个 JSON 请求，每行一个 JSON 回复。 */
export function startEngine(exe: string): Engine {
  const proc = spawn(exe, [], { windowsHide: true, stdio: ['pipe', 'pipe', 'inherit'] })
  const pending = new Map<number, [(value: any) => void, (error: unknown) => void]>()
  let next = 0
  let dead: string | null = null
  const fail = (reason: string) => {
    dead = reason
    for (const [, reject] of pending.values()) reject(reason)
    pending.clear()
  }
  proc.on('error', (e) => fail(String(e)))
  proc.on('exit', () => fail('PR_ENGINE_EXITED'))
  // 进程提前退出时写入会报 EPIPE，由 exit 统一处理
  proc.stdin.on('error', () => {})
  createInterface(proc.stdout).on('line', (line) => {
    const { id, ok, value } = JSON.parse(line)
    const waiter = pending.get(id)
    if (!waiter) return
    pending.delete(id)
    waiter[ok ? 0 : 1](value)
  })
  return {
    call: (cmd, args = {}) =>
      new Promise((resolve, reject) => {
        if (dead) return reject(dead)
        pending.set(++next, [resolve, reject])
        proc.stdin.write(JSON.stringify({ id: next, cmd, args }) + '\n')
      }),
    dispose: () => void proc.kill(),
  }
}
