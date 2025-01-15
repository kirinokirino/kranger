use anyhow::{anyhow, Error, Result};

use std::{
    io::Read,
    path::PathBuf,
    process::{Command, Output},
};

pub fn run_external_command(command: &str, args: &[&str]) -> Result<Option<Vec<String>>> {
    let output: Output = Command::new(command).args(args).output()?;

    // Check if the command was successful
    if !output.status.success() {
        return Err(anyhow!(
            "Executing external command failed with code: {}",
            output.status.code().unwrap_or(-1)
        ));
    }
    // Convert the output to a vector of lines
    let stdout = String::from_utf8_lossy(&output.stdout);
    let content: Vec<String> = stdout.lines().map(|line| line.trim().to_string()).collect();
    if content.is_empty() {
        Ok(None)
    } else {
        Ok(Some(content))
    }
}

pub fn get_media_length(path: &str) -> Result<f32> {
    let command = "ffprobe";
    let args = &[
        "-show_entries",
        "format=duration",
        "-of",
        "default=noprint_wrappers=1:nokey=1",
        path,
    ];
    let output: Output = Command::new(command).args(args).output()?;

    // Check if the command was successful
    if !output.status.success() {
        return Err(anyhow!(
            "Executing {command} {args:?} failed with code: {}",
            output.status.code().unwrap_or(-1)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next().unwrap();
    str::parse::<f32>(line.trim()).map_err(Error::from)
}

pub fn probably_valid_utf(path: &PathBuf) -> bool {
    if let Ok(mut file) = std::fs::File::open(path) {
        let mut buffer = vec![0; 1024]; // Read 1KB for analysis
        if let Ok(bytes) = file.read(&mut buffer) {
            // Check if the content is valid UTF-8
            return std::str::from_utf8(&buffer[..bytes]).is_ok();
        }
    }
    false
}
