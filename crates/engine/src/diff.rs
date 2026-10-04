use crate::{decode, err, Repo, Result};
use gix::ObjectId;
use serde::Serialize;

// ponytail: 超过这个大小的补丁不解析不显示；需要时改成按区块分页
const MAX_PATCH: usize = 4 << 20;

#[derive(Serialize, Default)]
pub struct Diff {
    pub binary: bool,
    pub too_large: bool,
    pub hunks: Vec<Hunk>,
}

#[derive(Serialize)]
pub struct Hunk {
    /// `@@ -a,b +c,d @@ ...` 整行，按行暂存时用来确认文件没变过
    pub header: String,
    pub lines: Vec<Line>,
}

#[derive(Serialize)]
pub struct Line {
    /// ' ' 上下文，'+' 新增，'-' 删除，'\\' 文件末尾无换行的标记
    pub kind: char,
    pub text: String,
    pub old_no: Option<u32>,
    pub new_no: Option<u32>,
}

/// 补丁按原始字节切开：按行暂存要原样拼回去，不能经过解码。
struct Raw<'a> {
    file_header: &'a [u8],
    binary: bool,
    hunks: Vec<(&'a [u8], Vec<&'a [u8]>)>,
}

fn split(patch: &[u8]) -> Raw<'_> {
    let mut raw = Raw { file_header: patch, binary: false, hunks: Vec::new() };
    let mut pos = 0;
    for line in patch.split_inclusive(|&b| b == b'\n') {
        let body = line.strip_suffix(b"\n").unwrap_or(line);
        if body.starts_with(b"@@") {
            if raw.hunks.is_empty() {
                raw.file_header = &patch[..pos];
            }
            raw.hunks.push((body, Vec::new()));
        } else if let Some(h) = raw.hunks.last_mut() {
            h.1.push(body);
        } else if body.starts_with(b"Binary files") || body.starts_with(b"GIT binary patch") {
            raw.binary = true;
        }
        pos += line.len();
    }
    raw
}

fn parse(patch: &[u8]) -> Diff {
    if patch.len() > MAX_PATCH {
        return Diff { too_large: true, ..Default::default() };
    }
    let raw = split(patch);
    let hunks = raw
        .hunks
        .iter()
        .map(|(header, lines)| {
            let header = decode(header);
            // @@ -旧起始[,行数] +新起始[,行数] @@
            let start = |sign: char| -> u32 {
                let s = header.split(' ').find(|s| s.starts_with(sign)).unwrap_or_default();
                s.get(1..).and_then(|s| s.split(',').next()).and_then(|n| n.parse().ok()).unwrap_or(0)
            };
            let (mut old_no, mut new_no) = (start('-'), start('+'));
            let take = |n: &mut u32| {
                *n += 1;
                Some(*n - 1)
            };
            let lines = lines
                .iter()
                .map(|l| {
                    let kind = l.first().map_or(' ', |&b| b as char);
                    let text = decode(l.get(1..).unwrap_or_default()).trim_end_matches('\r').to_owned();
                    match kind {
                        '+' => Line { kind, text, old_no: None, new_no: take(&mut new_no) },
                        '-' => Line { kind, text, old_no: take(&mut old_no), new_no: None },
                        '\\' => Line { kind, text, old_no: None, new_no: None },
                        _ => Line { kind: ' ', text, old_no: take(&mut old_no), new_no: take(&mut new_no) },
                    }
                })
                .collect();
            Hunk { header, lines }
        })
        .collect();
    Diff { binary: raw.binary, too_large: false, hunks }
}

fn patch(repo: &Repo, path: &str, staged: bool) -> Result<Vec<u8>> {
    let mut args = vec!["diff", "--no-color", "--no-ext-diff"];
    if staged {
        args.push("--cached");
    }
    args.extend(["--", path]);
    repo.git(&args)
}

/// 工作区（`staged` 为 false）或暂存区的单文件改动。未跟踪文件整个显示为新增。
pub fn worktree(repo: &Repo, path: &str, staged: bool, untracked: bool) -> Result<Diff> {
    if !untracked {
        return Ok(parse(&patch(repo, path, staged)?));
    }
    let bytes = std::fs::read(repo.path.join(path)).map_err(err)?;
    if bytes.len() > MAX_PATCH {
        return Ok(Diff { too_large: true, ..Default::default() });
    }
    if bytes.contains(&0) {
        return Ok(Diff { binary: true, ..Default::default() });
    }
    let text = decode(&bytes);
    let lines: Vec<_> = text
        .lines()
        .enumerate()
        .map(|(i, l)| Line { kind: '+', text: l.to_owned(), old_no: None, new_no: Some(i as u32 + 1) })
        .collect();
    Ok(Diff { hunks: vec![Hunk { header: format!("@@ -0,0 +1,{} @@", lines.len()), lines }], ..Default::default() })
}

/// 某个提交里单个文件的改动；合并提交对比第一个父提交。
pub fn commit(repo: &Repo, id: &str, path: &str) -> Result<Diff> {
    let oid = ObjectId::from_hex(id.as_bytes()).map_err(err)?.to_string();
    let args = ["show", "--format=", "--no-color", "--no-ext-diff", "--diff-merges=first-parent", &oid, "--", path];
    Ok(parse(&repo.git(&args)?))
}

/// 暂存（`staged` 为 false）或取消暂存第 `hunk` 个区块里选中的行。`lines` 是区块内的行下标。
pub fn apply_lines(repo: &Repo, path: &str, staged: bool, hunk: usize, header: &str, lines: &[usize]) -> Result<()> {
    let out = partial(repo, path, staged, staged, hunk, header, lines)?;
    let mut args = vec!["apply", "--cached", "--recount", "--whitespace=nowarn"];
    if staged {
        args.push("--reverse");
    }
    repo.git_in(&args, &out).map(drop)
}

/// 丢弃工作区里第 `hunk` 个区块中选中的行。丢弃前把当前全部改动存进贮藏列表，后悔了可以取回。
pub fn discard_lines(repo: &Repo, path: &str, hunk: usize, header: &str, lines: &[usize]) -> Result<()> {
    let out = partial(repo, path, false, true, hunk, header, lines)?;
    // stash create 只生成备份提交，不动工作区；store 把它挂进贮藏列表
    let backup = String::from_utf8_lossy(&repo.git(&["stash", "create"])?).trim().to_owned();
    if !backup.is_empty() {
        repo.git(&["stash", "store", "-m", &format!("丢弃前的备份：{path}"), &backup])?;
    }
    repo.git_in(&["apply", "--reverse", "--recount", "--whitespace=nowarn"], &out).map(drop)
}

/// 从 `path` 的补丁里取出第 `hunk` 个区块，只保留选中的行，拼成一个可以单独应用的补丁。
/// `staged` 决定用暂存区还是工作区的补丁，`reverse` 说明这个补丁之后会不会被反向应用。
fn partial(repo: &Repo, path: &str, staged: bool, reverse: bool, hunk: usize, header: &str, lines: &[usize]) -> Result<Vec<u8>> {
    let patch = patch(repo, path, staged)?;
    let raw = split(&patch);
    let (raw_header, raw_lines) =
        raw.hunks.get(hunk).filter(|h| decode(h.0) == header).ok_or("文件已经变化，请刷新后重试")?;

    // 正向应用时目标是补丁的旧侧：没选中的删除行在目标里还在，当上下文；没选中的新增行不存在，丢掉。
    // 反向应用时目标是新侧，两者对调。
    let (keep, discard) = if reverse { (b'+', b'-') } else { (b'-', b'+') };
    let mut out = [raw.file_header, raw_header, b"\n"].concat();
    let mut dropped = false;
    for (i, l) in raw_lines.iter().enumerate() {
        let kind = l.first().copied().unwrap_or(b' ');
        if kind == b'\\' {
            // 「文件末尾无换行」的标记跟随它前一行的去留
            if !dropped {
                out.extend_from_slice(l);
                out.push(b'\n');
            }
            continue;
        }
        dropped = kind == discard && !lines.contains(&i);
        if dropped {
            continue;
        }
        if kind == keep && !lines.contains(&i) {
            out.push(b' ');
            out.extend_from_slice(&l[1..]);
        } else {
            out.extend_from_slice(l);
        }
        out.push(b'\n');
    }
    Ok(out)
}
