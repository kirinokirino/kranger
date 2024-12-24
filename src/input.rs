use crate::{App, ApplicationEvent};

use crossterm::event::{poll, read, Event, KeyCode, KeyModifiers};

impl App {
    pub fn input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

    pub fn add_keybinding(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        event: ApplicationEvent,
    ) {
        self.keybindings.insert((key, modifiers), event);
    }

    pub fn resolve_keybinding(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
    ) -> Option<ApplicationEvent> {
        self.keybindings.get(&(key, modifiers)).copied()
    }
}
