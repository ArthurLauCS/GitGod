use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

// ponytail: 只支持 SHA-1 仓库，遇到 SHA-256 仓库时改成 32 字节或枚举
pub type Oid = [u8; 20];

pub struct Commit {
    pub id: Oid,
    pub parents: Vec<Oid>,
    pub time: i64,
    pub author: String,
    pub subject: String,
}

/// 流式读取 `git log`，每解析出一个提交回调一次。`args` 追加在 log 之后（如 `--all`、`-n`）。
pub fn walk(repo: &Path, args: &[&str], mut f: impl FnMut(Commit)) -> io::Result<()> {
    let mut child = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["log", "--format=%H%x00%P%x00%ct%x00%an%x00%s"])
        .args(args)
        .stdout(Stdio::piped())
        .spawn()?;
    let mut out = BufReader::with_capacity(1 << 16, child.stdout.take().unwrap());
    let mut line = Vec::new();
    while out.read_until(b'\n', &mut line)? > 0 {
        let mut fields = line.strip_suffix(b"\n").unwrap_or(&line).split(|&b| b == 0);
        let id = oid(fields.next().unwrap());
        let parents = fields.next().unwrap().split(|&b| b == b' ').filter(|p| !p.is_empty()).map(oid).collect();
        let time = std::str::from_utf8(fields.next().unwrap()).unwrap().parse().unwrap();
        let author = String::from_utf8_lossy(fields.next().unwrap()).into_owned();
        let subject = String::from_utf8_lossy(fields.next().unwrap()).into_owned();
        f(Commit { id, parents, time, author, subject });
        line.clear();
    }
    child.wait()?;
    Ok(())
}

pub fn load(repo: &Path, args: &[&str]) -> io::Result<Vec<Commit>> {
    let mut commits = Vec::new();
    walk(repo, args, |c| commits.push(c))?;
    Ok(commits)
}

fn oid(hex: &[u8]) -> Oid {
    let nib = |c: u8| if c <= b'9' { c - b'0' } else { c - b'a' + 10 };
    let mut id = [0; 20];
    for (b, h) in id.iter_mut().zip(hex.chunks_exact(2)) {
        *b = nib(h[0]) << 4 | nib(h[1]);
    }
    id
}
