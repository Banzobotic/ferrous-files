use std::{fs, path::PathBuf};

use chrono::{DateTime, Datelike, Local};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum FileType {
    Folder,
    SymLink,
    File,
}

impl From<&fs::Metadata> for FileType {
    fn from(value: &fs::Metadata) -> Self {
        if value.is_dir() {
            FileType::Folder
        } else if value.is_symlink() {
            FileType::SymLink
        } else if value.is_file() {
            FileType::File
        } else {
            unreachable!("Unrecognised file type");
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct FileInfo {
    pub name: String,
    pub file_type: FileType,
    pub size: u64,
    pub item_count: Option<usize>,
    pub last_modified: DateTime<Local>,
}

impl FileInfo {
    pub fn new(path: PathBuf) -> Self {
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        let metadata = &path.metadata().unwrap();

        let item_count = fs::read_dir(path).map(|r| r.count()).ok();

        FileInfo {
            name,
            file_type: metadata.into(),
            size: metadata.len(),
            item_count,
            last_modified: metadata.modified().unwrap().into(),
        }
    }
}

impl FileInfo {
    pub fn last_modified_fmt(&self) -> String {
        if self.last_modified.date_naive() == Local::now().date_naive() {
            format!("{}", self.last_modified.format("%H:%M"))
        } else if self.last_modified.naive_local().year() == Local::now().naive_local().year() {
            format!("{}", self.last_modified.format("%d %b"))
        } else {
            format!("{}", self.last_modified.format("%d %b %Y"))
        }
    }

    pub fn size_fmt(&self) -> String {
        match self.file_type {
            FileType::File => humansize::format_size(self.size, humansize::DECIMAL),
            FileType::Folder => {
                let item_count = self.item_count.unwrap();

                if item_count == 0 {
                    "Empty".to_string()
                } else if item_count == 1 {
                    format!("{item_count} item")
                } else {
                    format!("{item_count} items")
                }
            }
            FileType::SymLink => String::new(),
        }
    }
}
