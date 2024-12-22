use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use crossterm::event::{poll, read, Event, KeyCode, KeyModifiers};
use walkdir::WalkDir;

mod ansi {
    pub const CLEAR: &str = "\x1B[2J\x1B[1;1H";
    pub const RESET: &str = "\x1B[0m";
    pub const WHITE: &str = "\x1B[37m";
    pub const BLUE: &str = "\x1B[34m";
    pub const CYAN: &str = "\x1B[36m";
    /*
    Regular Files: White (\x1B[37m)
    Directories: Blue (\x1B[34m)
    Symbolic Links: Cyan (\x1B[36m)
    Executable Files: Green (\x1B[32m)
    Archive Files: Magenta (\x1B[35m)
    Compressed Files: Yellow (\x1B[33m)
    Socket Files: Magenta (\x1B[35m)
    FIFO Files: Yellow (\x1B[33m)
    Device Files: Red (\x1B[31m)
     */
}

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

            std::thread::sleep(std::time::Duration::from_millis(50));
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
        ];

        let events_for_default_keybindings = vec![
            ApplicationEvent::Close,
            ApplicationEvent::Close,
            ApplicationEvent::NavigateUp,
            ApplicationEvent::NavigateDown,
            ApplicationEvent::SelectPrevious,
            ApplicationEvent::SelectNext,
        ];
        for ((key, modifiers), event) in default_keybindings
            .into_iter()
            .zip(events_for_default_keybindings)
        {
            self.add_keybinding(key, modifiers, event);
        }
    }

    fn input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key_event) = read()? {
                let (key, modifiers) = (key_event.code, key_event.modifiers);
                if let Some(event) = self.resolve_keybinding(key, modifiers) {
                    self.new_events.push(event);
                }
            }
        }

        Ok(())
    }

    fn update(&mut self) {
        if self.directory_changed {
            self.current_directory_contents = directory_contents(&self.current_directory);
            self.parent_directory_contents =
                directory_contents(&self.parent_directory().unwrap_or("\\".into()));
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
            };
            if let Err(err) = result {
                self.msg(format!("Error: {}", err));
            }
        }
    }

    fn display(&self) {
        let empty = String::new();
        print!("{}", ansi::CLEAR);
        self.show_breadcrumbs();

        let (from, to) = self.rows_to_print();
        for line in from..to {
            let is_selected = line == self.current_selection;
            let selection_arrow = match is_selected {
                true => "->",
                false => "  ",
            };

            let formatted_current_item =
                self.display_file(self.current_directory_contents.get(line), 10);

            let formatted_parent_item =
                self.display_file(self.current_directory_contents.get(line), 15);

            println!(
                "{} | {selection_arrow} {}\r",
                formatted_parent_item, formatted_current_item
            );
        }

        println!("\r");
        for line in &self.debug_messages {
            println!("{}\r", line);
        }
    }

    // Input

    fn add_keybinding(&mut self, key: KeyCode, modifiers: KeyModifiers, event: ApplicationEvent) {
        self.keybindings.insert((key, modifiers), event);
    }

    fn resolve_keybinding(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
    ) -> Option<ApplicationEvent> {
        self.keybindings.get(&(key, modifiers)).copied()
    }

    // Update

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

    // Display

    fn rows_to_print(&self) -> (usize, usize) {
        let rows_to_show = 12;

        let max_lines = self
            .current_directory_contents
            .len()
            .max(self.parent_directory_contents.len());

        if max_lines > rows_to_show {
            let half = rows_to_show / 2;
            if self.current_selection < half {
                // limited from the start
                (0, rows_to_show)
            } else if self.current_selection > (max_lines - half) {
                // limited from the end
                (max_lines - rows_to_show, max_lines)
            } else {
                // if selection > half of rows_to_show, add the diff to the window
                let start_offset = self.current_selection - half;
                (start_offset, self.current_selection + half)
            }
        } else {
            (0, max_lines)
        }
    }

    fn display_file(&self, file: Option<&File>, max_length: usize) -> String {
        if let Some(file) = file {
            match file.ftype {
                FileType::File => self.display_normal_file(file, max_length),
                FileType::Directory => self.display_directory(file, max_length),
                FileType::Link => self.display_link(file, max_length),
            }
        } else {
            " ".repeat(max_length)
        }
    }

    fn display_normal_file(&self, file: &File, max_length: usize) -> String {
        format!(
            "{}{}{}",
            ansi::WHITE,
            truncate_with_ellipsis(&file.name, max_length),
            ansi::RESET
        )
    }
    fn display_directory(&self, file: &File, max_length: usize) -> String {
        format!(
            "{}{}{}",
            ansi::BLUE,
            truncate_with_ellipsis(&file.name, max_length),
            ansi::RESET
        )
    }
    fn display_link(&self, file: &File, max_length: usize) -> String {
        format!(
            "{}{}{}",
            ansi::CYAN,
            truncate_with_ellipsis(&file.name, max_length),
            ansi::RESET
        )
    }

    fn show_breadcrumbs(&self) {
        println!("{}\r", self.current_directory.display());
    }

    fn msg(&mut self, message: impl AsRef<str>) {
        if self.debug_messages.len() > 5 {
            self.debug_messages.remove(0);
        }
        self.debug_messages.push(message.as_ref().to_owned());
    }
}

fn directory_contents(path: &PathBuf) -> Vec<File> {
    use walkdir::DirEntry;
    WalkDir::new(path)
        .max_depth(1)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
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
        .collect()
}

fn truncate_with_ellipsis(input: &str, max_length: usize) -> String {
    if input.len() > max_length {
        format!("{}…", &input[..max_length - 1])
    } else {
        format!("{:<width$}", input, width = max_length)
    }
}

#[derive(Debug, Clone, Copy)]
enum ApplicationEvent {
    Close,
    NavigateUp,
    NavigateDown,
    SelectNext,
    SelectPrevious,
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

#[derive(Debug, Clone, Copy)]
enum FileType {
    File,
    Directory,
    Link,
}
