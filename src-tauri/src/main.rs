// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use engine::detail::{self, Detail};
use engine::graph::{Graph, Row};
use engine::refs::{self, Refs, Stash, Worktree};
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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Tabs::default())
        .invoke_handler(tauri::generate_handler![initial_repos, open_repo, close_repo, load_graph, rows, row_of, refs, stashes, worktrees, detail])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
