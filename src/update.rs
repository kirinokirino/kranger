use crate::file::{directory_contents, FileType};
use crate::info::Info;
use crate::{App, ApplicationEvent};

use anyhow::{anyhow, Result};

use std::{
    path::PathBuf,
    process::{Command, Output},
};

impl App {
    pub fn update(&mut self) {
        self.update_window_size();
        if self.directory_changed {
            self.current_directory_contents =
                directory_contents(&self.current_directory, self.show_hidden);
            self.parent_directory_contents = directory_contents(
                &self.parent_directory().unwrap_or("\\".into()),
                self.show_hidden,
            );
            self.directory_changed = false;

            self.current_selection = 0;
            self.update_selected_item();
        }

        let mut events = std::mem::take(&mut self.new_events);
        for event in events.drain(..) {
            let result = match event {
                ApplicationEvent::Close => {
                    self.should_run = false;
                    Ok(())
                }
                ApplicationEvent::NavigateUp => self.navigate_up(),
                ApplicationEvent::NavigateDown => self.navigate_down(),
                ApplicationEvent::SelectNext => self.change_selection(1),
                ApplicationEvent::SelectPrevious => self.change_selection(-1),
                ApplicationEvent::ToggleShowHidden => {
                    self.show_hidden = !self.show_hidden;
                    self.directory_changed = true;
                    Ok(())
                }
                ApplicationEvent::DebugEvent => {
                    let command = "pfiew";
                    let args = self.selected_item.clone().unwrap();
                    let args = args.to_str().unwrap();
                    self.run_command(command, &[format!("--input={}", args).as_str()])
                }
            };
            if let Err(err) = result {
                self.msg(format!("Error: {}", err));
            }
        }
    }

    fn update_window_size(&mut self) {
        self.msg(format!("{:?}", crossterm::terminal::window_size()));
    }

    fn parent_directory(&self) -> std::option::Option<PathBuf> {
        self.current_directory
            .ancestors()
            .nth(1)
            .map(|path| path.to_path_buf())
    }

    fn navigate_up(&mut self) -> Result<()> {
        let parent_directory = self
            .parent_directory()
            .ok_or(anyhow!("No parent directory available"))?;
        self.change_directory(parent_directory);
        Ok(())
    }

    fn navigate_down(&mut self) -> Result<()> {
        let selection = self
            .selected_item
            .clone()
            .ok_or(anyhow!("No item selected!"))?;
        match selection.is_dir() {
            true => {
                self.change_directory(selection);
                Ok(())
            }
            false => Err(anyhow!("Selected item is not a directory!")),
        }
    }

    fn change_directory(&mut self, to: PathBuf) {
        self.current_directory = to;
        self.directory_changed = true;
    }

    fn update_selected_item(&mut self) {
        match self.current_directory_contents.get(self.current_selection) {
            Some(item) => {
                self.selected_item = Some(self.current_directory.join(item.name.clone()));
                let path = self.selected_item.as_ref().unwrap();
                match item.ftype {
                    FileType::File => self.selection_info = Info::new(path).ok(),
                    FileType::Directory => self.selection_info = Some(Info::directory(path)),
                    FileType::Link => self.selection_info = Some(Info::link(path)),
                    FileType::Unknown => self.selected_item = None,
                }
            }
            _ => self.selected_item = None,
        };
    }

    fn change_selection(&mut self, change_by: i32) -> Result<()> {
        let should_loop = false;
        let max_selection = self.current_directory_contents.len() as i32;

        let mut next_selection = self.current_selection as i32 + change_by;
        if next_selection >= max_selection {
            if should_loop {
                next_selection = 0;
            } else {
                next_selection = max_selection - 1;
            }
        } else if next_selection < 0 {
            if should_loop {
                next_selection = max_selection - 1;
            } else {
                next_selection = 0;
            }
        }

        self.current_selection = next_selection as usize;
        self.update_selected_item();
        Ok(())
    }

    fn run_command(&mut self, command: &str, args: &[&str]) -> Result<()> {
        self.msg(format!("Running {} with {:?}", command, args));
        // Execute the `ldd` command
        let output: Output = Command::new(command).args(args).output()?;

        // Check if the command was successful
        if !output.status.success() {
            return Err(anyhow!(
                "Executing external command failed with code: {}",
                output.status.code().unwrap_or(-1)
            ));
        }

        //let stdout = String::from_utf8_lossy(&output.stdout);
        //Ok(stdout.lines().map(|line| line.trim().to_string()).collect())
        Ok(())
    }
}
