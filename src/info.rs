use std::path::Path;
use std::{fs::read_to_string, io::Read, os::unix::fs::MetadataExt, path::PathBuf};

use anyhow::Result;
use phf::phf_map;

use crate::display::display_file;
use crate::external::{probably_valid_utf, run_external_command};
use crate::file::directory_contents;

pub struct Info {
    pub info_type: InfoType,

    info_lines: Vec<String>,
}

impl Info {
    pub fn new(file: &PathBuf) -> Result<Self> {
        let info_type = InfoType::new(file)?;

        let info_lines = match info_type {
            InfoType::Text => read_to_string(file)
                .unwrap_or_default()
                .lines()
                .take(50)
                .map(|s| s.to_string())
                .collect(),
            InfoType::Executable => {
                let mut lines = match run_external_command("ldd", &[file.to_str().unwrap()]) {
                    Ok(output) => output.unwrap(),
                    Err(_err) => vec![String::from("Unable to run ldd")],
                };
                lines.insert(
                    0,
                    format!(
                        "Executable {}",
                        std::fs::metadata(file)
                            .map(|meta| format!("{:.2} MB", meta.size() as f32 / (1024.0 * 1024.0)))
                            .unwrap_or("Unknown size".to_string())
                    ),
                );
                lines
            }
            InfoType::Audio | InfoType::Video => {
                let mut lines = match run_external_command("metadata", &[file.to_str().unwrap()]) {
                    Ok(output) => output.unwrap(),
                    Err(_err) => vec![String::from("Unable to get metadata")],
                };
                lines.insert(0, format!("{info_type:?}"));
                lines
            }
            _ => Vec::new(),
        };

        Ok(Self {
            info_type,
            info_lines,
        })
    }

    pub fn link(_link: &Path) -> Self {
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
pub enum InfoType {
    Executable,
    Text,
    Unknown,
    Image,
    Video,
    Audio,
    Pdf,
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
                "png" | "jpg" | "jpeg" => Self::Image,
                "opus" | "flac" | "mp3" | "wav" | "ogg" => Self::Audio,
                "mp4" | "mkv" | "webm" => Self::Video,
                "pdf" => Self::Pdf,
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
