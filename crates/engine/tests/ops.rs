mod common;

use common::{git, init};
use engine::ops::{self, InProgress, Op, ResetMode};
use engine::{refs, status, Repo};
use std::path::Path;

fn ok(repo: &Repo, op: Op) {
    let log = ops::run(repo, op).unwrap();
    assert!(log.ok, "{}\n{}", log.command, log.output);
}

fn commit_file(dir: &Path, time: u32, name: &str, content: &str) {
    std::fs::write(dir.join(name), content).unwrap();
    git(dir, 0, &["add", "."]);
    git(dir, time, &["commit", "-q", "-m", name]);
}

fn id_of(repo: &Repo, name: &str) -> Option<String> {
    refs::list(repo).unwrap().refs.into_iter().find(|r| r.name == name).map(|r| r.id)
}

#[test]
fn branches_tags_stash_reset() {
    let dir = init("ops-local");
    commit_file(&dir, 1, "a.txt", "a\n");
    let repo = Repo::open(&dir).unwrap();
    let base = id_of(&repo, "refs/heads/main").unwrap();

    ok(&repo, Op::CreateBranch { name: "feat".into(), start: "main".into(), checkout: true });
    assert_eq!(refs::list(&repo).unwrap().head.as_deref(), Some("refs/heads/feat"));
    commit_file(&dir, 2, "b.txt", "b\n");
    let feat = id_of(&repo, "refs/heads/feat").unwrap();

    ok(&repo, Op::Checkout { target: "main".into() });
    ok(&repo, Op::Merge { target: "feat".into() });
    assert_eq!(id_of(&repo, "refs/heads/main"), Some(feat.clone()));

    ok(&repo, Op::RenameBranch { old: "feat".into(), new: "feat2".into() });
    ok(&repo, Op::DeleteBranch { name: "feat2".into(), force: false });
    assert_eq!(id_of(&repo, "refs/heads/feat2"), None);

    ok(&repo, Op::CreateTag { name: "v1".into(), target: base.clone(), message: String::new() });
    ok(&repo, Op::CreateTag { name: "v2".into(), target: "HEAD".into(), message: "带说明".into() });
    assert_eq!(id_of(&repo, "refs/tags/v1"), Some(base.clone()));
    assert_eq!(id_of(&repo, "refs/tags/v2"), Some(feat.clone()));
    ok(&repo, Op::DeleteTag { name: "v1".into() });
    assert_eq!(id_of(&repo, "refs/tags/v1"), None);

    // revert 生成新提交，cherry-pick 到新分支上再把它带回来
    ok(&repo, Op::Revert { id: feat.clone() });
    assert!(!dir.join("b.txt").exists());
    ok(&repo, Op::CherryPick { id: feat.clone() });
    assert!(dir.join("b.txt").exists());

    ok(&repo, Op::Reset { target: base.clone(), mode: ResetMode::Hard });
    assert_eq!(id_of(&repo, "refs/heads/main"), Some(base.clone()));
    assert!(!dir.join("b.txt").exists());

    // 贮藏：含未跟踪文件，弹出后恢复
    std::fs::write(dir.join("a.txt"), "changed\n").unwrap();
    std::fs::write(dir.join("new.txt"), "n\n").unwrap();
    ok(&repo, Op::StashPush { message: "半成品".into(), include_untracked: true });
    assert!(status::status(&repo).unwrap().is_empty());
    let stashes = refs::stashes(&repo).unwrap();
    assert!(stashes[0].subject.contains("半成品"));
    ok(&repo, Op::StashApply { name: stashes[0].name.clone(), pop: true });
    assert_eq!(status::status(&repo).unwrap().len(), 2);
    assert!(refs::stashes(&repo).unwrap().is_empty());

    // 以 - 开头的名字不会被拼进命令行
    assert!(ops::run(&repo, Op::Checkout { target: "--orphan".into() }).is_err());
    // git 自己拒绝的操作：日志里带失败输出
    let log = ops::run(&repo, Op::DeleteBranch { name: "不存在".into(), force: false }).unwrap();
    assert!(!log.ok && log.command == "git branch -d 不存在" && !log.output.is_empty());

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn conflict_state_and_abort() {
    let dir = init("ops-conflict");
    commit_file(&dir, 1, "a.txt", "base\n");
    let repo = Repo::open(&dir).unwrap();
    ok(&repo, Op::CreateBranch { name: "other".into(), start: "main".into(), checkout: true });
    commit_file(&dir, 2, "a.txt", "other\n");
    ok(&repo, Op::Checkout { target: "main".into() });
    commit_file(&dir, 3, "a.txt", "main\n");

    let log = ops::run(&repo, Op::Merge { target: "other".into() }).unwrap();
    assert!(!log.ok);
    assert_eq!(refs::list(&repo).unwrap().in_progress, Some("merge"));
    assert!(status::status(&repo).unwrap()[0].conflicted);
    ok(&repo, Op::Abort { what: InProgress::Merge });
    assert_eq!(refs::list(&repo).unwrap().in_progress, None);

    // 解决冲突后继续
    assert!(!ops::run(&repo, Op::Rebase { onto: "other".into() }).unwrap().ok);
    assert_eq!(refs::list(&repo).unwrap().in_progress, Some("rebase"));
    std::fs::write(dir.join("a.txt"), "resolved\n").unwrap();
    status::stage(&repo, &["a.txt".into()]).unwrap();
    ok(&repo, Op::Continue { what: InProgress::Rebase });
    assert_eq!(refs::list(&repo).unwrap().in_progress, None);

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn remote_fetch_push_pull_and_upstream() {
    let origin = init("ops-origin");
    commit_file(&origin, 1, "a.txt", "a\n");
    git(&origin, 0, &["config", "receive.denyCurrentBranch", "ignore"]);
    let dir = std::env::temp_dir().join(format!("gitgod-test-ops-clone-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    git(&origin, 0, &["clone", "-q", ".", dir.to_str().unwrap()]);
    for kv in [["user.name", "t"], ["user.email", "t@t"]] {
        git(&dir, 0, &["config", kv[0], kv[1]]);
    }
    let repo = Repo::open(&dir).unwrap();
    assert_eq!(refs::remotes(&repo).unwrap(), ["origin"]);
    let upstream_of = |name: &str| refs::list(&repo).unwrap().refs.into_iter().find(|r| r.name == name).unwrap().upstream;
    assert_eq!(upstream_of("refs/heads/main").as_deref(), Some("refs/remotes/origin/main"));
    assert_eq!(refs::list(&repo).unwrap().ahead_behind, Some((0, 0)));

    // 新分支没有上游；推到同名远程分支并设置上游
    ok(&repo, Op::CreateBranch { name: "feat".into(), start: "main".into(), checkout: true });
    assert_eq!(upstream_of("refs/heads/feat"), None);
    commit_file(&dir, 2, "b.txt", "b\n");
    ok(&repo, Op::Push { remote: "origin".into(), branch: "feat".into(), remote_branch: "feat".into(), force: false, set_upstream: true });
    assert_eq!(upstream_of("refs/heads/feat").as_deref(), Some("refs/remotes/origin/feat"));
    assert!(id_of(&Repo::open(&origin).unwrap(), "refs/heads/feat").is_some());

    // 本地领先 1 个
    commit_file(&dir, 3, "c.txt", "c\n");
    assert_eq!(refs::list(&repo).unwrap().ahead_behind, Some((1, 0)));

    // 远程前进后：fetch 看到落后，pull 跟上
    ok(&repo, Op::Checkout { target: "main".into() });
    commit_file(&origin, 4, "d.txt", "d\n");
    ok(&repo, Op::Fetch);
    assert_eq!(refs::list(&repo).unwrap().ahead_behind, Some((0, 1)));
    ok(&repo, Op::Pull);
    assert_eq!(refs::list(&repo).unwrap().ahead_behind, Some((0, 0)));

    // 检出远程分支：建立跟踪它的本地分支
    git(&origin, 0, &["branch", "remote-only"]);
    ok(&repo, Op::Fetch);
    ok(&repo, Op::Track { remote_branch: "origin/remote-only".into() });
    assert_eq!(upstream_of("refs/heads/remote-only").as_deref(), Some("refs/remotes/origin/remote-only"));

    std::fs::remove_dir_all(&dir).unwrap();
    std::fs::remove_dir_all(&origin).unwrap();
}
