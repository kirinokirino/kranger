use std::path::PathBuf;

use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone)]
pub struct File {
    pub ftype: FileType,
    pub name: String,
}

impl File {
    pub fn new(ftype: FileType, name: String) -> Self {
        Self { ftype, name }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    File,
    Directory,
    Link,
    Unknown,
}

pub fn directory_contents(path: &PathBuf, show_hidden: bool) -> Vec<File> {
    let mut files: Vec<File> = WalkDir::new(path)
        .max_depth(1)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| {
            if show_hidden {
                e.ok()
            } else {
                match e {
                    Ok(entry) => match is_hidden(&entry) {
                        true => None,
                        false => Some(entry),
                    },
                    Err(_) => None,
                }
            }
        })
        .map(|entry| {
            let ftype = entry.file_type();
            let ftype = if ftype.is_file() {
                FileType::File
            } else if ftype.is_dir() {
                FileType::Directory
            } else if ftype.is_symlink() {
                FileType::Link
            } else {
                FileType::Unknown
            };
            let name = entry.file_name().to_string_lossy().into_owned();
            File::new(ftype, name)
        })
        .collect();

    files.sort_by(|f1, f2| {
        let dir1 = f1.ftype == FileType::Directory;
        let dir2 = f2.ftype == FileType::Directory;
        if dir1 && dir2 {
            f1.name.cmp(&f2.name)
        } else if dir1 && !dir2 {
            std::cmp::Ordering::Less
        } else if !dir1 && dir2 {
            std::cmp::Ordering::Greater
        } else {
            f1.name.cmp(&f2.name)
        }
    });

    files
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}
