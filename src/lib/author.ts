const COUNT = 10

/** 用邮箱区分同名作者；没有邮箱的历史提交退回到姓名。 */
export function authorKey(author: { author: string; author_email: string }): string {
  return author.author_email ? `email:${author.author_email}` : `name:${author.author}`
}

/** 作者名 → 固定的一个作者色。同一个名字在任何仓库、任何时候都是同一个颜色。 */
export function authorColor(name: string): string {
  let h = 0
  for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) >>> 0
  return `var(--author-${h % COUNT})`
}
