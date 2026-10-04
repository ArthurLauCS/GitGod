//! 读路径回归基准。仓库路径由环境变量 GITGOD_BENCH_REPO 指定，默认 ../../../GitGod-bench/linux。
//! M0 的候选对比（git CLI / gix / git2）见提交 f6d95b3 的同名文件。

use criterion::{criterion_group, criterion_main, Criterion};
use engine::{detail, graph::Graph, refs, Repo};

fn repo() -> Repo {
    let path = std::env::var("GITGOD_BENCH_REPO").unwrap_or_else(|_| "../../../GitGod-bench/linux".into());
    Repo::open(path.as_ref()).unwrap()
}

fn bench(c: &mut Criterion) {
    let repo = repo();
    let mut g = c.benchmark_group("graph_load");
    g.sample_size(10);
    g.bench_function("first_2000", |b| b.iter(|| Graph::load(&repo, 2000).unwrap().len()));
    g.bench_function("300k", |b| b.iter(|| Graph::load(&repo, 300_000).unwrap().len()));
    g.bench_function("full", |b| b.iter(|| Graph::load(&repo, usize::MAX).unwrap().len()));
    g.finish();

    let graph = Graph::load(&repo, 300_000).unwrap();
    let id = graph.rows(&repo, 150_000, 1).unwrap().remove(0).id;
    // 窗口起点取在两个快照正中间之后，重放成本最大
    c.bench_function("rows_60_before_checkpoint", |b| b.iter(|| graph.rows(&repo, 150_000 + 1023, 60).unwrap().len()));
    c.bench_function("refs_list", |b| b.iter(|| refs::list(&repo).unwrap().refs.len()));
    c.bench_function("detail", |b| b.iter(|| detail::detail(&repo, &id).unwrap().files.len()));
}

criterion_group!(benches, bench);
criterion_main!(benches);
