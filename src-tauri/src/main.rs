// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use engine::conflict::{self, Conflict, Side};
use engine::detail::{self, Detail};
use engine::diff::{self, Diff};
use engine::status::{self, Entry};
use engine::graph::{Graph, Row};
use engine::identity::{self, Identity};
use engine::ops::{self, Log, Op};
use engine::refs::{self, Refs, Stash, Track, Worktree};
use engine::{Repo, Result};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

/// 打开仓库时先加载这么多提交出首屏，完整历史由 load_graph 在后台补齐
const FIRST_PAGE: usize = 2000;

#[derive(Clone)]
struct Session {
    repo: Arc<Repo>,
    graph: Arc<Graph>,
}

/// 每个页签一个仓库会话，页签 id 由后端分配
#[derive(Default)]
struct Tabs {
    next: AtomicU32,
    sessions: RwLock<HashMap<u32, Session>>,
}

type State<'a> = tauri::State<'a, Tabs>;

fn session(state: &State, tab: u32) -> Result<Session> {
    state.sessions.read().unwrap().get(&tab).cloned().ok_or_else(|| "页签已关闭".to_owned())
}

/// 命令行传入的仓库路径：`gitgod <路径>...`
#[tauri::command]
fn initial_repos() -> Vec<String> {
    std::env::args().skip(1).collect()
}

/// 返回 (页签 id, 工作区路径, 已加载的提交数)
#[tauri::command(async)]
fn open_repo(state: State, path: String) -> Result<(u32, String, usize)> {
    let repo = Arc::new(Repo::open(path.as_ref())?);
    let graph = Arc::new(Graph::load(&repo, FIRST_PAGE)?);
    let tab = state.next.fetch_add(1, Ordering::Relaxed);
    let out = (tab, repo.path.display().to_string(), graph.len());
    state.sessions.write().unwrap().insert(tab, Session { repo, graph });
    Ok(out)
}

#[tauri::command]
fn close_repo(state: State, tab: u32) {
    state.sessions.write().unwrap().remove(&tab);
}

/// 加载提交图：`full` 为 false 时只取首屏用的一小批。返回提交数。
#[tauri::command(async)]
fn load_graph(state: State, tab: u32, full: bool) -> Result<usize> {
    let repo = session(&state, tab)?.repo;
    let graph = Arc::new(Graph::load(&repo, if full { usize::MAX } else { FIRST_PAGE })?);
    let len = graph.len();
    // 加载期间页签可能已经关闭
    if let Some(s) = state.sessions.write().unwrap().get_mut(&tab) {
        s.graph = graph;
    }
    Ok(len)
}

#[tauri::command(async)]
fn rows(state: State, tab: u32, start: usize, count: usize) -> Result<Vec<Row>> {
    let s = session(&state, tab)?;
    s.graph.rows(&s.repo, start, count)
}

#[tauri::command(async)]
fn row_of(state: State, tab: u32, id: String) -> Result<Option<u32>> {
    Ok(session(&state, tab)?.graph.row_of(&id))
}

#[tauri::command(async)]
fn refs(state: State, tab: u32) -> Result<Refs> {
    refs::list(&session(&state, tab)?.repo)
}

#[tauri::command(async)]
fn stashes(state: State, tab: u32) -> Result<Vec<Stash>> {
    refs::stashes(&session(&state, tab)?.repo)
}

#[tauri::command(async)]
fn worktrees(state: State, tab: u32) -> Result<Vec<Worktree>> {
    refs::worktrees(&session(&state, tab)?.repo)
}

#[tauri::command(async)]
fn detail(state: State, tab: u32, id: String) -> Result<Detail> {
    detail::detail(&session(&state, tab)?.repo, &id)
}

#[tauri::command(async)]
fn status(state: State, tab: u32) -> Result<Vec<Entry>> {
    status::status(&session(&state, tab)?.repo)
}

#[tauri::command(async)]
fn stage(state: State, tab: u32, paths: Vec<String>) -> Result<()> {
    status::stage(&session(&state, tab)?.repo, &paths)
}

#[tauri::command(async)]
fn unstage(state: State, tab: u32, paths: Vec<String>) -> Result<()> {
    status::unstage(&session(&state, tab)?.repo, &paths)
}

#[tauri::command(async)]
fn commit(state: State, tab: u32, message: String, amend: bool) -> Result<()> {
    status::commit(&session(&state, tab)?.repo, &message, amend)
}

#[tauri::command(async)]
fn last_message(state: State, tab: u32) -> Result<String> {
    status::last_message(&session(&state, tab)?.repo)
}

#[tauri::command(async)]
fn commit_identity(state: State, tab: u32) -> Result<Identity> {
    identity::read(&session(&state, tab)?.repo)
}

#[tauri::command(async)]
fn set_commit_identity(state: State, tab: u32, name: String, email: String) -> Result<Identity> {
    identity::set(&session(&state, tab)?.repo, &name, &email)
}

#[tauri::command(async)]
fn diff_worktree(state: State, tab: u32, path: String, staged: bool, untracked: bool) -> Result<Diff> {
    diff::worktree(&session(&state, tab)?.repo, &path, staged, untracked)
}

#[tauri::command(async)]
fn diff_commit(state: State, tab: u32, id: String, path: String) -> Result<Diff> {
    diff::commit(&session(&state, tab)?.repo, &id, &path)
}

#[tauri::command(async)]
fn apply_lines(state: State, tab: u32, path: String, staged: bool, hunk: usize, header: String, lines: Vec<usize>) -> Result<()> {
    diff::apply_lines(&session(&state, tab)?.repo, &path, staged, hunk, &header, &lines)
}

#[tauri::command(async)]
fn op(state: State, tab: u32, op: Op) -> Result<Log> {
    ops::run(&session(&state, tab)?.repo, op)
}

#[tauri::command(async)]
fn remotes(state: State, tab: u32) -> Result<Vec<String>> {
    refs::remotes(&session(&state, tab)?.repo)
}

#[tauri::command(async)]
fn tracking(state: State, tab: u32) -> Result<Vec<Track>> {
    refs::tracking(&session(&state, tab)?.repo)
}

#[tauri::command(async)]
fn discard_lines(state: State, tab: u32, path: String, hunk: usize, header: String, lines: Vec<usize>) -> Result<()> {
    diff::discard_lines(&session(&state, tab)?.repo, &path, hunk, &header, &lines)
}

#[tauri::command(async)]
fn conflict_read(state: State, tab: u32, path: String) -> Result<Conflict> {
    conflict::read(&session(&state, tab)?.repo, &path)
}

#[tauri::command(async)]
fn conflict_resolve(state: State, tab: u32, path: String, choices: Vec<Side>) -> Result<()> {
    conflict::resolve(&session(&state, tab)?.repo, &path, &choices)
}

#[tauri::command(async)]
fn conflict_take(state: State, tab: u32, path: String, theirs: bool) -> Result<()> {
    conflict::take(&session(&state, tab)?.repo, &path, theirs)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Tabs::default())
        .invoke_handler(tauri::generate_handler![
            initial_repos, open_repo, close_repo, load_graph, rows, row_of, refs, stashes, worktrees, detail, status, stage, unstage,
            commit, last_message, commit_identity, set_commit_identity, diff_worktree, diff_commit, apply_lines, op, remotes, tracking, discard_lines, conflict_read, conflict_resolve, conflict_take
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
