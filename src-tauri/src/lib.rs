pub mod error;
pub mod markdown;
pub mod model;
pub mod persistence;
pub mod search;
pub mod sources;

pub mod commands;

use std::sync::Arc;

pub struct AppState {
    pub store: Arc<persistence::Store>,
    pub watchers: Arc<commands::watcher::WatcherRegistry>,
    pub search: Arc<search::SearchState>,
    pub http: reqwest::Client,
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
                .user_agent(concat!("Visum/", env!("CARGO_PKG_VERSION")))
                .gzip(true)
                .brotli(true)
                .build()
                .expect("build http client");

            app.manage(AppState {
                store,
                watchers,
                search,
                http,
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
            commands::search::index_folder,
            commands::search::search_folder,
            commands::search::clear_index,
        ])
        .run(tauri::generate_context!())
        .expect("error while running visum application");
}

// Re-exports used by the manager.
pub use tauri::Manager;
