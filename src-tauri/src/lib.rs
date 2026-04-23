pub mod error;
pub mod markdown;
pub mod menu;
pub mod model;
pub mod persistence;
pub mod search;
pub mod sources;

pub mod commands;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub store: Arc<persistence::Store>,
    pub watchers: Arc<commands::watcher::WatcherRegistry>,
    pub search: Arc<search::SearchState>,
    pub http: reqwest::Client,
    /// File paths handed to us by the OS ("Open With", drag-to-dock) while
    /// the webview wasn't yet ready to receive a `vidi://open-paths` event.
    /// Drained by the frontend on startup via `take_pending_opens`.
    pub pending_opens: Mutex<Vec<PathBuf>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init()
        .ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .menu(menu::build)
        .on_menu_event(|app, event| {
            // Forward menu activations to the frontend. A single event stream
            // `menu://action` keeps dispatch in one place on the JS side.
            use tauri::Emitter;
            let _ = app.emit("menu://action", event.id.as_ref());
        })
        .setup(|app| {
            let handle = app.handle();
            let data_dir = handle
                .path()
                .app_data_dir()
                .expect("app data dir available");
            std::fs::create_dir_all(&data_dir).ok();

            let store =
                Arc::new(persistence::Store::open(&data_dir).expect("open persistence store"));
            let watchers = Arc::new(commands::watcher::WatcherRegistry::new());
            let search = Arc::new(search::SearchState::new());
            let http = reqwest::Client::builder()
                .user_agent(concat!("Vidi/", env!("CARGO_PKG_VERSION")))
                .gzip(true)
                .brotli(true)
                .build()
                .expect("build http client");

            app.manage(AppState {
                store,
                watchers,
                search,
                http,
                pending_opens: Mutex::new(Vec::new()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::render::render_markdown,
            commands::render::render_markdown_inline,
            commands::folder::list_folder,
            commands::folder::read_file,
            commands::remote::fetch_remote,
            commands::render::resolve_link,
            commands::watcher::watch_folder,
            commands::watcher::unwatch_folder,
            commands::persistence::list_recents,
            commands::persistence::push_recent,
            commands::persistence::list_bookmarks,
            commands::persistence::add_bookmark,
            commands::persistence::remove_bookmark,
            commands::persistence::get_reading_position,
            commands::persistence::set_reading_position,
            commands::persistence::get_settings,
            commands::persistence::set_settings,
            commands::remote::open_external,
            commands::print::print_page,
            commands::search::index_folder,
            commands::search::search_folder,
            commands::search::clear_index,
            commands::open_files::take_pending_opens,
        ])
        .build(tauri::generate_context!())
        .expect("error while building vidi application")
        .run(|_app, _event| {
            // `RunEvent::Opened` is macOS-only ("Open With" / drag-onto-dock).
            // The variant isn't defined on other platforms, so the whole
            // handler is gated.
            #[cfg(target_os = "macos")]
            {
                use tauri::{Emitter, RunEvent};
                if let RunEvent::Opened { urls } = _event {
                    let paths: Vec<PathBuf> =
                        urls.iter().filter_map(|u| u.to_file_path().ok()).collect();
                    if paths.is_empty() {
                        return;
                    }
                    if let Some(state) = _app.try_state::<AppState>() {
                        if let Ok(mut pending) = state.pending_opens.lock() {
                            pending.extend(paths.iter().cloned());
                        }
                    }
                    let payload: Vec<String> = paths
                        .into_iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect();
                    let _ = _app.emit("vidi://open-paths", payload);
                }
            }
        });
}

// Re-exports used by the manager.
pub use tauri::Manager;
