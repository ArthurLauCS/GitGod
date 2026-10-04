import { messages as en, explanations as enHelp } from './locales/en'
import { messages as zh, explanations as zhHelp } from './locales/zh'
import { diagrams, type Explain, type ExplainKey } from './explain'

export type Locale = 'en' | 'zh-CN'
export const locale = $state({ current: (localStorage.getItem('locale') === 'zh-CN' ? 'zh-CN' : 'en') as Locale })
const catalogues = { en, 'zh-CN': zh }
const help = { en: enHelp, 'zh-CN': zhHelp }
const makeHelp = (language: Locale) => Object.fromEntries(
  Object.entries(help[language]).map(([key, copy]) => [key, { ...copy, ...diagrams[key as ExplainKey] }]),
) as Record<ExplainKey, Explain>

export const t = $state({ ...catalogues[locale.current] })
export const explain = $state(makeHelp(locale.current))
document.documentElement.lang = locale.current

export function setLocale(language: Locale) {
  locale.current = language
  Object.assign(t, catalogues[language])
  Object.assign(explain, makeHelp(language))
  localStorage.setItem('locale', language)
  document.documentElement.lang = language
}

/** Only translate application error codes; Git output and repository content stay verbatim. */
export function errorText(error: unknown): string {
  const text = String(error)
  const [code, ...detail] = text.split(': ')
  return Object.hasOwn(t.errors, code) ? t.errors[code] + (detail.length ? ': ' + detail.join(': ') : '') : text
}

export function fmtTime(sec: number): string {
  const d = new Date(sec * 1000)
  const p = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`
}
