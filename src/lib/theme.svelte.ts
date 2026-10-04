// 深色 / 浅色主题。没手动选过时跟随系统。

type Theme = 'dark' | 'light'

const system = matchMedia('(prefers-color-scheme: light)')
const stored = localStorage.getItem('theme') as Theme | null

export const theme = $state({ current: stored ?? (system.matches ? 'light' : 'dark') })

const apply = () => (document.documentElement.dataset.theme = theme.current)
apply()

system.addEventListener('change', () => {
  if (localStorage.getItem('theme')) return
  theme.current = system.matches ? 'light' : 'dark'
  apply()
})

export function toggleTheme() {
  theme.current = theme.current === 'dark' ? 'light' : 'dark'
  localStorage.setItem('theme', theme.current)
  apply()
}
