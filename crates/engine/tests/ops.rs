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
fn repository_revert_policy_is_shared_persistent_and_checked_at_execution() {
    let dir = init("revert-policy");
    commit_file(&dir, 1, "a.txt", "base\n");
    commit_file(&dir, 2, "a.txt", "second\n");
    let repo = Repo::open(&dir).unwrap();
    let second = refs::list(&repo).unwrap().head_id.unwrap();
    commit_file(&dir, 3, "a.txt", "third\n");
    let head = refs::list(&repo).unwrap().head_id;
    assert!(!refs::list(&repo).unwrap().revert_disabled);
    let linked_path = dir.join("linked");
    git(&dir, 0, &["worktree", "add", "-q", "-b", "linked", linked_path.to_str().unwrap()]);
    let linked = Repo::open(&linked_path).unwrap();
    let other = Repo::open(&init("revert-policy-other")).unwrap();

    ok(&repo, Op::SetRevertDisabled { disabled: true });
    assert!(refs::list(&Repo::open(&dir).unwrap()).unwrap().revert_disabled);
    assert!(refs::list(&linked).unwrap().revert_disabled);
    assert!(!refs::list(&other).unwrap().revert_disabled);
    for r in [&repo, &linked] {
        assert_eq!(ops::run(r, Op::Revert { id: "HEAD".into() }).unwrap_err(), "PR_REVERT_DISABLED");
        assert_eq!(ops::run(r, Op::Continue { what: InProgress::Revert }).unwrap_err(), "PR_REVERT_DISABLED");
    }
    assert_eq!(refs::list(&repo).unwrap().head_id, head);
    assert_eq!(std::fs::read_to_string(dir.join("a.txt")).unwrap(), "third\n");

    // 在另一工作树恢复设置后，原会话立即生效；中途禁用不能阻止安全中止。
    ok(&linked, Op::SetRevertDisabled { disabled: false });
    assert!(!refs::list(&repo).unwrap().revert_disabled);
    assert!(!ops::run(&repo, Op::Revert { id: second }).unwrap().ok);
    assert_eq!(refs::list(&repo).unwrap().in_progress, Some("revert"));
    ok(&repo, Op::SetRevertDisabled { disabled: true });
    assert_eq!(ops::run(&repo, Op::Continue { what: InProgress::Revert }).unwrap_err(), "PR_REVERT_DISABLED");
    ok(&repo, Op::Abort { what: InProgress::Revert });
    assert_eq!(refs::list(&repo).unwrap().head_id, head);
    assert_eq!(std::fs::read_to_string(dir.join("a.txt")).unwrap(), "third\n");
    assert!(refs::list(&repo).unwrap().in_progress.is_none());

    git(&dir, 0, &["config", "--local", "pushright.disableRevert", "invalid"]);
    assert!(ops::run(&repo, Op::Revert { id: "HEAD".into() }).is_err());
    ok(&repo, Op::SetRevertDisabled { disabled: false });
    ok(&repo, Op::Revert { id: "HEAD".into() });
    assert_eq!(std::fs::read_to_string(dir.join("a.txt")).unwrap(), "second\n");
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

    assert_eq!(
        refs::tracking(&repo).unwrap(),
        [
            refs::Track { name: "refs/heads/feat".into(), ahead: 1, behind: 0, gone: false },
            refs::Track { name: "refs/heads/main".into(), ahead: 0, behind: 0, gone: false },
        ]
    );

    // 远程前进后：fetch 看到落后，pull 跟上
    ok(&repo, Op::Checkout { target: "main".into() });
    commit_file(&origin, 4, "d.txt", "d\n");
    ok(&repo, Op::Fetch);
    assert_eq!(refs::list(&repo).unwrap().ahead_behind, Some((0, 1)));
    ok(&repo, Op::Pull { rebase: true });
    assert_eq!(refs::list(&repo).unwrap().ahead_behind, Some((0, 0)));

    // 检出远程分支：建立跟踪它的本地分支
    git(&origin, 0, &["branch", "remote-only"]);
    ok(&repo, Op::Fetch);
    ok(&repo, Op::Track { remote_branch: "origin/remote-only".into() });
    assert_eq!(upstream_of("refs/heads/remote-only").as_deref(), Some("refs/remotes/origin/remote-only"));

    std::fs::remove_dir_all(&dir).unwrap();
    std::fs::remove_dir_all(&origin).unwrap();
}

#[test]
fn pull_explicit_mode_and_conflict_recovery() {
    let origin = init("pull-origin");
    commit_file(&origin, 1, "a.txt", "base\n");
    let dir = init("pull-local");
    git(&dir, 0, &["remote", "add", "origin", origin.to_str().unwrap()]);
    git(&dir, 0, &["fetch", "-q", "origin"]);
    git(&dir, 0, &["checkout", "-B", "main", "origin/main"]);
    let repo = Repo::open(&dir).unwrap();
    commit_file(&dir, 2, "local.txt", "local\n");
    let before = id_of(&repo, "refs/heads/main").unwrap();
    git(&dir, 0, &["branch", "keep-original"]);
    commit_file(&origin, 3, "remote.txt", "remote\n");
    // Explicit rebase wins over merge preferences and does not move sibling branches.
    git(&dir, 0, &["config", "pull.rebase", "false"]);
    git(&dir, 0, &["config", "branch.main.rebase", "false"]);
    git(&dir, 0, &["config", "rebase.updateRefs", "true"]);
    git(&dir, 0, &["config", "pull.ff", "only"]);
    let log = ops::run(&repo, Op::Pull { rebase: true }).unwrap();
    assert!(log.ok, "{}", log.output);
    assert!(log.command.contains("pull --rebase"));
    assert_ne!(id_of(&repo, "refs/heads/main"), Some(before.clone()));
    assert_eq!(id_of(&repo, "refs/heads/keep-original"), Some(before));
    git(&dir, 0, &["merge-base", "--is-ancestor", "origin/main", "HEAD"]);
    let parents = std::process::Command::new("git").current_dir(&dir)
        .args(["rev-list", "--parents", "-1", "HEAD"]).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&parents.stdout).split_whitespace().count(), 2);

    // Unchecking rebase must override a repository that prefers rebase.
    git(&dir, 0, &["config", "pull.rebase", "true"]);
    git(&dir, 0, &["config", "branch.main.rebase", "true"]);
    git(&dir, 0, &["config", "pull.ff", "true"]);
    commit_file(&origin, 4, "remote2.txt", "remote2\n");
    ok(&repo, Op::Pull { rebase: false });
    let parents = std::process::Command::new("git").current_dir(&dir)
        .args(["rev-list", "--parents", "-1", "HEAD"]).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&parents.stdout).split_whitespace().count(), 3);

    // A conflicting pull pauses; Abort restores the local tip, Continue keeps the resolution.
    commit_file(&dir, 5, "a.txt", "local\n");
    let before_conflict = id_of(&repo, "refs/heads/main").unwrap();
    commit_file(&origin, 6, "a.txt", "remote\n");
    assert!(!ops::run(&repo, Op::Pull { rebase: true }).unwrap().ok);
    assert_eq!(refs::list(&repo).unwrap().in_progress, Some("rebase"));
    ok(&repo, Op::Abort { what: InProgress::Rebase });
    assert_eq!(id_of(&repo, "refs/heads/main"), Some(before_conflict));
    assert!(!ops::run(&repo, Op::Pull { rebase: true }).unwrap().ok);
    std::fs::write(dir.join("a.txt"), "resolved\n").unwrap();
    status::stage(&repo, &["a.txt".into()]).unwrap();
    ok(&repo, Op::Continue { what: InProgress::Rebase });
    assert_eq!(refs::list(&repo).unwrap().in_progress, None);
    assert_eq!(std::fs::read_to_string(dir.join("a.txt")).unwrap(), "resolved\n");
    std::fs::remove_dir_all(&dir).unwrap();
    std::fs::remove_dir_all(&origin).unwrap();
}

#[test]
fn worktree_add_overview_remove() {
    let dir = init("ops-wt");
    commit_file(&dir, 1, "a.txt", "a\n");
    let repo = Repo::open(&dir).unwrap();
    let other = std::env::temp_dir().join(format!("gitgod-test-ops-wt-other-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&other);
    let path = other.to_str().unwrap().to_owned();

    ok(&repo, Op::WorktreeAdd { path: path.clone(), start: "main".into(), new_branch: Some("side".into()) });
    std::fs::write(other.join("new.txt"), "n\n").unwrap();
    std::fs::write(other.join("a.txt"), "changed\n").unwrap();

    let list = refs::worktrees(&repo).unwrap();
    assert_eq!(list.len(), 2);
    let brief: Vec<_> = list.iter().map(|w| (w.branch.as_deref(), w.current, w.changes, w.subject.as_str())).collect();
    assert_eq!(brief, [(Some("refs/heads/main"), true, 0, "a.txt"), (Some("refs/heads/side"), false, 2, "a.txt")]);
    assert_eq!(list[1].time, 1_700_000_001);
    // 从另一个工作树打开时，current 跟着变
    let from_other = refs::worktrees(&Repo::open(&other).unwrap()).unwrap();
    assert_eq!(from_other.iter().map(|w| w.current).collect::<Vec<_>>(), [false, true]);

    // 分支已在别的工作树检出：git 拒绝，日志里是失败
    assert!(!ops::run(&repo, Op::Checkout { target: "side".into() }).unwrap().ok);
    // 有改动的工作树不强制就删不掉
    assert!(!ops::run(&repo, Op::WorktreeRemove { path: path.clone(), force: false }).unwrap().ok);
    ok(&repo, Op::WorktreeRemove { path, force: true });
    assert_eq!(refs::worktrees(&repo).unwrap().len(), 1);

    std::fs::remove_dir_all(&dir).unwrap();
}
