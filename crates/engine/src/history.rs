use crate::ops::safe;
use crate::{decode, Repo, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct Commit {
    pub id: String,
    pub author: String,
    pub author_email: String,
    pub time: i64,
    pub subject: String,
    /// 文件在这个提交里的路径（重命名之前是旧名字）；搜索结果里为空
    pub path: String,
}

/// 每条记录以 NUL 开头，字段以 0x1f 分隔；`--name-only` 的路径跟在记录后面的行里
const FORMAT: &str = "--format=%x00%H%x1f%an%x1f%ae%x1f%at%x1f%s";

fn parse(out: &[u8], path: &str, patches: bool) -> Vec<Commit> {
    decode(out)
        .split('\0')
        .skip(1)
        .filter_map(|rec| {
            let (head, rest) = rec.split_once('\n').unwrap_or((rec, ""));
            let mut f = head.split('\x1f');
            Some(Commit {
                id: f.next()?.to_owned(),
                author: f.next()?.to_owned(),
                author_email: f.next()?.to_owned(),
                time: f.next()?.parse().ok()?,
                subject: f.next()?.to_owned(),
                path: if patches { rest.lines().find_map(|l| l.strip_prefix("+++ ")) } else { rest.lines().find(|l| !l.is_empty()) }.unwrap_or(path).to_owned(),
            })
        })
        .collect()
}

/// 改过 `path` 的提交，新的在前，跟踪重命名。
pub fn file(repo: &Repo, path: &str, limit: usize) -> Result<Vec<Commit>> {
    let n = limit.to_string();
    let args = ["-c", "core.quotepath=false", "log", "--follow", "-n", &n, FORMAT, "--name-only", "--", path];
    Ok(parse(&repo.git(&args)?, path, false))
}

/// 改过 `path` 第 `start`..=`end` 行（从 1 开始）的提交。
pub fn lines(repo: &Repo, path: &str, start: u32, end: u32, limit: usize) -> Result<Vec<Commit>> {
    // -L 的行号属于 HEAD；本地行号变化时不能把另一段代码的历史显示为选区历史。
    repo.git(&["diff", "--quiet", "HEAD", "--", path]).map_err(|_| "PR_LINE_HISTORY_CHANGED")?;
    let n = limit.to_string();
    let range = format!("-L{start},{end}:{path}");
    let out = repo.git_limited(&["-c", "core.quotepath=false", "log", "-n", &n, FORMAT, "--no-prefix", "--no-color", "--no-ext-diff", "--no-textconv", &range], 4 * 1024 * 1024)?;
    if out.len() > 4 * 1024 * 1024 { return Err("PR_DIFF_TOO_LARGE".into()); }
    Ok(parse(&out, path, true))
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SearchKind {
    Message,
    Author,
    /// 改动内容里增删过这段文字的提交（git log -S）
    Content,
    File,
    Id,
}

/// 在所有分支里搜索提交。
pub fn search(repo: &Repo, kind: SearchKind, query: &str, limit: usize) -> Result<Vec<Commit>> {
    let n = limit.to_string();
    let pattern;
    let mut args = vec!["log", "-n", &n, FORMAT];
    match kind {
        SearchKind::Message => {
            pattern = format!("--grep={query}");
            args.extend(["--all", "-i", "--fixed-strings", &pattern]);
        }
        SearchKind::Author => {
            pattern = format!("--author={query}");
            args.extend(["--all", "-i", "--fixed-strings", &pattern]);
        }
        SearchKind::Content => {
            pattern = format!("-S{query}");
            args.extend(["--all", &pattern]);
        }
        SearchKind::File => args.extend(["--all", "--", query]),
        SearchKind::Id => args.extend(["--no-walk", safe(query)?]),
    }
    Ok(parse(&repo.git(&args)?, "", false))
}

/// 文件在某个版本的内容；`rev` 为空表示暂存区。
pub fn show(repo: &Repo, rev: &str, path: &str) -> Result<Option<String>> {
    if rev.starts_with('-') {
        return Err(format!("PR_INVALID_NAME: {rev:?}"));
    }
    let spec = format!("{rev}:{path}");
    // 先查类型和大小，不把大对象读进进程。不存在的版本用于显示新增/删除的空白一侧。
    let info = repo.git_in(&["cat-file", "--batch-check=%(objecttype) %(objectsize)", "-z"], format!("{spec}\0").as_bytes())?;
    let info = String::from_utf8_lossy(&info);
    if info.ends_with(" missing\n") { return Ok(None); }
    let size = info.strip_prefix("blob ").and_then(|s| s.trim().parse::<usize>().ok()).ok_or("PR_NOT_TEXT_FILE")?;
    const LIMIT: usize = 4 * 1024 * 1024;
    if size > LIMIT { return Err("PR_FILE_TOO_LARGE".into()); }
    let bytes = repo.git_limited(&["show", &spec], LIMIT)?;
    if bytes.len() > LIMIT { return Err("PR_FILE_TOO_LARGE".into()); }
    if bytes.contains(&0) { return Err("PR_NOT_TEXT_FILE".into()); }
    Ok(Some(decode(&bytes)))
}
