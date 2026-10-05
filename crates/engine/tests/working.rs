mod common;

use common::{git, init};
use engine::{diff, refs, status, Repo};

fn lines(from: u32, to: u32) -> String {
    (from..=to).map(|i| format!("l{i}\n")).collect()
}

fn brief(repo: &Repo) -> Vec<(String, Option<char>, Option<char>)> {
    status::status(repo).unwrap().into_iter().map(|e| (e.path, e.staged, e.unstaged)).collect()
}

fn index_content(dir: &std::path::Path, path: &str) -> String {
    let out = std::process::Command::new("git").current_dir(dir).args(["show", &format!(":{path}")]).output().unwrap();
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn stage_lines_commit_and_amend() {
    let dir = init("working");
    std::fs::write(dir.join("f.txt"), lines(1, 20)).unwrap();
    git(&dir, 0, &["add", "."]);
    git(&dir, 1, &["commit", "-q", "-m", "base"]);
    let repo = Repo::open(&dir).unwrap();
    let f = "f.txt".to_owned();

    // 两处改动相隔足够远，形成两个区块；第一个区块里既有替换也有纯新增
    let edited = lines(1, 20).replace("l2\n", "L2\nnew\n").replace("l18\n", "L18\n");
    std::fs::write(dir.join("f.txt"), &edited).unwrap();
    std::fs::write(dir.join("中文.txt"), [0xc4, 0xe3, 0xba, 0xc3, b'\n']).unwrap(); // GBK 的「你好」
    assert_eq!(brief(&repo), [("f.txt".into(), None, Some('M')), ("中文.txt".into(), None, Some('?'))]);

    let d = diff::worktree(&repo, &f, false, false).unwrap();
    assert_eq!(d.hunks.len(), 2);
    let h = &d.hunks[0];
    let shape: Vec<_> = h.lines.iter().map(|l| (l.kind, l.text.as_str(), l.old_no, l.new_no)).collect();
    assert_eq!(shape[..4], [(' ', "l1", Some(1), Some(1)), ('-', "l2", Some(2), None), ('+', "L2", None, Some(2)), ('+', "new", None, Some(3))]);

    // 只暂存「new」这一行：暂存区里 l2 保持原样，后面多一行 new
    assert!(diff::apply_lines(&repo, &f, false, 0, "@@ 过期的区块头 @@", &[3]).is_err());
    diff::apply_lines(&repo, &f, false, 0, &h.header, &[3]).unwrap();
    assert_eq!(index_content(&dir, "f.txt"), lines(1, 20).replace("l2\n", "l2\nnew\n"));
    assert_eq!(brief(&repo)[0], ("f.txt".into(), Some('M'), Some('M')));

    // 再把它从暂存区按行撤回
    let staged = diff::worktree(&repo, &f, true, false).unwrap();
    let pos = staged.hunks[0].lines.iter().position(|l| l.kind == '+').unwrap();
    diff::apply_lines(&repo, &f, true, 0, &staged.hunks[0].header, &[pos]).unwrap();
    assert_eq!(index_content(&dir, "f.txt"), lines(1, 20));

    // 暂存整个第二个区块
    let d = diff::worktree(&repo, &f, false, false).unwrap();
    let all: Vec<_> = (0..d.hunks[1].lines.len()).collect();
    diff::apply_lines(&repo, &f, false, 1, &d.hunks[1].header, &all).unwrap();
    assert_eq!(index_content(&dir, "f.txt"), lines(1, 20).replace("l18\n", "L18\n"));

    // 整文件暂存 / 取消暂存
    status::stage(&repo, &[f.clone()]).unwrap();
    assert_eq!(brief(&repo)[0], ("f.txt".into(), Some('M'), None));
    status::unstage(&repo, &[f.clone()]).unwrap();
    assert_eq!(brief(&repo)[0], ("f.txt".into(), None, Some('M')));

    // 未跟踪的 GBK 文件：解码显示，暂存后提交
    let gbk = diff::worktree(&repo, "中文.txt", false, true).unwrap();
    assert_eq!((gbk.hunks[0].lines[0].kind, gbk.hunks[0].lines[0].text.as_str()), ('+', "你好"));
    status::stage(&repo, &["中文.txt".into()]).unwrap();
    status::commit(&repo, "提交信息\n\n正文", false).unwrap();
    assert_eq!(brief(&repo), [("f.txt".into(), None, Some('M'))]);
    assert_eq!(status::last_message(&repo).unwrap(), "提交信息\n\n正文");
    let head = refs::list(&repo).unwrap().head_id.unwrap();
    let shown = diff::commit(&repo, &head, "中文.txt").unwrap();
    assert_eq!(shown.hunks[0].lines[0].text, "你好");

    status::commit(&repo, "改过的信息", true).unwrap();
    assert_eq!(status::last_message(&repo).unwrap(), "改过的信息");
    assert!(status::commit(&repo, "没有暂存内容", false).is_err());

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn untracked_directories_list_preview_and_stage_individual_files() {
    let dir = init("untracked-directory");
    std::fs::write(dir.join(".gitignore"), "*.ignored\n").unwrap();
    git(&dir, 0, &["add", ".gitignore"]);
    git(&dir, 1, &["commit", "-q", "-m", "base"]);
    std::fs::create_dir_all(dir.join("新增 目录/nested")).unwrap();
    let paths = ["新增 目录/a.txt", "新增 目录/nested/b.txt"];
    for path in paths {
        std::fs::write(dir.join(path), format!("{path}\n")).unwrap();
    }
    std::fs::write(dir.join("新增 目录/skip.ignored"), "ignored\n").unwrap();
    let repo = Repo::open(&dir).unwrap();
    let expected: Vec<_> = paths.iter().map(|p| ((*p).into(), None, Some('?'))).collect();
    assert_eq!(brief(&repo), expected);
    // 用户配置隐藏未跟踪文件时，仍按文件列出。
    git(&dir, 0, &["config", "status.showUntrackedFiles", "no"]);
    assert_eq!(brief(&repo), expected);
    for path in paths {
        let shown = diff::worktree(&repo, path, false, true).unwrap();
        assert_eq!(shown.hunks[0].lines[0].text, path);
    }
    status::stage(&repo, &[paths[0].into()]).unwrap();
    assert_eq!(brief(&repo), [(paths[0].into(), Some('A'), None), (paths[1].into(), None, Some('?'))]);
    assert_eq!(diff::worktree(&repo, paths[0], true, false).unwrap().hunks[0].lines[0].text, paths[0]);
    status::unstage(&repo, &[paths[0].into()]).unwrap();
    assert_eq!(brief(&repo), expected);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn unstage_before_first_commit_and_binary() {
    let dir = init("unborn");
    let repo = Repo::open(&dir).unwrap();
    std::fs::write(dir.join("b.bin"), [0u8, 1, 2]).unwrap();
    assert!(diff::worktree(&repo, "b.bin", false, true).unwrap().binary);
    status::stage(&repo, &["b.bin".into()]).unwrap();
    assert_eq!(brief(&repo), [("b.bin".into(), Some('A'), None)]);
    assert!(diff::worktree(&repo, "b.bin", true, false).unwrap().binary);
    status::unstage(&repo, &["b.bin".into()]).unwrap();
    assert_eq!(brief(&repo), [("b.bin".into(), None, Some('?'))]);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn oversized_diffs_are_bounded_and_cannot_apply_stale_lines() {
    let dir = init("large-diff");
    let repo = Repo::open(&dir).unwrap();
    let large = "a line of text\n".repeat(400_000);
    std::fs::write(dir.join("large.txt"), &large).unwrap();
    assert!(diff::worktree(&repo, "large.txt", false, true).unwrap().too_large);
    status::stage(&repo, &["large.txt".into()]).unwrap();
    assert!(diff::worktree(&repo, "large.txt", true, false).unwrap().too_large);
    status::commit(&repo, "large file", false).unwrap();
    let id = refs::list(&repo).unwrap().head_id.unwrap();
    assert!(diff::commit(&repo, &id, "large.txt").unwrap().too_large);
    std::fs::write(dir.join("large.txt"), large.replace('a', "b")).unwrap();
    assert!(diff::worktree(&repo, "large.txt", false, false).unwrap().too_large);
    assert!(diff::apply_lines(&repo, "large.txt", false, 0, "@@ -1 +1 @@", &[0]).is_err());
    assert_eq!(index_content(&dir, "large.txt"), large);
    assert!(diff::commit(&repo, &"a".repeat(40), "large.txt").is_err());
    std::fs::remove_dir_all(&dir).unwrap();
}
