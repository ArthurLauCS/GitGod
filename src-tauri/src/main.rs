// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use engine::detail::{self, Detail};
use engine::graph::{Graph, Row};
use engine::refs::{self, Refs, Stash, Worktree};
use engine::{Repo, Result};
use std::sync::{Arc, RwLock};

/// 打开仓库时先加载这么多提交出首屏，完整历史由 load_graph 在后台补齐
const FIRST_PAGE: usize = 2000;

#[derive(Clone)]
struct Session {
    repo: Arc<Repo>,
    graph: Arc<Graph>,
}

type State<'a> = tauri::State<'a, RwLock<Option<Session>>>;

fn session(state: &State) -> Result<Session> {
    state.read().unwrap().clone().ok_or_else(|| "未打开仓库".to_owned())
}

/// 命令行传入的仓库路径：`gitgod <路径>`
#[tauri::command]
fn initial_repo() -> Option<String> {
    std::env::args().nth(1)
}

/// 返回 (工作区路径, 已加载的提交数)
#[tauri::command(async)]
fn open_repo(state: State, path: String) -> Result<(String, usize)> {
    let repo = Arc::new(Repo::open(path.as_ref())?);
    let graph = Arc::new(Graph::load(&repo, FIRST_PAGE)?);
    let out = (repo.path.display().to_string(), graph.len());
    *state.write().unwrap() = Some(Session { repo, graph });
    Ok(out)
}

/// 加载提交图：`full` 为 false 时只取首屏用的一小批。返回提交数。
#[tauri::command(async)]
fn load_graph(state: State, full: bool) -> Result<usize> {
    let repo = session(&state)?.repo;
    let graph = Arc::new(Graph::load(&repo, if full { usize::MAX } else { FIRST_PAGE })?);
    let len = graph.len();
    // 加载期间用户可能已经切到别的仓库
    if let Some(s) = state.write().unwrap().as_mut().filter(|s| Arc::ptr_eq(&s.repo, &repo)) {
        s.graph = graph;
    }
    Ok(len)
}

#[tauri::command(async)]
fn rows(state: State, start: usize, count: usize) -> Result<Vec<Row>> {
    let s = session(&state)?;
    s.graph.rows(&s.repo, start, count)
}

#[tauri::command(async)]
fn row_of(state: State, id: String) -> Result<Option<u32>> {
    Ok(session(&state)?.graph.row_of(&id))
}

#[tauri::command(async)]
fn refs(state: State) -> Result<Refs> {
    refs::list(&session(&state)?.repo)
}

#[tauri::command(async)]
fn stashes(state: State) -> Result<Vec<Stash>> {
    refs::stashes(&session(&state)?.repo)
}

#[tauri::command(async)]
fn worktrees(state: State) -> Result<Vec<Worktree>> {
    refs::worktrees(&session(&state)?.repo)
}

#[tauri::command(async)]
fn detail(state: State, id: String) -> Result<Detail> {
    detail::detail(&session(&state)?.repo, &id)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(RwLock::new(None::<Session>))
        .invoke_handler(tauri::generate_handler![initial_repo, open_repo, load_graph, rows, row_of, refs, stashes, worktrees, detail])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
