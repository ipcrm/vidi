use crate::error::AppResult;
use crate::model::{Bookmark, ReadingPosition, RecentFile, Settings, Source};
use crate::AppState;

#[tauri::command(rename_all = "camelCase")]
pub async fn list_recents(state: tauri::State<'_, AppState>) -> AppResult<Vec<RecentFile>> {
    Ok(state.store.recents())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn push_recent(
    state: tauri::State<'_, AppState>,
    source: Source,
    title: String,
) -> AppResult<()> {
    state.store.push_recent(RecentFile {
        source,
        title,
        opened_at: now_secs(),
    })
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_bookmarks(state: tauri::State<'_, AppState>) -> AppResult<Vec<Bookmark>> {
    Ok(state.store.bookmarks())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_bookmark(
    state: tauri::State<'_, AppState>,
    source: Source,
    label: String,
    anchor: Option<String>,
) -> AppResult<Bookmark> {
    let b = Bookmark {
        id: uuid_like(),
        source,
        label,
        anchor,
        created_at: now_secs(),
    };
    state.store.add_bookmark(b)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn remove_bookmark(state: tauri::State<'_, AppState>, id: String) -> AppResult<()> {
    state.store.remove_bookmark(&id)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_reading_position(
    state: tauri::State<'_, AppState>,
    source: Source,
) -> AppResult<Option<ReadingPosition>> {
    Ok(state.store.reading_position(&source))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_reading_position(
    state: tauri::State<'_, AppState>,
    source: Source,
    position: ReadingPosition,
) -> AppResult<()> {
    state.store.set_reading_position(source, position)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_settings(state: tauri::State<'_, AppState>) -> AppResult<Settings> {
    Ok(state.store.settings())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_settings(state: tauri::State<'_, AppState>, settings: Settings) -> AppResult<()> {
    state.store.set_settings(settings)
}

fn now_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let rand = std::process::id() as u128;
    format!("{:016x}{:08x}", nanos, rand)
}
