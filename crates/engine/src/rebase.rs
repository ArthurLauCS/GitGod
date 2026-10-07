use crate::{decode, err, git_command, ops::Log, refs, tools::revision, Repo, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::PathBuf, process::Stdio, time::{SystemTime, UNIX_EPOCH}};

#[derive(Serialize, Deserialize, Clone)]
pub struct Step {
    pub id: String,
    pub subject: String,
    pub action: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct Plan { pub head: String, pub base: String, pub steps: Vec<Step> }

fn git_path(repo: &Repo, name: &str) -> Result<PathBuf> {
    Ok(repo.path.join(decode(&repo.git(&["rev-parse", "--git-path", name])?).trim()))
}

pub fn plan(repo: &Repo, base: &str) -> Result<Plan> {
    let base = revision(repo, base)?;
    let head = revision(repo, "HEAD")?;
    repo.git(&["merge-base", "--is-ancestor", &base, &head]).map_err(|_| "PR_REBASE_BASE")?;
    let range = format!("{base}..{head}");
    // A linear editor must not silently flatten merge commits.
    if !repo.git(&["rev-list", "--min-parents=2", &range])?.is_empty() { return Err("PR_REBASE_MERGES".into()); }
    let out = repo.git(&["log", "--reverse", "--format=%H%x00%s", &range])?;
    let steps = decode(&out).lines().filter_map(|line| {
        let (id, subject) = line.split_once('\0')?;
        Some(Step { id: id.into(), subject: subject.into(), action: "pick".into(), message: String::new() })
    }).collect();
    Ok(Plan { head, base, steps })
}

fn quote(path: &std::path::Path) -> String { format!("'{}'", path.to_string_lossy().replace('\\', "/").replace('\'', "'\"'\"'")) }

pub fn run(repo: &Repo, base: &str, head: &str, steps: &[Step]) -> Result<Log> {
    let original = plan(repo, base)?;
    if original.head != head { return Err("PR_FILE_CHANGED".into()); }
    if refs::list(repo)?.in_progress.is_some() || !repo.git(&["status", "--porcelain"])?.is_empty() { return Err("PR_REBASE_DIRTY".into()); }
    let expected: HashSet<_> = original.steps.iter().map(|s| s.id.as_str()).collect();
    let actual: HashSet<_> = steps.iter().map(|s| s.id.as_str()).collect();
    if steps.is_empty() || expected != actual || actual.len() != steps.len() { return Err("PR_REBASE_PLAN".into()); }
    let mut previous = false;
    for step in steps {
        match step.action.as_str() {
            "pick" | "edit" => previous = true,
            "reword" if !step.message.trim().is_empty() => previous = true,
            "squash" | "fixup" if previous => {},
            "drop" => {},
            _ => return Err("PR_REBASE_PLAN".into()),
        }
    }
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map_err(err)?.as_nanos();
    let temp = git_path(repo, &format!("pushright-rebase-{stamp}"))?;
    fs::create_dir(&temp).map_err(err)?;
    let result = (|| {
        fs::create_dir(temp.join("messages")).map_err(err)?;
        let mut todo = String::new();
        for step in steps {
            todo.push_str(&format!("{} {}\n", step.action, step.id));
            if !step.message.trim().is_empty() { fs::write(temp.join("messages").join(&step.id), &step.message).map_err(err)?; }
        }
        fs::write(temp.join("todo"), todo).map_err(err)?;
        // Git for Windows includes sh. Scripts are fixed; user messages are data files, never shell code.
        let editor = "state=$(git rev-parse --git-path rebase-merge) || exit 1\nlast=''\nwhile read -r action id rest; do last=$id; done < \"$state/done\"\nif test -f \"$state/pushright-messages/$last\"; then cat \"$state/pushright-messages/$last\" > \"$1\" || exit 1; fi\nexit 0\n";
        fs::write(temp.join("editor.sh"), editor).map_err(err)?;
        let sequence = format!(
            "test \"$(git rev-parse HEAD)\" = '{head}' || exit 1\nstate=$(git rev-parse --git-path rebase-merge) || exit 1\ncp {} \"$state/pushright-editor.sh\" && cp -R {} \"$state/pushright-messages\" && cat {} > \"$1\"\n",
            quote(&temp.join("editor.sh")), quote(&temp.join("messages")), quote(&temp.join("todo")),
        );
        fs::write(temp.join("sequence.sh"), sequence).map_err(err)?;
        let backup = format!("refs/heads/pushright-backup/{stamp}");
        repo.git(&["update-ref", &backup, head, ""])?;
        let out = git_command(&repo.path)
            .args(["-c", "rebase.updateRefs=false", "-c", "rebase.abbreviateCommands=false", "rebase", "-i", "--no-autosquash", "--no-autostash", "--keep-empty", &original.base])
            .env("GIT_SEQUENCE_EDITOR", format!("sh {}", quote(&temp.join("sequence.sh"))))
            .env("GIT_EDITOR", editor_command(repo)?)
            .stdin(Stdio::null()).output().map_err(err)?;
        Ok(Log { command: format!("git rebase -i {}", original.base), output: format!("{backup}\n{}{}", decode(&out.stdout), decode(&out.stderr)), ok: out.status.success() })
    })();
    // This unique directory belongs to this invocation; Git's state retains the editor during conflicts.
    let _ = fs::remove_dir_all(&temp);
    result
}

fn editor_command(repo: &Repo) -> Result<String> { Ok(format!("sh {}", quote(&git_path(repo, "rebase-merge/pushright-editor.sh")?))) }

pub fn continue_rebase(repo: &Repo) -> Result<Log> {
    let editor = git_path(repo, "rebase-merge/pushright-editor.sh")?;
    let out = git_command(&repo.path).args(["rebase", "--continue"])
        .env("GIT_EDITOR", if editor.is_file() { editor_command(repo)? } else { "true".into() })
        .stdin(Stdio::null()).output().map_err(err)?;
    Ok(Log { command: "git rebase --continue".into(), output: [decode(&out.stdout), decode(&out.stderr)].concat(), ok: out.status.success() })
}
