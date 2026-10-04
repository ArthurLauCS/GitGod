//! M0 读路径选型基准：git CLI / gix 两个候选（git2 提交加载慢 5 倍以上，已淘汰）在同一仓库上的对比。
//! 仓库路径由环境变量 GITGOD_BENCH_REPO 指定，默认 ../../../GitGod-bench/linux。
//! 落选的候选在选型结束后连同其 dev-dependency 一起删除。

use criterion::{criterion_group, criterion_main, Criterion};
use engine::log::Commit;
use engine::{graph, log};
use std::hint::black_box;
use std::path::PathBuf;
use std::process::Command;

const N: usize = 300_000;
const PAGE: usize = 100;

fn repo() -> PathBuf {
    std::env::var("GITGOD_BENCH_REPO").unwrap_or_else(|_| "../../../GitGod-bench/linux".into()).into()
}

fn git(args: &[&str]) -> Vec<u8> {
    let out = Command::new("git").arg("-C").arg(repo()).args(args).output().unwrap();
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    out.stdout
}

fn cli_log(args: &[&str]) -> Vec<Commit> {
    log::load(&repo(), args).unwrap()
}

fn gix_log(n: usize) -> Vec<Commit> {
    let repo = gix::open(repo()).unwrap();
    let head = repo.head_id().unwrap().detach();
    repo.rev_walk([head])
        .sorting(gix::revision::walk::Sorting::ByCommitTime(Default::default()))
        .all()
        .unwrap()
        .take(n)
        .map(|info| {
            let info = info.unwrap();
            let commit = info.object().unwrap();
            let c = commit.decode().unwrap();
            Commit {
                id: info.id.as_bytes().try_into().unwrap(),
                parents: c.parents().map(|p| p.as_bytes().try_into().unwrap()).collect(),
                time: c.committer().unwrap().seconds(),
                author: c.author().unwrap().name.to_string(),
                subject: c.message().title.to_string(),
            }
        })
        .collect()
}

/// 只取图骨架（id / 父 / 时间），不解码提交对象
fn gix_skeleton(n: usize) -> usize {
    let repo = gix::open(repo()).unwrap();
    let head = repo.head_id().unwrap().detach();
    repo.rev_walk([head])
        .sorting(gix::revision::walk::Sorting::ByCommitTime(Default::default()))
        .all()
        .unwrap()
        .take(n)
        .map(|info| {
            let info = info.unwrap();
            black_box((info.id, &info.parent_ids, info.commit_time));
        })
        .count()
}

fn bench_skeleton(c: &mut Criterion) {
    let n = N.to_string();
    let mut g = c.benchmark_group("skeleton_300k");
    g.sample_size(10);
    g.bench_function("cli_rev_list", |b| b.iter(|| git(&["rev-list", "--parents", "--timestamp", "-n", &n, "HEAD"])));
    g.bench_function("gix", |b| b.iter(|| gix_skeleton(N)));
    g.finish();
    let mut g = c.benchmark_group("skeleton_first_page");
    g.bench_function("cli_rev_list", |b| b.iter(|| git(&["rev-list", "--parents", "--timestamp", "-n", "100", "HEAD"])));
    g.bench_function("gix", |b| b.iter(|| gix_skeleton(PAGE)));
    g.finish();
}

fn bench_log(c: &mut Criterion) {
    let n = N.to_string();
    let page = PAGE.to_string();
    assert_eq!(cli_log(&["-n", &n]).len(), N);
    assert_eq!(gix_log(N).len(), N);

    let mut g = c.benchmark_group("log_300k");
    g.sample_size(10);
    g.bench_function("cli", |b| b.iter(|| cli_log(&["-n", &n])));
    g.bench_function("cli_topo", |b| b.iter(|| cli_log(&["--topo-order", "-n", &n])));
    g.bench_function("cli_all_date_order", |b| b.iter(|| cli_log(&["--all", "--date-order", "-n", &n])));
    g.bench_function("gix", |b| b.iter(|| gix_log(N)));
    g.finish();

    // 首屏：从零（含打开仓库 / 启动进程）到拿到前 100 条
    let mut g = c.benchmark_group("log_first_page");
    g.bench_function("cli", |b| b.iter(|| cli_log(&["-n", &page])));
    g.bench_function("cli_all_date_order", |b| b.iter(|| cli_log(&["--all", "--date-order", "-n", &page])));
    g.bench_function("gix", |b| b.iter(|| gix_log(PAGE)));
    g.finish();
}

fn bench_status(c: &mut Criterion) {
    let mut g = c.benchmark_group("status");
    g.sample_size(10);
    g.bench_function("cli", |b| {
        b.iter(|| git(&["-c", "core.fsmonitor=false", "-c", "core.untrackedCache=false", "status", "--porcelain=v2", "-z"]))
    });
    // 先跑一次让 fsmonitor 守护进程启动并建好 untracked cache
    git(&["-c", "core.fsmonitor=true", "-c", "core.untrackedCache=true", "status", "--porcelain=v2", "-z"]);
    g.bench_function("cli_fsmonitor", |b| {
        b.iter(|| git(&["-c", "core.fsmonitor=true", "-c", "core.untrackedCache=true", "status", "--porcelain=v2", "-z"]))
    });
    g.bench_function("gix", |b| {
        b.iter(|| {
            let repo = gix::open(repo()).unwrap();
            repo.status(gix::progress::Discard).unwrap().into_iter(None).unwrap().count()
        })
    });
    g.finish();
}

fn bench_refs(c: &mut Criterion) {
    let mut g = c.benchmark_group("refs");
    g.bench_function("cli", |b| b.iter(|| git(&["for-each-ref", "--format=%(refname)%00%(objectname)%00%(upstream)"])));
    g.bench_function("cli_ahead_behind", |b| {
        b.iter(|| git(&["for-each-ref", "--format=%(refname)%00%(objectname)%00%(upstream)%00%(ahead-behind:HEAD)"]))
    });
    g.bench_function("gix", |b| {
        b.iter(|| {
            let repo = gix::open(repo()).unwrap();
            let n = repo.references().unwrap().all().unwrap().map(|r| black_box(r.unwrap().try_id())).count();
            n
        })
    });
    g.finish();
}

fn bench_diff(c: &mut Criterion) {
    let sha = String::from_utf8(git(&["rev-list", "--no-merges", "-n", "1", "--skip", "100", "HEAD"])).unwrap();
    let sha = sha.trim();
    let mut g = c.benchmark_group("diff_commit");
    g.bench_function("cli", |b| b.iter(|| git(&["diff-tree", "-p", "-r", "--no-commit-id", sha])));
    g.finish();
}

fn bench_graph(c: &mut Criterion) {
    let commits = cli_log(&["--all", "--date-order", "-n", &N.to_string()]);
    let mut g = c.benchmark_group("graph_layout");
    g.bench_function("full_300k", |b| b.iter(|| graph::layout(&commits)));
    g.bench_function("first_1000", |b| b.iter(|| graph::layout(&commits[..1000])));
    g.finish();
    let l = graph::layout(&commits);
    println!("图宽（最大泳道数）: {}", l.lanes.iter().max().unwrap() + 1);
}

criterion_group!(benches, bench_skeleton, bench_log, bench_status, bench_refs, bench_diff, bench_graph);
criterion_main!(benches);
