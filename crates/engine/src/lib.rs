pub mod detail;
pub mod graph;
pub mod refs;

use std::path::{Path, PathBuf};
use std::process::Command;

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
        let mut cmd = Command::new("git");
        cmd.arg("-C").arg(&self.path).args(args);
        #[cfg(windows)]
        std::os::windows::process::CommandExt::creation_flags(&mut cmd, 0x0800_0000); // CREATE_NO_WINDOW
        let out = cmd.output().map_err(err)?;
        if out.status.success() {
            Ok(out.stdout)
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_owned())
        }
    }
}
