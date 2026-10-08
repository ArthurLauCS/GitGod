//! 把引擎包成子进程供 VS Code 扩展使用：标准输入每行一个 `{id, cmd, args}`，
//! 标准输出每行一个 `{id, ok, value}`。命令名和参数与桌面版 src-tauri/src/main.rs 一致。

use engine::graph::Graph;
use base64::Engine;
use engine::{conflict, detail, diff, history, identity, local_files, ops, refs, status, tools, rebase, Repo, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

/// 打开仓库时先加载这么多提交出首屏，完整历史由 load_graph 在后台补齐
const FIRST_PAGE: usize = 2000;

#[derive(Clone)]
struct Session {
    repo: Arc<Repo>,
    graph: Arc<Graph>,
}

#[derive(Default)]
struct Tabs {
    next: AtomicU32,
    sessions: RwLock<HashMap<u32, Session>>,
}

#[derive(Deserialize)]
struct Request {
    id: u64,
    cmd: String,
    #[serde(default)]
    args: Value,
}

fn arg<T: DeserializeOwned>(args: &Value, key: &str) -> Result<T> {
    serde_json::from_value(args[key].clone()).map_err(|e| format!("{key}: {e}"))
}

fn out(value: impl Serialize) -> Result<Value> {
    serde_json::to_value(value).map_err(|e| e.to_string())
}

fn call(tabs: &Tabs, cmd: &str, a: &Value) -> Result<Value> {
    if cmd == "create_repo" { return out(tools::create(&arg::<String>(a, "path")?, arg::<Option<String>>(a, "url")?.as_deref())?); }
    if cmd == "open_repo" {
        let path: String = arg(a, "path")?;
        let repo = Arc::new(Repo::open(path.as_ref())?);
        let graph = Arc::new(Graph::load(&repo, FIRST_PAGE)?);
        let tab = tabs.next.fetch_add(1, Ordering::Relaxed);
        let reply = out((tab, repo.path.display().to_string(), graph.len()));
        tabs.sessions.write().unwrap().insert(tab, Session { repo, graph });
        return reply;
    }
    let tab: u32 = arg(a, "tab")?;
    if cmd == "close_repo" {
        tabs.sessions.write().unwrap().remove(&tab);
        return Ok(Value::Null);
    }
    let s = tabs.sessions.read().unwrap().get(&tab).cloned().ok_or("PR_TAB_CLOSED")?;
    let repo = &*s.repo;
    match cmd {
        "compare" => out(tools::compare(repo, &arg::<String>(a, "left")?, &arg::<String>(a, "right")?, arg(a, "commonBase")?)?),
        "diff_between" => out(diff::between(repo, &arg::<String>(a, "left")?, &arg::<String>(a, "right")?, &arg::<String>(a, "path")?, arg::<Option<String>>(a, "oldPath")?.as_deref())?),
        "diff_page" => out(diff::page(repo, &arg::<String>(a, "mode")?, &arg::<String>(a, "left")?, &arg::<String>(a, "right")?, &arg::<String>(a, "path")?, arg::<Option<String>>(a, "oldPath")?.as_deref(), arg(a, "skip")?)?),
        "remote_details" => out(tools::remotes(repo)?),
        "reflog" => out(tools::reflog(repo, arg(a, "skip")?, arg(a, "limit")?)?),
        "rebase_plan" => out(rebase::plan(repo, &arg::<String>(a, "base")?)?),
        "rebase_run" => out(rebase::run(repo, &arg::<String>(a, "base")?, &arg::<String>(a, "head")?, &arg::<Vec<rebase::Step>>(a, "steps")?)?),
        "load_graph" => {
            let scope: Option<String> = arg(a, "scope")?;
            let graph = Arc::new(Graph::load_scope(repo, if arg(a, "full")? { usize::MAX } else { FIRST_PAGE }, scope.as_deref().unwrap_or("all"))?);
            let len = graph.len();
            // 加载期间页签可能已经关闭
            if let Some(s) = tabs.sessions.write().unwrap().get_mut(&tab) {
                s.graph = graph;
            }
            out(len)
        }
        "rows" => out(s.graph.rows(repo, arg(a, "start")?, arg(a, "count")?)?),
        "row_of" => out(s.graph.row_of(&arg::<String>(a, "id")?)),
        "refs" => out(refs::list(repo)?),
        "stashes" => out(refs::stashes(repo)?),
        "worktrees" => out(refs::worktrees(repo)?),
        "remotes" => out(refs::remotes(repo)?),
        "tracking" => out(refs::tracking(repo)?),
        "detail" => out(detail::detail(repo, &arg::<String>(a, "id")?)?),
        "stash_detail" => out(detail::stash(repo, &arg::<String>(a, "name")?)?),
        "status" => out(status::status(repo)?),
        "stage" => out(status::stage(repo, &arg::<Vec<String>>(a, "paths")?)?),
        "unstage" => out(status::unstage(repo, &arg::<Vec<String>>(a, "paths")?)?),
        "commit" => out(status::commit(repo, &arg::<String>(a, "message")?, arg(a, "amend")?)?),
        "last_message" => out(status::last_message(repo)?),
        "commit_identity" => out(identity::read(repo)?),
        "set_commit_identity" => out(identity::set(repo, &arg::<String>(a, "name")?, &arg::<String>(a, "email")?)?),
        "diff_worktree" => out(diff::worktree(repo, &arg::<String>(a, "path")?, arg(a, "staged")?, arg(a, "untracked")?)?),
        "diff_commit" => out(diff::commit(repo, &arg::<String>(a, "id")?, &arg::<String>(a, "path")?)?),
        "stash_diff" => out(diff::stash(repo, &arg::<String>(a, "id")?, &arg::<String>(a, "path")?)?),
        "apply_lines" => out(diff::apply_lines(
            repo,
            &arg::<String>(a, "path")?,
            arg(a, "staged")?,
            arg(a, "hunk")?,
            &arg::<String>(a, "header")?,
            &arg::<Vec<usize>>(a, "lines")?,
        )?),
        "discard_lines" => out(diff::discard_lines(
            repo,
            &arg::<String>(a, "path")?,
            arg(a, "hunk")?,
            &arg::<String>(a, "header")?,
            &arg::<Vec<usize>>(a, "lines")?,
        )?),
        "op" => out(ops::run(repo, arg(a, "op")?)?),
        "conflict_read" => out(conflict::read(repo, &arg::<String>(a, "path")?)?),
        "conflict_resolve" => out(conflict::resolve(repo, &arg::<String>(a, "path")?, &arg::<Vec<conflict::Side>>(a, "choices")?)?),
        "conflict_take" => out(conflict::take(repo, &arg::<String>(a, "path")?, arg(a, "theirs")?)?),
        // 以下命令只有扩展使用
        "repositories" => out(local_files::repositories(repo)?),
        "ignored" => out(local_files::ignored(repo, &arg::<String>(a, "path")?)?),
        "is_tracked" => out(local_files::tracked(repo, &arg::<String>(a, "path")?)?),
        "ignore_file" => out(local_files::ignore_file(repo, &arg::<String>(a, "path")?, arg(a, "shared")?)?),
        "track_file" => out(local_files::track(repo, &arg::<String>(a, "path")?, arg(a, "track")?)?),
        "file_history" => out(history::file_page(repo, &arg::<String>(a, "path")?, arg(a, "limit")?, arg::<Option<usize>>(a, "skip")?.unwrap_or(0))?),
        "line_history" => out(history::lines(repo, &arg::<String>(a, "path")?, arg(a, "start")?, arg(a, "end")?, arg(a, "limit")?)?),
        "line_history_page" => out(history::lines_page(repo, &arg::<String>(a, "path")?, arg(a, "start")?, arg(a, "end")?, arg(a, "limit")?, arg(a, "skip")?, arg::<Option<String>>(a, "contents")?.as_deref())?),
        "search" => out(history::search_page(repo, arg(a, "kind")?, &arg::<String>(a, "query")?, arg(a, "limit")?, arg::<Option<usize>>(a, "skip")?.unwrap_or(0))?),
        "show" => out(history::show_limit(repo, &arg::<String>(a, "rev")?, &arg::<String>(a, "path")?, arg::<Option<usize>>(a, "limit")?.unwrap_or(4 << 20))?),
        "show_binary" => out(history::blob(repo, &arg::<String>(a, "rev")?, &arg::<String>(a, "path")?, 32 << 20)?.map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes))),
        _ => Err(format!("unknown command: {cmd}")),
    }
}

fn main() {
    // 扩展靠监听 .git 的变化来刷新；不让只读命令（status）顺手改写索引，否则每次刷新又触发一次变化
    std::env::set_var("GIT_OPTIONAL_LOCKS", "0");
    let tabs = Arc::new(Tabs::default());
    // 标准输入关闭（扩展主机退出）时循环结束，进程随之退出
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        let tabs = tabs.clone();
        // ponytail: 每个请求一个线程；实测往返成为瓶颈时换线程池
        std::thread::spawn(move || {
            let reply = match serde_json::from_str::<Request>(&line) {
                Ok(r) => match call(&tabs, &r.cmd, &r.args) {
                    Ok(value) => json!({ "id": r.id, "ok": true, "value": value }),
                    Err(e) => json!({ "id": r.id, "ok": false, "value": e }),
                },
                Err(e) => json!({ "id": null, "ok": false, "value": e.to_string() }),
            };
            let mut stdout = std::io::stdout().lock();
            let _ = writeln!(stdout, "{reply}");
            let _ = stdout.flush();
        });
    }
}
