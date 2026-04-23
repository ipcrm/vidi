//! Print command — bridges to the native `WebviewWindow::print` so the
//! JS side can trigger the OS print dialog.
//!
//! Tauri v2's `@tauri-apps/api` (as of 2.10.x) doesn't expose `print()` on
//! `WebviewWindow` to the JS side. We invoke it from Rust instead.

use crate::error::{AppError, AppResult};

#[tauri::command(rename_all = "camelCase")]
pub async fn print_page(window: tauri::WebviewWindow) -> AppResult<()> {
    window
        .print()
        .map_err(|e| AppError::Internal(format!("print: {e}")))
}
