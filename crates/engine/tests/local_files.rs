mod common;
use common::{git, init};
use engine::{local_files as local, status, Repo};
use std::fs;

#[test]
fn ignore_track_and_nested_repositories_preserve_local_content() {
    let dir = init("local-files");
    let repo = Repo::open(&dir).unwrap();
    fs::write(dir.join("tracked.txt"), "committed\n").unwrap();
    git(&dir, 0, &["add", "."]);
    git(&dir, 1, &["commit", "-q", "-m", "root"]);
    fs::write(dir.join("skill[1].md"), "private skill\n").unwrap();
    fs::write(dir.join("skill1.md"), "public skill\n").unwrap();
    local::ignore_file(&repo, "skill[1].md", false).unwrap();
    local::ignore_file(&repo, "skill[1].md", false).unwrap();
    assert_eq!(fs::read_to_string(dir.join(".git/info/exclude")).unwrap().matches("/skill\\[1\\].md").count(), 1);
    assert!(!dir.join(".gitignore").exists());
    assert_eq!(local::ignored(&repo, "").unwrap(), ["skill[1].md"]);
    assert_eq!(status::status(&repo).unwrap()[0].path, "skill1.md");
    assert!(local::ignore_file(&repo, "tracked.txt", false).is_err());
    for path in ["../escape", ".git/config", "a\nb"] { assert!(local::ignore_file(&repo, path, false).is_err()); }

    local::track(&repo, "skill[1].md", true).unwrap();
    assert!(local::tracked(&repo, "skill[1].md").unwrap());
    fs::write(dir.join("skill[1].md"), "new unstaged content\n").unwrap();
    assert!(local::track(&repo, "skill[1].md", false).is_err(), "must not discard independently staged contents");
    assert!(local::tracked(&repo, "skill[1].md").unwrap());
    local::track(&repo, "tracked.txt", false).unwrap();
    assert_eq!(fs::read_to_string(dir.join("tracked.txt")).unwrap(), "committed\n");
    assert!(!local::tracked(&repo, "tracked.txt").unwrap());
    assert!(status::status(&repo).unwrap().iter().any(|e| e.path == "tracked.txt" && e.staged == Some('D')));

    fs::create_dir_all(dir.join("private/deep")).unwrap();
    fs::write(dir.join("private/a.md"), "a").unwrap();
    fs::write(dir.join("private/deep/b.md"), "b").unwrap();
    local::ignore_file(&repo, "private", true).unwrap();
    assert_eq!(fs::read_to_string(dir.join(".gitignore")).unwrap(), "/private/\n");
    assert!(local::ignored(&repo, "").unwrap().contains(&"private/".to_owned()));
    assert_eq!(local::ignored(&repo, "private").unwrap(), ["private/a.md", "private/deep/"]);
    assert_eq!(local::ignored(&repo, "private/deep").unwrap(), ["private/deep/b.md"]);

    let child = dir.join("nested/repo");
    fs::create_dir_all(&child).unwrap();
    git(&child, 0, &["init", "-q", "-b", "main"]);
    git(&child, 0, &["commit", "-q", "--allow-empty", "-m", "child"]);
    assert_eq!(local::repositories(&repo).unwrap(), [child.to_string_lossy()]);
    git(&dir, 0, &["add", "nested/repo"]);
    assert_eq!(local::repositories(&repo).unwrap(), [child.to_string_lossy()]);
    fs::remove_dir_all(dir).unwrap();
}
