use crate::error::AppResult;
use crate::model::FileTree;
use crate::sources::local;
use std::path::PathBuf;

#[tauri::command(rename_all = "camelCase")]
pub async fn list_folder(path: String) -> AppResult<FileTree> {
    local::walk_folder(&PathBuf::from(path))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn read_file(path: String) -> AppResult<String> {
    local::read_markdown_file(&PathBuf::from(path))
}
