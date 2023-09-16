// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs, path::PathBuf};
use walkdir::WalkDir;

use api_types::FileInfo;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn files_in_dir(dir: PathBuf) -> String {
    let mut file_list = Vec::new();

    for file in fs::read_dir(dir).unwrap().flatten() {
        file_list.push(FileInfo::new(file.path()));
    }

    file_list.sort_by_key(|file| file.file_type);

    serde_json::to_string(&file_list).unwrap()
}

#[tauri::command]
fn find_file(dir: PathBuf, search_term: String) -> String {
    let mut results = Vec::new();

    for entry in WalkDir::new(dir).into_iter().flatten() {
        let file_name = entry.file_name().to_str().unwrap();

        if file_name
            .to_lowercase()
            .contains(&search_term.to_lowercase())
        {
            results.push(FileInfo::new(entry.into_path()));
        }
    }

    serde_json::to_string(&results).unwrap()
}

#[tauri::command]
fn open_file(dir: PathBuf) {
    open::that_detached(dir).unwrap();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![files_in_dir, find_file, open_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
