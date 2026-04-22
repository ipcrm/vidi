use crate::error::{AppError, AppResult};
use crate::model::RemoteFetch;
use crate::sources::{github, remote};
use crate::AppState;
use tauri_plugin_opener::OpenerExt;

#[tauri::command(rename_all = "camelCase")]
pub async fn fetch_remote(
    state: tauri::State<'_, AppState>,
    url: String,
) -> AppResult<RemoteFetch> {
    let normalized = github::normalize(&url);
    remote::fetch(&state.http, &normalized.fetch_url).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn open_external(app: tauri::AppHandle, url: String) -> AppResult<()> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("mailto:"))
    {
        return Err(AppError::InvalidArgument(format!(
            "refusing to open non-http/mailto url: {trimmed}"
        )));
    }
    app.opener()
        .open_url(trimmed, None::<&str>)
        .map_err(|e| AppError::Internal(e.to_string()))
}
