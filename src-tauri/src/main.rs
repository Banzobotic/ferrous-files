// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs, path::PathBuf, sync::{mpsc, Mutex}, thread};
use walkdir::WalkDir;
use tauri::{Window, Manager};
use notify::{RecommendedWatcher, recommended_watcher, EventKind};

use api_types::FileInfo;

#[derive(Default)]
struct AppState {
    watcher: Mutex<Option<RecommendedWatcher>>,
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
// #[tauri::command]
// async fn watch_fs_changes(path: PathBuf, window: Window) {
//     let (tx, rx) = mpsc::channel();
//     let watcher = recommended_watcher(tx).unwrap();

//     thread::spawn(move || {
//         for msg in rx {
//             if let Ok(event) = msg {
//                 match event.kind {
//                     EventKind::Remove => {

//                     }
//                     _ => (),
//                 }
//             }
//         }
//     });
// }

#[tauri::command]
fn files_in_dir(dir: PathBuf) -> Vec<FileInfo> {
    let mut file_list = Vec::new();

    for file in fs::read_dir(dir).unwrap().flatten() {
        file_list.push(FileInfo::new(file.path(), false));
    }

    file_list.sort_by_key(|file| file.file_type);

    file_list
}

#[tauri::command]
fn find_file(dir: PathBuf, search_term: String) -> Vec<FileInfo> {
    let mut results = Vec::new();

    for entry in WalkDir::new(dir).into_iter().flatten() {
        let file_name = entry.file_name().to_str().unwrap();

        if file_name
            .to_lowercase()
            .contains(&search_term.to_lowercase())
        {
            results.push(FileInfo::new(entry.into_path(), true));
        }
    }

    results
}

#[tauri::command]
fn open_file(file: PathBuf) {
    open::that_detached(file).unwrap();
}

#[tauri::command]
fn delete_files(files: Vec<PathBuf>) {
    dbg!(&files);
    trash::delete_all(files).unwrap();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            files_in_dir,
            find_file,
            open_file,
            delete_files
        ])
        .setup(|app| {
            app.manage(AppState::default());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
