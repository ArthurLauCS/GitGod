use crate::{err, Repo, Result};
use gix::ObjectId;
use serde::Serialize;

#[derive(Serialize)]
pub struct Ref {
    /// 完整引用名，如 refs/heads/main
    pub name: String,
    /// 剥到提交后的 id
    pub id: String,
}

#[derive(Serialize)]
pub struct Refs {
    /// HEAD 指向的完整引用名；游离 HEAD 时为 None
    pub head: Option<String>,
    pub head_id: Option<String>,
    pub refs: Vec<Ref>,
}

#[derive(Serialize)]
pub struct Stash {
    pub name: String,
    pub id: String,
    pub subject: String,
}

#[derive(Serialize, Default)]
pub struct Worktree {
    pub path: String,
    pub head: String,
    /// 完整引用名；游离 HEAD 或裸仓库时为 None
    pub branch: Option<String>,
}

fn commits(gix: &gix::Repository) -> Result<Vec<(String, ObjectId)>> {
    let mut out = Vec::new();
    for r in gix.references().map_err(err)?.all().map_err(err)? {
        let Ok(r) = r else { continue };
        let name = r.name().as_bstr().to_string();
        // 标签可以指向 tree/blob（内核的 v2.6.11），这些不进提交图
        let Ok(id) = r.into_fully_peeled_id() else { continue };
        if id.header().is_ok_and(|h| h.kind() == gix::object::Kind::Commit) {
            out.push((name, id.detach()));
        }
    }
    Ok(out)
}

/// 提交图的起点：所有引用加 HEAD（游离 HEAD 不在引用里）。
pub(crate) fn tips(gix: &gix::Repository) -> Result<Vec<ObjectId>> {
    let mut tips: Vec<_> = commits(gix)?.into_iter().map(|(_, id)| id).collect();
    tips.extend(gix.head_id().ok().map(|id| id.detach()));
    Ok(tips)
}

pub fn list(repo: &Repo) -> Result<Refs> {
    let gix = repo.gix();
    Ok(Refs {
        head: gix.head_name().map_err(err)?.map(|n| n.as_bstr().to_string()),
        head_id: gix.head_id().ok().map(|id| id.to_string()),
        refs: commits(&gix)?.into_iter().map(|(name, id)| Ref { name, id: id.to_string() }).collect(),
    })
}

pub fn stashes(repo: &Repo) -> Result<Vec<Stash>> {
    let out = repo.git(&["stash", "list", "--format=%gd%x00%H%x00%gs"])?;
    Ok(String::from_utf8_lossy(&out)
        .lines()
        .filter_map(|l| {
            let mut f = l.split('\0');
            Some(Stash { name: f.next()?.into(), id: f.next()?.into(), subject: f.next()?.into() })
        })
        .collect())
}

pub fn worktrees(repo: &Repo) -> Result<Vec<Worktree>> {
    let out = repo.git(&["worktree", "list", "--porcelain", "-z"])?;
    let mut list = Vec::new();
    let mut cur: Option<Worktree> = None;
    // 每条记录以空字段结束
    for field in String::from_utf8_lossy(&out).split('\0') {
        match field.split_once(' ').unwrap_or((field, "")) {
            ("worktree", path) => cur = Some(Worktree { path: path.into(), ..Default::default() }),
            ("HEAD", id) => cur.as_mut().unwrap().head = id.into(),
            ("branch", name) => cur.as_mut().unwrap().branch = Some(name.into()),
            ("", _) => list.extend(cur.take()),
            _ => {}
        }
    }
    Ok(list)
}
