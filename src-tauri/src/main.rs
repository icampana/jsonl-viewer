// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use commands::{file_parser, search, export, network};

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::Emitter;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let handle = app.handle();

            // App configurations
            #[cfg(target_os = "macos")]
            let app_name = "JSONL Viewer";
            #[cfg(not(target_os = "macos"))]
            let app_name = "File";

            // App Menu
            let app_menu = Submenu::with_items(handle, app_name, true, &[
                &PredefinedMenuItem::about(handle, None, None)?,
                &PredefinedMenuItem::separator(handle)?,
                &PredefinedMenuItem::services(handle, None)?,
                &PredefinedMenuItem::hide(handle, None)?,
                &PredefinedMenuItem::hide_others(handle, None)?,
                &PredefinedMenuItem::show_all(handle, None)?,
                &PredefinedMenuItem::separator(handle)?,
                &PredefinedMenuItem::quit(handle, None)?,
            ])?;

            // File Menu
            let open_i = MenuItem::with_id(handle, "open_file", "Open File", true, Some("CmdOrCtrl+O"))?;
            let open_url_i = MenuItem::with_id(handle, "open_url", "Open from URL...", true, Some("CmdOrCtrl+Shift+O"))?;
            let export_i = MenuItem::with_id(handle, "export_file", "Export...", true, Some("CmdOrCtrl+E"))?;
            let close_i = MenuItem::with_id(handle, "close_file", "Close File", true, Some("CmdOrCtrl+W"))?;

            let file_menu = Submenu::with_items(handle, "File", true, &[
                &open_i,
                &open_url_i,
                &export_i,
                &PredefinedMenuItem::separator(handle)?,
                &close_i,
            ])?;

            // Edit Menu
            let edit_menu = Submenu::with_items(handle, "Edit", true, &[
                &PredefinedMenuItem::undo(handle, None)?,
                &PredefinedMenuItem::redo(handle, None)?,
                &PredefinedMenuItem::separator(handle)?,
                &PredefinedMenuItem::cut(handle, None)?,
                &PredefinedMenuItem::copy(handle, None)?,
                &PredefinedMenuItem::paste(handle, None)?,
                &PredefinedMenuItem::select_all(handle, None)?,
            ])?;

            // View Menu
            let view_menu = Submenu::with_items(handle, "View", true, &[
                &PredefinedMenuItem::fullscreen(handle, None)?,
            ])?;

            // Window Menu
            let window_menu = Submenu::with_items(handle, "Window", true, &[
                &PredefinedMenuItem::minimize(handle, None)?,
                &PredefinedMenuItem::maximize(handle, None)?,
                &PredefinedMenuItem::separator(handle)?,
                &PredefinedMenuItem::close_window(handle, None)?,
            ])?;

            let menu = Menu::with_items(handle, &[
                &app_menu,
                &file_menu,
                &edit_menu,
                &view_menu,
                &window_menu
            ])?;

            app.set_menu(menu)?;

            app.on_menu_event(move |app, event| {
                let event_id = event.id().as_ref();
                if event_id == "open_file" {
                    let _ = app.emit("menu:open-file", ());
                } else if event_id == "open_url" {
                    let _ = app.emit("menu:open-url", ());
                } else if event_id == "export_file" {
                    let _ = app.emit("menu:export-file", ());
                } else if event_id == "close_file" {
                    let _ = app.emit("menu:close-file", ());
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![

            file_parser::parse_file_streaming,
            search::search_in_file,
            export::export_to_csv,
            export::export_to_excel,
            network::download_url_to_temp
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}