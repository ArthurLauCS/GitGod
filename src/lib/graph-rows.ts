/** 只索引已展开的提交，不为整份历史或每个文件分配显示行。 */
export interface ExpansionRange { row: number; start: number; size: number; offset: number }

export function expansionRanges(sizes: [number, number][]): ExpansionRange[] {
  let offset = 0
  return sizes.sort(([a], [b]) => a - b).map(([row, size]) => {
    const range = { row, start: row + offset + 1, size, offset }
    offset += size
    return range
  })
}

function preceding(ranges: ExpansionRange[], value: number, key: 'start' | 'row') {
  let low = 0, high = ranges.length
  while (low < high) {
    const mid = (low + high) >>> 1
    if (ranges[mid][key] <= value) low = mid + 1
    else high = mid
  }
  return ranges[low - 1]
}

/** 显示行对应的提交和子文件序号；child=-1 表示提交本身。 */
export function graphPosition(index: number, ranges: ExpansionRange[]) {
  const range = preceding(ranges, index, 'start')
  return range && index < range.start + range.size
    ? { row: range.row, child: index - range.start }
    : { row: index - (range ? range.offset + range.size : 0), child: -1 }
}

export function graphIndex(row: number, ranges: ExpansionRange[]) {
  const range = preceding(ranges, row - 1, 'row')
  return row + (range ? range.offset + range.size : 0)
}
