use crate::error::AppResult;
use crate::search::SearchHit;
use crate::AppState;

#[tauri::command(rename_all = "camelCase")]
pub async fn index_folder(state: tauri::State<'_, AppState>, path: String) -> AppResult<usize> {
    state.search.build_for(std::path::Path::new(&path))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn search_folder(
    state: tauri::State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> AppResult<Vec<SearchHit>> {
    state.search.search(&query, limit.unwrap_or(20))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn clear_index(state: tauri::State<'_, AppState>) -> AppResult<()> {
    state.search.clear();
    Ok(())
}
