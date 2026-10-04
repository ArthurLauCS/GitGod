mod common;

use common::{git, init};
use engine::{detail, graph::Graph, refs, Repo};
use std::path::PathBuf;

/// main: A → C(把 a.txt 改名为 c.txt) → M(合并 feat)，feat: A → B(新增 b.txt)，标签 v1 在 M
fn fixture(name: &str) -> PathBuf {
    let dir = init(name);
    std::fs::write(dir.join("a.txt"), "hello\nworld\n").unwrap();
    git(&dir, 0, &["add", "."]);
    git(&dir, 1, &["commit", "-q", "-m", "A"]);
    git(&dir, 0, &["checkout", "-q", "-b", "feat"]);
    std::fs::write(dir.join("b.txt"), "b\n").unwrap();
    git(&dir, 0, &["add", "."]);
    git(&dir, 2, &["commit", "-q", "-m", "B"]);
    git(&dir, 0, &["checkout", "-q", "main"]);
    git(&dir, 0, &["mv", "a.txt", "c.txt"]);
    git(&dir, 3, &["commit", "-q", "-m", "C"]);
    git(&dir, 4, &["merge", "-q", "--no-ff", "-m", "M\n\n正文", "feat"]);
    git(&dir, 0, &["tag", "v1"]);
    dir
}

#[test]
fn graph_rows_refs_and_detail() {
    let dir = fixture("browse");
    let repo = Repo::open(&dir).unwrap();

    let graph = Graph::load(&repo, usize::MAX).unwrap();
    let rows = graph.rows(&repo, 0, 100).unwrap();
    assert_eq!(rows.iter().map(|r| r.subject.as_str()).collect::<Vec<_>>(), ["M", "C", "B", "A"]);
    assert_eq!(rows.iter().map(|r| r.lane).collect::<Vec<_>>(), [0, 0, 1, 0]);
    assert_eq!(rows[0].out, [0, 1]);
    assert_eq!(rows[1].through, [1]);
    assert_eq!(rows[3].incoming, [0, 1]);
    assert_eq!(rows[0].author, "t");
    assert_eq!(rows[0].time, 1_700_000_004);
    // 窗口从中间开始时泳道状态与从头算一致
    let tail = graph.rows(&repo, 2, 100).unwrap();
    assert_eq!((tail.len(), tail[0].lane, &tail[1].incoming), (2, 1, &vec![0, 1]));
    assert_eq!(graph.row_of(&rows[2].id), Some(2));
    // 截断加载：只取最近 2 个提交
    assert_eq!(Graph::load(&repo, 2).unwrap().len(), 2);

    let r = refs::list(&repo).unwrap();
    assert_eq!(r.head.as_deref(), Some("refs/heads/main"));
    assert_eq!(r.head_id.as_deref(), Some(rows[0].id.as_str()));
    let id_of = |name: &str| r.refs.iter().find(|x| x.name == name).map(|x| x.id.as_str());
    assert_eq!(id_of("refs/heads/feat"), Some(rows[2].id.as_str()));
    assert_eq!(id_of("refs/tags/v1"), Some(rows[0].id.as_str()));

    let merge = detail::detail(&repo, &rows[0].id).unwrap();
    assert_eq!(merge.message, "M\n\n正文");
    assert_eq!(merge.parents, [rows[1].id.clone(), rows[2].id.clone()]);
    assert_eq!(merge.files.iter().map(|f| (f.status, f.path.as_str())).collect::<Vec<_>>(), [('A', "b.txt")]);
    let rename = detail::detail(&repo, &rows[1].id).unwrap();
    assert_eq!((rename.files[0].status, rename.files[0].old_path.as_deref(), rename.files[0].path.as_str()), ('R', Some("a.txt"), "c.txt"));
    let root = detail::detail(&repo, &rows[3].id).unwrap();
    assert_eq!(root.files[0].path, "a.txt");
    assert!(detail::detail(&repo, "--output=x").is_err());

    assert!(refs::stashes(&repo).unwrap().is_empty());
    let wt = refs::worktrees(&repo).unwrap();
    assert_eq!((wt.len(), wt[0].branch.as_deref(), wt[0].head.as_str()), (1, Some("refs/heads/main"), rows[0].id.as_str()));

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn empty_repo() {
    let dir = init("empty");
    let repo = Repo::open(&dir).unwrap();
    let graph = Graph::load(&repo, usize::MAX).unwrap();
    assert_eq!(graph.len(), 0);
    assert!(graph.rows(&repo, 0, 100).unwrap().is_empty());
    let r = refs::list(&repo).unwrap();
    assert_eq!((r.head.as_deref(), r.head_id, r.refs.len()), (Some("refs/heads/main"), None, 0));
    std::fs::remove_dir_all(&dir).unwrap();
}
