//! Drain any file paths that macOS / the OS handed us before the webview
//! was ready to receive them.
//!
//! See `RunEvent::Opened` handling in `lib.rs`. The frontend calls this
//! once at startup to pick up files from a cold launch via "Open With".

use crate::error::AppResult;
use crate::AppState;

#[tauri::command(rename_all = "camelCase")]
pub async fn take_pending_opens(state: tauri::State<'_, AppState>) -> AppResult<Vec<String>> {
    let mut pending = state
        .pending_opens
        .lock()
        .map_err(|e| crate::error::AppError::Internal(format!("lock: {e}")))?;
    let drained: Vec<String> = pending
        .drain(..)
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    Ok(drained)
}
