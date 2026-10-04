//! cargo run --release -p engine --example diff_probe -- <repo> <path> <untracked|staged>
//! GITGOD_BENCH_HOLD=1 时输出结果后等待回车，供父进程读取峰值内存。
use engine::{diff, Repo};
use std::time::Instant;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(args.len() == 4, "expected <repo> <path> <untracked|staged>");
    assert!(matches!(args[3].as_str(), "untracked" | "staged"));
    let repo = Repo::open(args[1].as_ref()).unwrap();
    let start = Instant::now();
    let result = diff::worktree(&repo, &args[2], args[3] == "staged", args[3] == "untracked").unwrap();
    println!("elapsed_ms={:.3} too_large={}", start.elapsed().as_secs_f64() * 1000.0, result.too_large);
    if std::env::var_os("GITGOD_BENCH_HOLD").is_some() {
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}
