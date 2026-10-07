use crate::{decode, err, git_at, ops::safe, Repo, Result};
use crate::detail::{parse_name_status, FileChange};
use serde::Serialize;
use std::path::Path;

/// Resolve user input once so comparison and recovery keep referring to the same objects.
pub fn revision(repo: &Repo, rev: &str) -> Result<String> {
    Ok(decode(&repo.git(&["rev-parse", "--verify", "--end-of-options", &format!("{}^{{commit}}", safe(rev)?)])?).trim().to_owned())
}

#[derive(Serialize)]
pub struct Comparison {
    pub left: String,
    pub right: String,
    pub files: Vec<FileChange>,
}

pub fn compare(repo: &Repo, left: &str, right: &str, common_base: bool) -> Result<Comparison> {
    let mut left = revision(repo, left)?;
    let right = revision(repo, right)?;
    if common_base { left = decode(&repo.git(&["merge-base", &left, &right])?).trim().to_owned(); }
    let out = repo.git(&["diff", "--name-status", "-z", "-M", &left, &right, "--"])?;
    Ok(Comparison { left, right, files: parse_name_status(&decode(&out)) })
}

#[derive(Serialize)]
pub struct Remote { pub name: String, pub fetch: String, pub push: String }

pub fn remotes(repo: &Repo) -> Result<Vec<Remote>> {
    crate::refs::remotes(repo)?.into_iter().map(|name| {
        let fetch = decode(&repo.git(&["remote", "get-url", &name])?).trim().to_owned();
        let push = decode(&repo.git(&["remote", "get-url", "--push", &name])?).trim().to_owned();
        Ok(Remote { name, fetch, push })
    }).collect()
}

#[derive(Serialize)]
pub struct Reflog { pub id: String, pub selector: String, pub subject: String }

pub fn reflog(repo: &Repo, skip: usize, limit: usize) -> Result<Vec<Reflog>> {
    let out = repo.git(&["reflog", "show", "--all", "--format=%H%x00%gD%x00%gs", &format!("--skip={skip}"), &format!("--max-count={}", limit.min(201))])?;
    Ok(decode(&out).lines().filter_map(|line| {
        let mut fields = line.splitn(3, '\0');
        Some(Reflog { id: fields.next()?.into(), selector: fields.next()?.into(), subject: fields.next()?.into() })
    }).collect())
}

/// Never delete or replace the destination, including on a failed clone.
pub fn create(path: &str, url: Option<&str>) -> Result<String> {
    let path = Path::new(path);
    if !path.is_absolute() { return Err("PR_INVALID_PATH".into()); }
    if path.join(".git").exists() { return Err("PR_REPO_EXISTS".into()); }
    if let Some(url) = url {
        safe(url)?;
        if path.exists() && std::fs::read_dir(path).map_err(err)?.next().is_some() { return Err("PR_DEST_NOT_EMPTY".into()); }
        let parent = path.parent().ok_or("PR_INVALID_PATH")?;
        git_at(parent, &["-c", "credential.interactive=false", "clone", "--", url, &path.to_string_lossy()], &[])?;
    } else {
        git_at(path, &["init", "-b", "main"], &[])?;
    }
    Ok(Repo::open(path)?.path.to_string_lossy().into_owned())
}
