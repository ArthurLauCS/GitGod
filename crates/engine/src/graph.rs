use crate::log::{Commit, Oid};

pub struct Layout {
    /// 每个提交所在的泳道
    pub lanes: Vec<u32>,
    /// 每条「提交→父提交」连线走的泳道，按提交顺序、父顺序平铺
    pub edge_lanes: Vec<u32>,
}

/// 按 `commits` 的顺序（子在父之前）分配泳道。
pub fn layout(commits: &[Commit]) -> Layout {
    // active[i] = 泳道 i 正在等待的提交
    // ponytail: 线性扫描泳道，O(提交数 × 图宽)；基准显示成为瓶颈时换成 Oid→泳道 的哈希表
    let mut active: Vec<Option<Oid>> = Vec::new();
    let mut lanes = Vec::with_capacity(commits.len());
    let mut edge_lanes = Vec::with_capacity(commits.len());
    for c in commits {
        let mut lane = None;
        for (i, slot) in active.iter_mut().enumerate() {
            if *slot == Some(c.id) {
                lane.get_or_insert(i);
                *slot = None;
            }
        }
        let lane = lane.unwrap_or_else(|| free(&mut active));
        lanes.push(lane as u32);
        for (n, p) in c.parents.iter().enumerate() {
            let l = if n == 0 {
                lane
            } else {
                active.iter().position(|s| *s == Some(*p)).unwrap_or_else(|| free(&mut active))
            };
            active[l] = Some(*p);
            edge_lanes.push(l as u32);
        }
    }
    Layout { lanes, edge_lanes }
}

fn free(active: &mut Vec<Option<Oid>>) -> usize {
    active.iter().position(Option::is_none).unwrap_or_else(|| {
        active.push(None);
        active.len() - 1
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(id: u8, parents: &[u8]) -> Commit {
        Commit { id: [id; 20], parents: parents.iter().map(|&p| [p; 20]).collect(), time: 0, author: String::new(), subject: String::new() }
    }

    #[test]
    fn merge_opens_and_closes_a_lane() {
        // M 合并 A、B，二者都基于 R
        let l = layout(&[c(4, &[2, 3]), c(3, &[1]), c(2, &[1]), c(1, &[])]);
        assert_eq!(l.lanes, [0, 1, 0, 0]);
        assert_eq!(l.edge_lanes, [0, 1, 1, 0]);
        // R 之后两条泳道都已释放，新的根提交回到泳道 0
        let l = layout(&[c(4, &[2, 3]), c(3, &[1]), c(2, &[1]), c(1, &[]), c(9, &[])]);
        assert_eq!(l.lanes[4], 0);
    }
}
