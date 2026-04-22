//! Folder watcher — debounced notifications emitted to the frontend as
//! `folder://changed` events.

use crate::error::{AppError, AppResult};
use crate::model::WatchHandle;
use crate::AppState;
use notify::{RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::Emitter;

type DebouncerT = Debouncer<notify::RecommendedWatcher, notify_debouncer_full::FileIdMap>;

pub struct WatcherRegistry {
    next_id: AtomicU64,
    watchers: Mutex<HashMap<u64, DebouncerT>>,
}

impl WatcherRegistry {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            watchers: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for WatcherRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// One coalesced notification per debouncer tick. `paths` is the deduplicated
/// set of paths that changed within the window (limited to a sane size to
/// keep the IPC payload small during mass-change events like a git pull).
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ChangeEvent {
    /// Deduplicated changed paths (capped to `MAX_PATHS_PER_EVENT`).
    paths: Vec<PathBuf>,
    /// True when we truncated — the frontend should treat the tree as wholly dirty.
    truncated: bool,
    /// Total number of distinct paths before truncation.
    total: usize,
}

const MAX_PATHS_PER_EVENT: usize = 64;

#[tauri::command(rename_all = "camelCase")]
pub async fn watch_folder(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    path: String,
) -> AppResult<WatchHandle> {
    let root = PathBuf::from(path);
    if !root.is_dir() {
        return Err(AppError::InvalidArgument(format!(
            "not a directory: {}",
            root.display()
        )));
    }

    let app_handle = app.clone();
    let mut debouncer = new_debouncer(
        Duration::from_millis(250),
        None,
        move |res: DebounceEventResult| match res {
            Ok(events) => {
                // Coalesce all paths into a deduplicated set — a branch switch
                // or `git pull` touching hundreds of files would otherwise
                // emit hundreds of IPC events, each triggering a frontend
                // tree-refresh.
                let mut unique: HashSet<PathBuf> = HashSet::new();
                for ev in &events {
                    for p in &ev.paths {
                        unique.insert(p.clone());
                    }
                }
                if unique.is_empty() {
                    return;
                }
                let total = unique.len();
                let truncated = total > MAX_PATHS_PER_EVENT;
                let paths: Vec<PathBuf> = if truncated {
                    unique.into_iter().take(MAX_PATHS_PER_EVENT).collect()
                } else {
                    unique.into_iter().collect()
                };
                let payload = ChangeEvent {
                    paths,
                    truncated,
                    total,
                };
                let _ = app_handle.emit("folder://changed", payload);
            }
            Err(errs) => {
                for e in errs {
                    tracing::warn!("watcher error: {e:?}");
                }
            }
        },
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;

    debouncer
        .watcher()
        .watch(Path::new(&root), RecursiveMode::Recursive)
        .map_err(|e: notify::Error| AppError::Internal(e.to_string()))?;

    let id = state.watchers.next_id.fetch_add(1, Ordering::SeqCst);
    state
        .watchers
        .watchers
        .lock()
        .unwrap()
        .insert(id, debouncer);

    Ok(WatchHandle { id })
}

#[tauri::command(rename_all = "camelCase")]
pub async fn unwatch_folder(
    state: tauri::State<'_, AppState>,
    handle: WatchHandle,
) -> AppResult<()> {
    let mut w = state.watchers.watchers.lock().unwrap();
    w.remove(&handle.id);
    Ok(())
}
