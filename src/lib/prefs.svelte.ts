// 界面偏好，存 localStorage。

export interface AuthorStyle {
  border: boolean
  fill: boolean
}

let authorStyles: Record<string, AuthorStyle> = {}
try {
  // 旧版全局开关无法确定要标记谁，不迁移到任何作者。
  authorStyles = JSON.parse(localStorage.getItem('prefs') ?? '{}')?.authorStyles ?? {}
} catch {
  // 损坏的本地偏好不应阻止打开仓库。
}

export const prefs = $state({ authorStyles })

// 另一个窗口（或 VS Code 扩展主机）改了作者样式时跟着更新
globalThis.addEventListener?.('storage', (e) => {
  if (e.key === 'prefs' && e.newValue) prefs.authorStyles = JSON.parse(e.newValue).authorStyles ?? {}
})

export function setAuthorStyle(key: string, style: AuthorStyle) {
  if (style.border || style.fill) prefs.authorStyles[key] = style
  else delete prefs.authorStyles[key]
  localStorage.setItem('prefs', JSON.stringify(prefs))
}
