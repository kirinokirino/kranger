use std::path::PathBuf;

use crossterm::event::{poll, read, Event, KeyCode, KeyModifiers};

const CLEAR: &str = "\x1B[2J\x1B[1;1H";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new()?;

    app.run()
}
struct App {
    starting_directory: PathBuf,
    current_directory: PathBuf,
    should_run: bool,

    debug_messages: Vec<String>,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let starting_directory = std::env::current_dir().unwrap();
        let mut current_directory = starting_directory.clone();

        Ok(Self {
            starting_directory,
            current_directory,
            should_run: true,

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
                match (key_event.code, key_event.modifiers) {
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        self.msg("Exiting via Ctrl+C...");
                        self.should_run = false;
                    }
                    (KeyCode::Char(c), _) => self.msg(format!("You pressed: {}", c)),
                    (KeyCode::Enter, _) => self.msg("You pressed Enter!"),
                    (KeyCode::Esc, _) => {
                        self.msg("Closing krender");
                        self.should_run = false;
                    }
                    _ => self.msg("Some other key pressed"),
                }
            }
        }

        Ok(())
    }
    fn update(&mut self) {}
    fn display(&self) {
        print!("{CLEAR}");
        self.show_breadcrumbs();

        for line in &self.debug_messages {
            println!("{}\r", line);
        }
    }

    fn show_breadcrumbs(&self) {
        println!("{}\r", self.current_directory.display());
    }

    fn msg<T: AsRef<str>>(&mut self, message: T) {
        if self.debug_messages.len() > 5 {
            self.debug_messages.remove(0);
        }
        self.debug_messages.push(message.as_ref().to_owned());
    }
}
