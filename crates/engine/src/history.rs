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

fn parse(out: &[u8], path: &str) -> Vec<Commit> {
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
                path: rest.lines().find(|l| !l.is_empty()).unwrap_or(path).to_owned(),
            })
        })
        .collect()
}

/// 改过 `path` 的提交，新的在前，跟踪重命名。
pub fn file(repo: &Repo, path: &str, limit: usize) -> Result<Vec<Commit>> {
    let n = limit.to_string();
    let args = ["-c", "core.quotepath=false", "log", "--follow", "-n", &n, FORMAT, "--name-only", "--", path];
    Ok(parse(&repo.git(&args)?, path))
}

/// 改过 `path` 第 `start`..=`end` 行（从 1 开始）的提交。
pub fn lines(repo: &Repo, path: &str, start: u32, end: u32, limit: usize) -> Result<Vec<Commit>> {
    let n = limit.to_string();
    let range = format!("-L{start},{end}:{path}");
    Ok(parse(&repo.git(&["log", "-n", &n, FORMAT, "-s", &range])?, path))
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
    Ok(parse(&repo.git(&args)?, ""))
}

/// 文件在某个版本的内容；`rev` 为空表示暂存区。
pub fn show(repo: &Repo, rev: &str, path: &str) -> Result<String> {
    if rev.starts_with('-') {
        return Err(format!("PR_INVALID_NAME: {rev:?}"));
    }
    Ok(decode(&repo.git(&["show", &format!("{rev}:{path}")])?))
}
