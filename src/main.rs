// when selected non-directory, run something on ->
//		depending on what is selected

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use file::File;
use info::Info;

mod ansi;
mod display;
mod file;
mod info;
mod input;
mod update;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new()?;

    app.run()
}
struct App {
    width: usize,
    height: usize,

    starting_directory: PathBuf,
    current_directory: PathBuf,
    current_selection: usize,
    selected_item: Option<PathBuf>,

    current_directory_contents: Vec<File>,
    parent_directory_contents: Vec<File>,
    selection_info: Option<Info>,

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
            width: 80,
            height: 15,
            starting_directory,
            current_directory,
            current_selection: 0,
            selected_item: None,

            current_directory_contents: Vec::new(),
            parent_directory_contents: Vec::new(),
            selection_info: None,

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
        let _ = crossterm::execute!(std::io::stdout(), crossterm::cursor::Hide);

        self.setup();

        while self.should_run {
            if let Err(err) = self.input() {
                self.msg(format!("{}", err));
            }
            self.update();
            self.display();

            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let _ = crossterm::execute!(std::io::stdout(), crossterm::cursor::Show);
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }

    fn setup(&mut self) {
        self.add_default_keybindings();
    }

    fn msg(&mut self, message: impl AsRef<str>) {
        if self.debug_messages.len() > 5 {
            self.debug_messages.remove(0);
        }
        self.debug_messages.push(message.as_ref().to_owned());
    }
}

#[derive(Debug, Clone, Copy)]
enum ApplicationEvent {
    Close,
    NavigateUp,
    NavigateDown,
    SelectNext,
    SelectPrevious,
    OpenImage,
    OpenText,
    OpenExecutable,
    ToggleShowHidden,
    DebugEvent,
}
