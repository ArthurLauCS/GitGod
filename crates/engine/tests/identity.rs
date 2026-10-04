mod common;

use engine::{identity, status, Repo};

#[test]
fn identity_is_effective_and_edits_stay_in_the_repository() {
    // 本测试独立运行在一个集成测试进程中，不读取或写入开发者的全局配置。
    let dir = common::init("identity");
    let global = dir.join("global-config");
    let config = "[user]\n name = Global User\n email = global@example.com\n";
    std::fs::write(&global, config).unwrap();
    std::env::set_var("GIT_CONFIG_GLOBAL", &global);
    std::env::set_var("GIT_CONFIG_NOSYSTEM", "1");
    for key in ["GIT_AUTHOR_NAME", "GIT_AUTHOR_EMAIL", "GIT_COMMITTER_NAME", "GIT_COMMITTER_EMAIL"] {
        std::env::remove_var(key);
    }
    common::git(&dir, 0, &["config", "--unset", "user.name"]);
    common::git(&dir, 0, &["config", "--unset", "user.email"]);
    let repo = Repo::open(&dir).unwrap();
    let inherited = identity::read(&repo).unwrap();
    assert_eq!(inherited.author.as_deref(), Some("Global User <global@example.com>"));

    let changed = identity::set(&repo, " 林同学 ", " lin@example.com ").unwrap();
    assert_eq!(changed.name, "林同学");
    assert_eq!(changed.email, "lin@example.com");
    assert_eq!(changed.author.as_deref(), Some("林同学 <lin@example.com>"));
    assert_eq!(changed.author, changed.committer);
    assert_eq!(std::fs::read_to_string(&global).unwrap(), config);
    assert!(identity::set(&repo, "Wrong", "bad\nmail@example.com").is_err());
    assert_eq!(identity::read(&repo).unwrap().name, "林同学");

    std::fs::write(dir.join("file.txt"), "content\n").unwrap();
    status::stage(&repo, &["file.txt".into()]).unwrap();
    status::commit(&repo, "verify identity", false).unwrap();
    let out = std::process::Command::new("git").arg("-C").arg(&dir)
        .args(["log", "-1", "--format=%an <%ae>"]).output().unwrap();
    assert_eq!(String::from_utf8(out.stdout).unwrap().trim(), "林同学 <lin@example.com>");

    std::env::set_var("GIT_AUTHOR_NAME", "Override");
    std::env::set_var("GIT_AUTHOR_EMAIL", "override@example.com");
    let overridden = identity::set(&repo, "Local User", "local@example.com").unwrap();
    assert_eq!(overridden.author.as_deref(), Some("Override <override@example.com>"));
    assert_eq!(overridden.committer.as_deref(), Some("Local User <local@example.com>"));
    std::fs::remove_dir_all(&dir).unwrap();
}
