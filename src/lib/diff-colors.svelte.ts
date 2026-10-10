import { readDiffColors, type DiffColors } from './diff-colors'

export const diffColors = $state<{ current: DiffColors | null }>({ current: readDiffColors(localStorage.getItem('diffColors')) })

function apply() {
  for (const key of ['added', 'deleted'] as const) {
    if (diffColors.current) document.documentElement.style.setProperty(`--diff-${key}`, diffColors.current[key])
    else document.documentElement.style.removeProperty(`--diff-${key}`)
  }
}
apply()
globalThis.addEventListener?.('storage', (event) => {
  if (event.key !== 'diffColors') return
  diffColors.current = readDiffColors(event.newValue)
  apply()
})

export function setDiffColors(colors: DiffColors | null) {
  localStorage.setItem('diffColors', JSON.stringify(colors))
  diffColors.current = colors
  apply()
}
