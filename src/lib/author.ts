const COUNT = 10

/** 作者名 → 固定的一个作者色。同一个名字在任何仓库、任何时候都是同一个颜色。 */
export function authorColor(name: string): string {
  let h = 0
  for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) >>> 0
  return `var(--author-${h % COUNT})`
}
