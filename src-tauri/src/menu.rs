use tauri::menu::{Menu, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Wry};

/// Меню приложения без нативных Undo/Redo.
///
/// Акселераторы Cmd+Z и Cmd+Shift+Z у стандартных пунктов меню macOS
/// перехватывают нажатие до WebView, поэтому горячие клавиши доски
/// (отмена и возврат хода) до неё не доходили. Пункты Cut/Copy/Paste
/// остаются — они нужны полям ввода.
pub fn build(app: &AppHandle<Wry>) -> tauri::Result<Menu<Wry>> {
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
    Menu::with_items(app, &[&app_menu, &edit_menu, &view_menu, &window_menu])
}
