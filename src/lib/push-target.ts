/** 把上游 `refs/remotes/<远程>/<分支>` 拆开。远程名可以带斜杠，所以用已知的远程列表来拆。 */
export function splitUpstream(upstream: string | null | undefined, remotes: string[]): { remote: string; branch: string } | null {
  const remote = remotes.find((r) => upstream?.startsWith(`refs/remotes/${r}/`))
  return remote ? { remote, branch: upstream!.slice(`refs/remotes/${remote}/`.length) } : null
}
