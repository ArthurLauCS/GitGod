// 各栏位的尺寸，拖动分隔条时改这里；存 localStorage，所有页签共用一份。

export const DEFAULTS = {
  /** 侧栏宽度 */
  sidebar: 264,
  /** 提交详情面板高度 */
  detail: 320,
  /** 本地更改页左侧文件列表宽度 */
  changes: 400,
  /** 提交详情里文件列表宽度 */
  files: 340,
  /** 提交图的作者列、日期列宽度 */
  author: 150,
  date: 136,
}
export type LayoutKey = keyof typeof DEFAULTS

export const layout = $state({ ...DEFAULTS, ...JSON.parse(localStorage.getItem('layout') ?? '{}') } as typeof DEFAULTS)

export function saveLayout() {
  localStorage.setItem('layout', JSON.stringify(layout))
}
