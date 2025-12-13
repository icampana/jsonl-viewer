// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use commands::{file_parser, search, export};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            file_parser::parse_file_streaming,
            search::search_in_file,
            export::export_to_csv
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}