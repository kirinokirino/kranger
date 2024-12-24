use std::{fs::read_to_string, io::Read, path::PathBuf};

use anyhow::Result;
use phf::phf_map;

use crate::{display::display_file, file::directory_contents};

pub struct Info {
    info_type: InfoType,

    info_lines: Vec<String>,
}

impl Info {
    pub fn new(file: &PathBuf) -> Result<Self> {
        let info_type = InfoType::new(file)?;

        let mut info_lines = Vec::new();
        match info_type {
            InfoType::Text => {
                if let Ok(contents) = read_to_string(file) {
                    for line in contents.lines().take(50) {
                        info_lines.push(String::from(line))
                    }
                }
            }
            _ => (),
        };

        Ok(Self {
            info_type,
            info_lines,
        })
    }

    pub fn link(link: &PathBuf) -> Self {
        Self {
            info_type: InfoType::Link,
            info_lines: Vec::new(),
        }
    }

    pub fn directory(directory: &PathBuf) -> Self {
        let directory_contents = directory_contents(directory, false);

        let mut info_lines = Vec::with_capacity(directory_contents.len());
        for file in directory_contents {
            info_lines.push(display_file(Some(&file), 50));
        }

        Self {
            info_type: InfoType::Directory,
            info_lines,
        }
    }

    pub fn lines(&self) -> Vec<String> {
        if self.info_lines.is_empty() {
            vec![format!("{:?}", self.info_type)]
        } else {
            self.info_lines.clone()
        }
    }
}

static KNOWN_NAMES: phf::Map<&'static str, InfoType> = phf_map! {
    "README" => InfoType::Text,
    ".gitignore" => InfoType::Text,
};

#[derive(Debug, Copy, Clone)]
enum InfoType {
    Executable,
    Text,
    Unknown,
    Link,
    Directory,
}

impl InfoType {
    pub fn new(file: &PathBuf) -> Result<Self> {
        let result = if let Some(extension) = file.extension() {
            InfoType::from_extension(extension.to_str())
        } else if let Some(info_type) = KNOWN_NAMES.get(
            file.file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default(),
        ) {
            *info_type
        } else {
            InfoType::from_contents(file)
        };
        Ok(result)
    }

    pub fn from_extension(extension: Option<&str>) -> Self {
        match extension {
            Some(extension) => match extension {
                "rs" | "md" | "txt" | "toml" | "lock" | "ini" => Self::Text,
                "exe" => Self::Executable,
                _ => Self::Unknown,
            },
            None => Self::Unknown,
        }
    }
    pub fn from_contents(path: &PathBuf) -> Self {
        if let Ok(mut file) = std::fs::File::open(path) {
            let mut magic_bytes = [0u8; 4];
            if let Ok(bytes_read) = file.read(&mut magic_bytes) {
                if bytes_read < 4 {
                    return InfoType::Unknown;
                }

                // Match the magic bytes
                return match &magic_bytes {
                    b"\x7FELF" => InfoType::Executable,
                    // b"\x89PNG" => Some("PNG"),
                    // b"%PDF" => Some("PDF"),
                    _ => {
                        if probably_valid_utf(path) {
                            InfoType::Text
                        } else {
                            InfoType::Unknown
                        }
                    }
                };
            } // couldn't read bytes
        } // couldn't open file
        InfoType::Unknown
    }
}

fn probably_valid_utf(path: &PathBuf) -> bool {
    if let Ok(mut file) = std::fs::File::open(path) {
        let mut buffer = vec![0; 1024]; // Read 1KB for analysis
        if let Ok(bytes) = file.read(&mut buffer) {
            // Check if the content is valid UTF-8
            return std::str::from_utf8(&buffer[..bytes]).is_ok();
        }
    }
    false
}
