use tauri::menu::{Menu, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Wry};

/// Application menu without the native Undo/Redo items.
///
/// The Cmd+Z and Cmd+Shift+Z accelerators on the standard macOS menu items
/// intercept the keystroke before it reaches the WebView, so the board's
/// shortcuts (undo and redo a move) never arrived. Cut/Copy/Paste are kept
/// because text inputs need them.
///
/// Only macOS gets a native menu: on Windows and Linux the WebView already
/// handles the standard editing shortcuts, and the macOS-only items
/// (Services, Hide, Quit) are unsupported there.
pub fn build(app: &AppHandle<Wry>) -> tauri::Result<Option<Menu<Wry>>> {
    if !cfg!(target_os = "macos") {
        return Ok(None);
    }
    let name = app.package_info().name.clone();
    let app_menu = Submenu::with_items(
        app,
        name,
        true,
        &[
            &PredefinedMenuItem::about(app, None, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::services(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, None)?,
            &PredefinedMenuItem::hide_others(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, None)?,
        ],
    )?;
    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;
    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[&PredefinedMenuItem::fullscreen(app, None)?],
    )?;
    let window_menu = Submenu::with_items(
        app,
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::close_window(app, None)?,
        ],
    )?;
    Ok(Some(Menu::with_items(
        app,
        &[&app_menu, &edit_menu, &view_menu, &window_menu],
    )?))
}
