pub mod conflict;
pub mod detail;
pub mod diff;
pub mod graph;
pub mod ops;
pub mod refs;
pub mod status;

use std::path::{Path, PathBuf};
use std::io::Write;
use std::process::{Command, Stdio};

pub type Result<T> = std::result::Result<T, String>;

pub(crate) fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// 读走 gix（进程内），写和 status/diff 走 git CLI，依据见 M0 基准。
pub struct Repo {
    /// 工作区根目录；裸仓库时为 .git 目录
    pub path: PathBuf,
    gix: gix::ThreadSafeRepository,
}

impl Repo {
    pub fn open(path: &Path) -> Result<Repo> {
        let gix = gix::discover(path).map_err(err)?;
        let path = gix.workdir().unwrap_or(gix.git_dir()).to_owned();
        Ok(Repo { path, gix: gix.into_sync() })
    }

    pub(crate) fn gix(&self) -> gix::Repository {
        self.gix.to_thread_local()
    }

    pub(crate) fn git(&self, args: &[&str]) -> Result<Vec<u8>> {
        self.git_in(args, &[])
    }

    /// 同 `git`，并把 `stdin` 写给子进程（提交信息、补丁、路径列表）。
    pub(crate) fn git_in(&self, args: &[&str], stdin: &[u8]) -> Result<Vec<u8>> {
        git_at(&self.path, args, stdin)
    }
}

/// 在任意目录跑 git（其他工作树不在 `Repo::path` 下）。
pub(crate) fn git_at(dir: &Path, args: &[&str], stdin: &[u8]) -> Result<Vec<u8>> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(dir).args(args);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(&mut cmd, 0x0800_0000); // CREATE_NO_WINDOW
    let mut child = cmd.spawn().map_err(err)?;
    // 读完 stdin 前就退出的命令会让写入失败，以它的退出状态为准
    let _ = child.stdin.take().unwrap().write_all(stdin);
    let out = child.wait_with_output().map_err(err)?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_owned())
    }
}

impl Repo {
    /// 跑一条写操作并记录命令和输出，不论成败都返回日志。
    pub(crate) fn git_log(&self, args: &[String]) -> ops::Log {
        let mut cmd = Command::new("git");
        // 没有终端可以输入用户名密码，凭据交给 credential helper，否则直接失败而不是挂住
        cmd.arg("-C").arg(&self.path).args(args).env("GIT_TERMINAL_PROMPT", "0").stdin(Stdio::null());
        #[cfg(windows)]
        std::os::windows::process::CommandExt::creation_flags(&mut cmd, 0x0800_0000); // CREATE_NO_WINDOW
        let command = format!("git {}", args.join(" "));
        match cmd.output() {
            Ok(out) => {
                let text = [decode(&out.stdout), decode(&out.stderr)].concat();
                ops::Log { command, output: text.trim().to_owned(), ok: out.status.success() }
            }
            Err(e) => ops::Log { command, output: e.to_string(), ok: false },
        }
    }
}

/// 文件内容和 diff 的解码：合法 UTF-8 原样用，否则按 GB18030（GBK 的超集）解。
pub(crate) fn decode(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_owned(),
        Err(_) => encoding_rs::GB18030.decode(bytes).0.into_owned(),
    }
}
