export interface DiffColors { added: string; deleted: string }

export const DIFF_PRESETS = {
  blueOrange: { added: '#0072b2', deleted: '#e69f00' },
  purpleGold: { added: '#aa88dd', deleted: '#d9a400' },
  cyanRose: { added: '#00a6b2', deleted: '#cc79a7' },
} satisfies Record<string, DiffColors>

export const isDiffColor = (value: unknown): value is string => typeof value === 'string' && /^#[\da-f]{6}$/i.test(value)

export function validDiffColors(value: unknown): value is DiffColors {
  return !!value && typeof value === 'object' && 'added' in value && 'deleted' in value
    && isDiffColor(value.added) && isDiffColor(value.deleted)
}

export function readDiffColors(value: string | null): DiffColors | null {
  try { const parsed = JSON.parse(value ?? 'null'); return validDiffColors(parsed) ? parsed : null }
  catch { return null }
}
