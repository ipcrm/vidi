//! Native application menu.
//!
//! Every actionable item carries a stable string id. When clicked (or
//! activated by accelerator), the Tauri menu handler emits a
//! `menu://action` event with that id as payload, and the frontend
//! dispatches to the matching handler.
//!
//! Keeping the single source of truth in Rust has two benefits:
//!
//! 1. macOS menu accelerators intercept keystrokes before they reach the
//!    webview, which avoids the "JS handler and menu both fire" problem
//!    we'd otherwise get on some platforms.
//! 2. The menu structure is declarative in one place.

use tauri::menu::{
    AboutMetadataBuilder, Menu, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder,
};
use tauri::{AppHandle, Runtime};

/// Build the full application menu. Call from `setup`.
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let about_meta = AboutMetadataBuilder::new()
        .name(Some("Vidi"))
        .version(Some(env!("CARGO_PKG_VERSION")))
        .copyright(Some("© 2026 Vidi contributors · MIT"))
        .website(Some("https://ipcrm.github.io/vidi"))
        .build();

    let app_menu = SubmenuBuilder::new(app, "Vidi")
        .item(&PredefinedMenuItem::about(
            app,
            Some("About Vidi"),
            Some(about_meta),
        )?)
        .separator()
        .item(&PredefinedMenuItem::services(app, None)?)
        .separator()
        .item(&PredefinedMenuItem::hide(app, Some("Hide Vidi"))?)
        .item(&PredefinedMenuItem::hide_others(app, None)?)
        .item(&PredefinedMenuItem::show_all(app, None)?)
        .separator()
        .item(&PredefinedMenuItem::quit(app, Some("Quit Vidi"))?)
        .build()?;

    let file_menu = SubmenuBuilder::new(app, "File")
        .item(
            &MenuItemBuilder::new("Open File…")
                .id("open_file")
                .accelerator("CmdOrCtrl+Shift+O")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("Open Folder…")
                .id("open_folder")
                .accelerator("CmdOrCtrl+O")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("Recents")
                .id("open_recents")
                .build(app)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::new("New Tab")
                .id("new_tab")
                .accelerator("CmdOrCtrl+T")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("Close Tab")
                .id("close_tab")
                .accelerator("CmdOrCtrl+W")
                .build(app)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::new("Bookmark This Document")
                .id("bookmark")
                .accelerator("CmdOrCtrl+D")
                .build(app)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::new("Print…")
                .id("print")
                .accelerator("CmdOrCtrl+P")
                .build(app)?,
        )
        .build()?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .item(&PredefinedMenuItem::undo(app, None)?)
        .item(&PredefinedMenuItem::redo(app, None)?)
        .separator()
        .item(&PredefinedMenuItem::cut(app, None)?)
        .item(&PredefinedMenuItem::copy(app, None)?)
        .item(&PredefinedMenuItem::paste(app, None)?)
        .item(&PredefinedMenuItem::select_all(app, None)?)
        .build()?;

    let go_menu = SubmenuBuilder::new(app, "Go")
        .item(
            &MenuItemBuilder::new("Back")
                .id("go_back")
                .accelerator("CmdOrCtrl+[")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("Forward")
                .id("go_forward")
                .accelerator("CmdOrCtrl+]")
                .build(app)?,
        )
        .build()?;

    let view_menu = SubmenuBuilder::new(app, "View")
        .item(
            &MenuItemBuilder::new("Toggle Sidebar")
                .id("toggle_sidebar")
                .accelerator("CmdOrCtrl+\\")
                .build(app)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::new("Bookmarks")
                .id("panel_bookmarks")
                .accelerator("CmdOrCtrl+B")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("Recents")
                .id("panel_recents")
                .accelerator("CmdOrCtrl+Y")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("Settings")
                .id("panel_settings")
                .accelerator("CmdOrCtrl+,")
                .build(app)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::new("Search in Folder")
                .id("search_folder")
                .accelerator("CmdOrCtrl+Shift+F")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("Find in Document")
                .id("find_in_doc")
                .accelerator("CmdOrCtrl+F")
                .build(app)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::new("Toggle Theme")
                .id("toggle_theme")
                .build(app)?,
        )
        .build()?;

    let window_menu = SubmenuBuilder::new(app, "Window")
        .item(&PredefinedMenuItem::minimize(app, None)?)
        .item(&PredefinedMenuItem::close_window(app, None)?)
        .build()?;

    let help_menu = SubmenuBuilder::new(app, "Help")
        .item(
            &MenuItemBuilder::new("Keyboard Shortcuts")
                .id("help_shortcuts")
                .accelerator("CmdOrCtrl+/")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("Vidi Documentation")
                .id("help_docs")
                .build(app)?,
        )
        .build()?;

    MenuBuilder::new(app)
        .item(&app_menu)
        .item(&file_menu)
        .item(&edit_menu)
        .item(&go_menu)
        .item(&view_menu)
        .item(&window_menu)
        .item(&help_menu)
        .build()
}
