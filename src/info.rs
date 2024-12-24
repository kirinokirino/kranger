use std::{
    fs::read_to_string,
    io::Read,
    os::unix::fs::MetadataExt,
    path::PathBuf,
    process::{Command, Output},
};

use anyhow::{anyhow, Result};
use phf::phf_map;

use crate::{display::display_file, file::directory_contents};

pub struct Info {
    info_type: InfoType,

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
                let mut lines = run_ldd(file.to_str().unwrap()).unwrap_or_default();
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
            _ => Vec::new(),
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

fn run_ldd(executable: impl AsRef<str>) -> Result<Vec<String>> {
    // Execute the `ldd` command
    let output: Output = Command::new("ldd").arg(executable.as_ref()).output()?;

    // Check if the command was successful
    if !output.status.success() {
        return Err(anyhow!(
            "ldd command failed with exit code: {}",
            output.status.code().unwrap_or(-1)
        ));
    }

    // Convert the output to a vector of lines
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().map(|line| line.trim().to_string()).collect())
}
