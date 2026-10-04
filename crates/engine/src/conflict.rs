use crate::{decode, err, Repo, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Default)]
pub struct Conflict {
    /// 二进制文件没法逐块选，只能整个文件取一边
    pub binary: bool,
    pub blocks: Vec<Block>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Block {
    /// 两边一致的部分；长的只保留头尾几行供定位
    Text { text: String },
    Conflict { ours: String, theirs: String, ours_label: String, theirs_label: String },
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Ours,
    Theirs,
    /// 先我方后对方，两段都保留
    Both,
}

/// 按原始字节切开：写回文件时原样拼接，保持文件原来的编码和换行。
enum Raw<'a> {
    Text(Vec<&'a [u8]>),
    Conflict { ours: Vec<&'a [u8]>, theirs: Vec<&'a [u8]>, ours_label: &'a [u8], theirs_label: &'a [u8] },
}

fn split(bytes: &[u8]) -> Result<Vec<Raw<'_>>> {
    #[derive(PartialEq)]
    enum In {
        Text,
        Ours,
        Base,
        Theirs,
    }
    let mut state = In::Text;
    let mut blocks = vec![Raw::Text(Vec::new())];
    let (mut ours, mut theirs, mut ours_label) = (Vec::new(), Vec::new(), &b""[..]);
    for line in bytes.split_inclusive(|&b| b == b'\n') {
        let body = line.strip_suffix(b"\n").unwrap_or(line);
        let body = body.strip_suffix(b"\r").unwrap_or(body);
        let label = |marker: &[u8]| body.strip_prefix(marker).map(|l| l.strip_prefix(b" ").unwrap_or(l));
        match state {
            In::Text => match label(b"<<<<<<<") {
                Some(l) => (state, ours_label) = (In::Ours, l),
                None => match blocks.last_mut() {
                    Some(Raw::Text(lines)) => lines.push(line),
                    _ => blocks.push(Raw::Text(vec![line])),
                },
            },
            // diff3 风格多一段共同祖先，界面不展示
            In::Ours if label(b"|||||||").is_some() => state = In::Base,
            In::Ours | In::Base if body == b"=======" => state = In::Theirs,
            In::Ours => ours.push(line),
            In::Base => {}
            In::Theirs => match label(b">>>>>>>") {
                Some(theirs_label) => {
                    blocks.push(Raw::Conflict { ours: std::mem::take(&mut ours), theirs: std::mem::take(&mut theirs), ours_label, theirs_label });
                    state = In::Text;
                }
                None => theirs.push(line),
            },
        }
    }
    if state != In::Text {
        return Err("PR_CONFLICT_MARKERS".into());
    }
    Ok(blocks)
}

pub fn read(repo: &Repo, path: &str) -> Result<Conflict> {
    let bytes = std::fs::read(repo.path.join(path)).map_err(err)?;
    if bytes.contains(&0) {
        return Ok(Conflict { binary: true, ..Default::default() });
    }
    let text = |lines: &[&[u8]]| decode(&lines.concat());
    let blocks = split(&bytes)?
        .iter()
        .filter_map(|b| match b {
            Raw::Text(lines) if lines.is_empty() => None,
            Raw::Text(lines) if lines.len() > 8 => {
                Some(Block::Text { text: format!("{}⋯\n{}", text(&lines[..3]), text(&lines[lines.len() - 3..])) })
            }
            Raw::Text(lines) => Some(Block::Text { text: text(lines) }),
            Raw::Conflict { ours, theirs, ours_label, theirs_label } => Some(Block::Conflict {
                ours: text(ours),
                theirs: text(theirs),
                ours_label: decode(ours_label),
                theirs_label: decode(theirs_label),
            }),
        })
        .collect();
    Ok(Conflict { binary: false, blocks })
}

/// 按 `choices`（每个冲突块一个）写回文件并暂存，即标记为已解决。
pub fn resolve(repo: &Repo, path: &str, choices: &[Side]) -> Result<()> {
    let file = repo.path.join(path);
    let bytes = std::fs::read(&file).map_err(err)?;
    let blocks = split(&bytes)?;
    if blocks.iter().filter(|b| matches!(b, Raw::Conflict { .. })).count() != choices.len() {
        return Err("PR_FILE_CHANGED".into());
    }
    let mut choices = choices.iter();
    let mut out = Vec::with_capacity(bytes.len());
    for b in &blocks {
        match b {
            Raw::Text(lines) => out.extend(lines.concat()),
            Raw::Conflict { ours, theirs, .. } => match choices.next().unwrap() {
                Side::Ours => out.extend(ours.concat()),
                Side::Theirs => out.extend(theirs.concat()),
                Side::Both => out.extend([ours.concat(), theirs.concat()].concat()),
            },
        }
    }
    std::fs::write(&file, out).map_err(err)?;
    repo.git(&["add", "--", path]).map(drop)
}

/// 整个文件取一边并暂存。二进制文件（.rbxl、贴图）只能这样解决。
pub fn take(repo: &Repo, path: &str, theirs: bool) -> Result<()> {
    repo.git(&["checkout", if theirs { "--theirs" } else { "--ours" }, "--", path])?;
    repo.git(&["add", "--", path]).map(drop)
}
