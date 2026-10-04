use crate::{Repo, Result};
use serde::Serialize;

#[derive(Serialize)]
pub struct Identity {
    /// 用于编辑的有效 user 配置（含全局配置和 include）。
    pub name: String,
    pub email: String,
    /// Git 实际使用的身份，包含 author/committer 配置和环境变量覆盖。
    pub author: Option<String>,
    pub committer: Option<String>,
}

pub fn read(repo: &Repo) -> Result<Identity> {
    let config = |key| {
        repo.git(&["config", "--default", "", "--get", key])
            .map(|out| String::from_utf8_lossy(&out).trim().to_owned())
    };
    let actual = |key| {
        let out = repo.git(&["var", key]).ok()?;
        let out = String::from_utf8_lossy(&out);
        let end = out.rfind('>')?;
        Some(out[..=end].to_owned())
    };
    Ok(Identity {
        name: config("user.name")?,
        email: config("user.email")?,
        author: actual("GIT_AUTHOR_IDENT"),
        committer: actual("GIT_COMMITTER_IDENT"),
    })
}

/// 只写仓库配置；关联的工作树共享它，不改全局配置或已有提交。
pub fn set(repo: &Repo, name: &str, email: &str) -> Result<Identity> {
    let (name, email) = (name.trim(), email.trim());
    if name.is_empty() || email.is_empty()
        || name.chars().any(|c| c.is_control() || c == '<' || c == '>')
        || email.chars().any(|c| c.is_whitespace() || c.is_control() || c == '<' || c == '>')
    {
        return Err("请输入非空的提交姓名和邮箱，不能包含换行或尖括号，邮箱不能包含空格".into());
    }
    repo.git(&["config", "--local", "--replace-all", "--", "user.name", name])?;
    repo.git(&["config", "--local", "--replace-all", "--", "user.email", email])?;
    read(repo)
}
