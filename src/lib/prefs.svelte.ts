// 界面偏好，存 localStorage。

const DEFAULTS = {
  /** 作者名加边框 */
  authorBorder: false,
  /** 作者名加底色 */
  authorFill: false,
}

export const prefs = $state({ ...DEFAULTS, ...JSON.parse(localStorage.getItem('prefs') ?? '{}') } as typeof DEFAULTS)

export function savePrefs() {
  localStorage.setItem('prefs', JSON.stringify(prefs))
}
