use std::path::PathBuf;

use anyhow::{anyhow, Result};
use crossterm::event::{poll, read, Event, KeyCode, KeyModifiers};
use walkdir::WalkDir;

const CLEAR: &str = "\x1B[2J\x1B[1;1H";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new()?;

    app.run()
}
struct App {
    starting_directory: PathBuf,
    current_directory: PathBuf,

    current_directory_contents: Vec<String>,
    parent_directory_contents: Vec<String>,

    should_run: bool,
    directory_changed: bool,

    new_events: Vec<ApplicationEvent>,

    debug_messages: Vec<String>,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let starting_directory = std::env::current_dir().unwrap();
        let mut current_directory = starting_directory.clone();

        Ok(Self {
            starting_directory,
            current_directory,

            current_directory_contents: Vec::new(),
            parent_directory_contents: Vec::new(),

            should_run: true,
            directory_changed: true,

            new_events: Vec::new(),

            debug_messages: Vec::new(),
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        crossterm::terminal::enable_raw_mode()?;
        while self.should_run {
            self.input();
            self.update();
            self.display();

            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
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
        }

        let mut events = std::mem::take(&mut self.new_events);
        for event in events.drain(..) {
            match event {
                ApplicationEvent::Close => self.should_run = false,
                ApplicationEvent::NavigateUp => self
                    .navigate_up()
                    .unwrap_or_else(|error| self.msg(format!("Unable to navigate up, {error}"))),
            }
        }
    }

    fn display(&self) {
        let empty = String::new();
        print!("{CLEAR}");
        self.show_breadcrumbs();

        let max_lines = self
            .current_directory_contents
            .len()
            .max(self.parent_directory_contents.len());

        for line in 0..max_lines {
            let current_item = self.current_directory_contents.get(line).unwrap_or(&empty);

            let parent_item = self.parent_directory_contents.get(line).unwrap_or(&empty);

            println!(
                "{} | {}\r",
                truncate_with_ellipsis(parent_item, 10),
                truncate_with_ellipsis(current_item, 15)
            );
        }

        println!("\r");
        for line in &self.debug_messages {
            println!("{}\r", line);
        }
    }

    // Input

    fn resolve_keybinding(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
    ) -> Option<ApplicationEvent> {
        match (key, modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(ApplicationEvent::Close),
            // (KeyCode::Char(c), _) => {
            //     self.msg(format!("You pressed: {}", c));
            //     None
            // }
            (KeyCode::Esc, _) => Some(ApplicationEvent::Close),
            (KeyCode::Char('a'), _) => Some(ApplicationEvent::NavigateUp),
            other => {
                self.msg(format!("Other: {other:?}"));
                None
            }
        }
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

    fn change_directory(&mut self, to: PathBuf) {
        self.current_directory = to;
        self.directory_changed = true;
    }

    // Display

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

fn directory_contents(path: &PathBuf) -> Vec<String> {
    WalkDir::new(path)
        .max_depth(1)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
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
}
