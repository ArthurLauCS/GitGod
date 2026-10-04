use std::path::{Path, PathBuf};
use std::process::Command;

/// 在 `dir` 里跑 git，作者/提交时间固定为基准时间加 `time` 秒。
pub fn git(dir: &Path, time: u32, args: &[&str]) {
    let date = format!("@{} +0000", 1_700_000_000 + time);
    let ok = Command::new("git")
        .current_dir(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false", "-c", "core.autocrlf=false"])
        .args(args)
        .env("GIT_AUTHOR_DATE", &date)
        .env("GIT_COMMITTER_DATE", &date)
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?}");
}

/// 新建一个空仓库（main 分支），目录名带测试名和进程号。
pub fn init(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gitgod-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    git(&dir, 0, &["init", "-q", "-b", "main"]);
    for kv in [["user.name", "t"], ["user.email", "t@t"], ["commit.gpgsign", "false"], ["core.autocrlf", "false"]] {
        git(&dir, 0, &["config", kv[0], kv[1]]);
    }
    dir
}
