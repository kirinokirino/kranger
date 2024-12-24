use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use walkdir::{DirEntry, WalkDir};

mod ansi;
mod display;
mod input;
mod update;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new()?;

    app.run()
}
struct App {
    starting_directory: PathBuf,
    current_directory: PathBuf,
    current_selection: usize,
    selected_item: Option<PathBuf>,

    current_directory_contents: Vec<File>,
    parent_directory_contents: Vec<File>,

    should_run: bool,
    directory_changed: bool,
    show_hidden: bool,

    keybindings: HashMap<(KeyCode, KeyModifiers), ApplicationEvent>,

    new_events: Vec<ApplicationEvent>,

    debug_messages: Vec<String>,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let starting_directory = std::env::current_dir().unwrap();
        let current_directory = starting_directory.clone();

        Ok(Self {
            starting_directory,
            current_directory,
            current_selection: 0,
            selected_item: None,

            current_directory_contents: Vec::new(),
            parent_directory_contents: Vec::new(),

            should_run: true,
            directory_changed: true,
            show_hidden: true,

            keybindings: HashMap::new(),

            new_events: Vec::new(),

            debug_messages: Vec::new(),
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        crossterm::terminal::enable_raw_mode()?;

        self.setup();

        while self.should_run {
            if let Err(err) = self.input() {
                self.msg(format!("{}", err));
            }
            self.update();
            self.display();

            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }

    fn setup(&mut self) {
        let default_keybindings = vec![
            (KeyCode::Esc, KeyModifiers::NONE),
            (KeyCode::Char('c'), KeyModifiers::CONTROL),
            (KeyCode::Char('a'), KeyModifiers::NONE),
            (KeyCode::Char('d'), KeyModifiers::NONE),
            (KeyCode::Char('w'), KeyModifiers::NONE),
            (KeyCode::Char('s'), KeyModifiers::NONE),
            //
            (KeyCode::Left, KeyModifiers::NONE),
            (KeyCode::Right, KeyModifiers::NONE),
            (KeyCode::Up, KeyModifiers::NONE),
            (KeyCode::Down, KeyModifiers::NONE),
            (KeyCode::Char('h'), KeyModifiers::NONE),
        ];

        let events_for_default_keybindings = vec![
            ApplicationEvent::Close,
            ApplicationEvent::Close,
            ApplicationEvent::NavigateUp,
            ApplicationEvent::NavigateDown,
            ApplicationEvent::SelectPrevious,
            ApplicationEvent::SelectNext,
            //
            ApplicationEvent::NavigateUp,
            ApplicationEvent::NavigateDown,
            ApplicationEvent::SelectPrevious,
            ApplicationEvent::SelectNext,
            ApplicationEvent::ToggleShowHidden,
        ];
        for ((key, modifiers), event) in default_keybindings
            .into_iter()
            .zip(events_for_default_keybindings)
        {
            self.add_keybinding(key, modifiers, event);
        }
    }

    fn update(&mut self) {
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
            };
            if let Err(err) = result {
                self.msg(format!("Error: {}", err));
            }
        }
    }

    fn msg(&mut self, message: impl AsRef<str>) {
        if self.debug_messages.len() > 5 {
            self.debug_messages.remove(0);
        }
        self.debug_messages.push(message.as_ref().to_owned());
    }
}

fn directory_contents(path: &PathBuf, show_hidden: bool) -> Vec<File> {
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
                unimplemented!()
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

#[derive(Debug, Clone, Copy)]
enum ApplicationEvent {
    Close,
    NavigateUp,
    NavigateDown,
    SelectNext,
    SelectPrevious,
    ToggleShowHidden,
}

#[derive(Debug, Clone)]
struct File {
    ftype: FileType,
    name: String,
}

impl File {
    fn new(ftype: FileType, name: String) -> Self {
        Self { ftype, name }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FileType {
    File,
    Directory,
    Link,
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}
