mod common;

use common::{git, init};
use engine::history::{self, SearchKind};
use engine::Repo;

#[test]
fn file_and_line_history_search_and_show() {
    let dir = init("history");
    std::fs::write(dir.join("a.txt"), "one\ntwo\nthree\n").unwrap();
    git(&dir, 0, &["add", "."]);
    git(&dir, 1, &["commit", "-q", "-m", "add file"]);
    std::fs::write(dir.join("a.txt"), "one\ntwo\nTHREE\n").unwrap();
    git(&dir, 2, &["commit", "-q", "-am", "shout three"]);
    git(&dir, 0, &["mv", "a.txt", "新名.txt"]);
    git(&dir, 3, &["-c", "user.name=other", "commit", "-q", "-m", "rename"]);
    std::fs::write(dir.join("新名.txt"), "ONE\ntwo\nTHREE\n").unwrap();
    git(&dir, 0, &["add", "."]);
    let repo = Repo::open(&dir).unwrap();

    let file = history::file(&repo, "新名.txt", 100).unwrap();
    let brief: Vec<_> = file.iter().map(|c| (c.subject.as_str(), c.path.as_str())).collect();
    assert_eq!(brief, [("rename", "新名.txt"), ("shout three", "a.txt"), ("add file", "a.txt")]);
    assert_eq!((file[0].author.as_str(), file[0].author_email.as_str(), file[0].time), ("other", "t@t", 1_700_000_003));
    assert_eq!(history::file(&repo, "新名.txt", 1).unwrap().len(), 1);

    // 第 1 行只在创建时写过，第 3 行后来又改过一次
    let subjects = |start, end| -> Vec<String> {
        history::lines(&repo, "a.txt", start, end, 100).unwrap_or_default().into_iter().map(|c| c.subject).collect()
    };
    git(&dir, 0, &["stash", "-q"]);
    git(&dir, 0, &["checkout", "-q", "HEAD~1"]);
    assert_eq!(subjects(1, 1), ["add file"]);
    assert_eq!(subjects(3, 3), ["shout three", "add file"]);

    let found = |kind, q: &str| -> Vec<String> { history::search(&repo, kind, q, 100).unwrap().into_iter().map(|c| c.subject).collect() };
    assert_eq!(found(SearchKind::Message, "SHOUT"), ["shout three"]);
    assert_eq!(found(SearchKind::Author, "other"), ["rename"]);
    assert_eq!(found(SearchKind::Content, "THREE"), ["shout three"]);
    assert_eq!(found(SearchKind::File, "a.txt"), ["rename", "shout three", "add file"]);
    assert_eq!(found(SearchKind::Id, &file[2].id), ["add file"]);
    assert!(history::search(&repo, SearchKind::Id, "--all", 100).is_err());

    assert_eq!(history::show(&repo, &file[2].id, "a.txt").unwrap(), "one\ntwo\nthree\n");
    assert_eq!(history::show(&repo, "main", "新名.txt").unwrap(), "one\ntwo\nTHREE\n");
    assert!(history::show(&repo, "--help", "a.txt").is_err());
    assert!(history::show(&repo, "main", "missing.txt").is_err());
}
