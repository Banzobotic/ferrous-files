use api_types::{FileInfo, FileType};
use leptos::html::Input;
use leptos::*;
use leptos::{ev::SubmitEvent, logging::log};
use rand::random;
use serde::{Deserialize, Serialize};
use tauri_sys::tauri::invoke;

use std::path::PathBuf;

#[derive(Clone, Copy)]
struct DisplayFile {
    pub id: u128,
    pub info: RwSignal<FileInfo>,
    pub selected: RwSignal<bool>,
}

impl DisplayFile {
    pub fn info(&self) -> FileInfo {
        self.info.get()
    }
}

impl From<FileInfo> for DisplayFile {
    fn from(value: FileInfo) -> Self {
        DisplayFile {
            id: random(),
            info: create_rw_signal(value),
            selected: create_rw_signal(false),
        }
    }
}

#[derive(Clone)]
struct FileListParams {
    pub grid: bool,
    pub visible_columns: Vec<FileColumn>,
}

impl FileListParams {
    fn new() -> Self {
        Self {
            grid: false,
            visible_columns: vec![
                FileColumn::Name,
                FileColumn::Size(9.0),
                FileColumn::LastModified(11.0),
            ],
        }
    }
}

#[derive(Clone)]
enum FileColumn {
    Name,
    Size(f32),
    LastModified(f32),
}

#[derive(Clone, Copy)]
struct Modifiers {
    pub ctrl_key: bool,
    pub shift_key: bool,
}

impl Modifiers {
    fn new() -> Self {
        Self {
            ctrl_key: false,
            shift_key: false,
        }
    }
}

#[derive(Clone)]
struct History {
    history: Vec<PathBuf>,
    position: usize,
}

impl History {
    fn new(start_dir: PathBuf) -> Self {
        Self {
            history: vec![start_dir],
            position: 0,
        }
    }

    fn current_dir(&self) -> PathBuf {
        self.history[self.position].clone()
    }

    fn enter_dir(&mut self, folder: String) {
        log!("{:?}", self.history);
        log!("{}", self.position);

        self.history.truncate(self.position + 1);
        let mut new_dir = self.current_dir();
        new_dir.push(folder);
        self.history.push(new_dir);
        self.position += 1;

        log!("{:?}", self.history);
        log!("{}", self.position);
    }

    fn navigate_to(&mut self, new_dir: PathBuf) {
        self.history.truncate(self.position + 1);
        self.history.push(new_dir);
        self.position += 1;
    }

    fn can_go_forward(&self) -> bool {
        self.position >= self.history.len() - 1
    }

    fn forward(&mut self) {
        self.position += 1;
    }

    fn can_go_back(&self) -> bool {
        self.position == 0
    }

    fn back(&mut self) {
        self.position -= 1;
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (search_term, set_search_term) = create_signal(String::new());
    let (file_list, set_file_list) = create_signal(Vec::new());
    let (search_results, set_search_results) = create_signal(Vec::new());
    let (is_searching, set_searching) = create_signal(false);
    let (file_list_params, set_file_list_params) = create_signal(FileListParams::new());
    let (modifiers, set_modifiers) = create_signal(Modifiers::new());

    let history = create_rw_signal(History::new(PathBuf::from(r"C:\Users\Andrew\Downloads")));
    let selected_files = create_rw_signal(Vec::new());

    let update_search_term = move |ev| {
        let value = event_target_value(&ev);
        set_search_term.set(value);
    };

    let search = move |ev: SubmitEvent| {
        ev.prevent_default();
        spawn_local(async move {
            #[derive(Serialize, Deserialize)]
            #[allow(non_snake_case)]
            struct Args {
                dir: PathBuf,
                searchTerm: String,
            }

            let files = invoke::<_, Vec<FileInfo>>(
                "find_file",
                &Args {
                    dir: history.with(|h| h.current_dir()),
                    searchTerm: search_term.get_untracked(),
                },
            )
            .await
            .unwrap();

            let files: Vec<DisplayFile> = files.into_iter().map(|f| f.into()).collect();

            set_search_results.set(files);
            set_searching(true);
        });
    };

    let get_file_list = move || {
        spawn_local(async move {
            #[derive(Serialize, Deserialize)]
            struct Args {
                dir: PathBuf,
            }

            let files = invoke::<_, Vec<FileInfo>>(
                "files_in_dir",
                &Args {
                    dir: history.with_untracked(|h| h.current_dir()),
                },
            )
            .await
            .unwrap();

            let files: Vec<DisplayFile> = files.into_iter().map(|f| f.into()).collect();

            set_file_list.set(files);
        })
    };

    let open_file = move |file: PathBuf| {
        spawn_local(async move {
            #[derive(Serialize, Deserialize)]
            struct Args {
                file: PathBuf,
            }

            invoke::<_, ()>("open_file", &Args { file })
                .await
                .unwrap();
        });
    };

    let delete_selected_files = move || {
        spawn_local(async move {
            let mut files: Vec<DisplayFile> = selected_files.get_untracked();
            files.dedup_by_key(|f| f.id);

            for file in files.iter() {
                set_file_list.update(|file_list| {
                    let i = file_list.iter().position(|f| f.id == file.id).unwrap();
                    file_list.remove(i);
                });
            }

            let files = if !is_searching.get_untracked() {
                files
                    .iter()
                    .map(|f| {
                        let mut current_dir = history.with_untracked(|h| h.current_dir());
                        current_dir.push(f.info.get_untracked().name);
                        current_dir.clone()
                    })
                    .collect()
            } else {
                files
                    .iter()
                    .map(|f| f.info.get_untracked().full_path.unwrap())
                    .collect()
            };

            #[derive(Serialize, Deserialize)]
            struct DeleteFilesArgs {
                files: Vec<PathBuf>,
            }

            invoke::<_, ()>("delete_files", &DeleteFilesArgs { files })
                .await
                .unwrap();
        })
    };

    let test_file_reactivity = move || {
        set_file_list.update(|files| {
            for file in files {
                file.info.update(|info| info.name.push('a'));
            }
        });
    };

    let last_selected_idx = create_rw_signal(None);
    let shift_select_pivot = create_rw_signal(None);

    let select_file = move |file: DisplayFile, idx: usize| {
        let normal_click = || {
            let initial_state = file.selected.get();

            let len = selected_files.with(|list| {
                list.iter()
                    .for_each(|s: &DisplayFile| s.selected.set(false));
                list.len()
            });

            selected_files.update(|list| {
                list.clear();
                list.push(file);
            });

            if len > 1 {
                file.selected.set(true)
            } else {
                file.selected.set(!initial_state);
            }
        };

        if modifiers().shift_key {
            match shift_select_pivot.get() {
                Some(last_idx) => {
                    let select_range = if idx > last_idx {
                        last_idx..=idx
                    } else {
                        idx..=last_idx
                    };

                    if !modifiers().ctrl_key {
                        selected_files.update(|list| {
                            list.iter()
                                .for_each(|s: &DisplayFile| s.selected.set(false));
                            list.clear();
                        });
                    }

                    for &file in &file_list.get()[select_range] {
                        selected_files.update(|list| {
                            list.push(file);
                        });

                        file.selected.set(true);
                    }
                }
                None => normal_click(),
            }
        } else if modifiers().ctrl_key {
            selected_files.update(|list| {
                list.push(file);
            });

            file.selected.update(|s| *s = !*s);
        } else {
            normal_click();
        }

        if !modifiers().shift_key {
            shift_select_pivot.set(Some(idx));
        }
        last_selected_idx.set(Some(idx));
    };

    let search_box_ref = create_node_ref::<Input>();

    let open_dir = move |file: DisplayFile| {
        if !is_searching() {
            match file.info().file_type {
                FileType::Folder => {
                    history.update(|history| history.enter_dir(file.info().name));
                    selected_files.update(|files| files.clear());
                    get_file_list();
                }
                FileType::File => {
                    let mut file_dir = history.with(|h| h.current_dir());
                    file_dir.push(file.info().name);
                    open_file(file_dir);
                }
            }
        } else {
            match file.info().file_type {
                FileType::Folder => {
                    let search_box = search_box_ref.get().unwrap();

                    set_searching(false);
                    search_box.blur().unwrap();
                    search_box.set_value("");

                    history.update(|history| history.navigate_to(file.info().full_path.unwrap()));
                }
                FileType::File => open_file(file.info().full_path.unwrap()),
            }
        }
    };

    window_event_listener(ev::keydown, move |ev| {
        let search_box = search_box_ref.get().unwrap();

        if ev.key().len() == 1 {
            search_box.focus().unwrap();
        }

        if &ev.code() == "Escape" {
            set_searching(false);
            search_box.blur().unwrap();
            search_box.set_value("");
        }

        if &ev.code() == "Delete" {
            delete_selected_files()
        }

        if ev.code() == "F3" || (ev.key() == "f" || ev.code() == "KeyG") && ev.ctrl_key() {
            ev.prevent_default();
        }

        if ev.key() == "f" && ev.ctrl_key() {
            test_file_reactivity();
        }

        set_modifiers(Modifiers {
            ctrl_key: ev.ctrl_key(),
            shift_key: ev.shift_key(),
        });
    });

    window_event_listener(ev::keyup, move |ev| {
        set_modifiers(Modifiers {
            ctrl_key: ev.ctrl_key(),
            shift_key: ev.shift_key(),
        })
    });

    get_file_list();

    let go_back = move |_| {
        if is_searching() {
            let search_box = search_box_ref.get().unwrap();

            set_searching(false);
            search_box.blur().unwrap();
            search_box.set_value("");
        } else {
            history.update(|h| h.back());
            get_file_list();
        }
    };

    let go_forward = move |_| {
        history.update(|h| h.forward());
        get_file_list();
    };

    view! {
        <div class="top-bar">
            <button on:click=go_back disabled=move || history.with(|h| h.can_go_back())>"⬅"</button>
            <button on:click=go_forward disabled=move || history.with(|h| h.can_go_forward())>"➡"</button>
            <form on:submit=search>
                <input
                    id="search-box"
                    _ref=search_box_ref
                    placeholder="Search for a file..."
                    autocomplete="off"
                    on:input=update_search_term
                />
                <button type="submit">"🔎"</button>
            </form>
        </div>
        <main>
            <div class="file-list">
                <For
                    each=move || if is_searching() { search_results } else { file_list }.with(|files| files.clone().into_iter().enumerate())
                    key=|file| file.1.id
                    view=move |(idx, file)| {
                        view! {
                            <File
                                file
                                params=file_list_params
                                on:click=move |_| select_file(file, idx)
                                on:dblclick=move |_| open_dir(file)
                            />
                        }
                    }
                />
            </div>
        </main>
    }
}

#[component]
fn File(file: DisplayFile, params: ReadSignal<FileListParams>) -> impl IntoView {
    view! {
        <div class="file-row" class:selected=move || file.selected.get()>
            {move || {
                if file.info().file_type == FileType::Folder {
                    view! { <img src="public/folder.svg"/> }
                } else {
                    view! { <img src="public/file.svg"/> }
                }
            }}

            {move || {
                params()
                    .visible_columns
                    .into_iter()
                    .map(|column| match column {
                        FileColumn::Name => {
                            view! {
                                <p>{move || file.info().name}</p>
                            }
                        }
                        FileColumn::Size(width) => {
                            view! {
                                <p style:width=move || {
                                    format!("{}ch", width)
                                }>{move || file.info().size_fmt()}</p>
                            }
                        }
                        FileColumn::LastModified(width) => {
                            view! {
                                <p style:width=move || {
                                    format!("{}ch", width)
                                }>{move || file.info().last_modified_fmt()}</p>
                            }
                        }
                    })
                    .collect_view()
            }}

        </div>
    }
}
