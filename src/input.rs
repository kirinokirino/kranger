use crate::{App, ApplicationEvent};

use crossterm::event::{poll, read, Event, KeyCode, KeyModifiers};

impl App {
    pub fn input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key_event) = read()? {
                let (key, modifiers) = (key_event.code, key_event.modifiers);
                if let Some(event) = self.resolve_keybinding(key, modifiers) {
                    self.new_events.push(event);
                }
            }
        }

        Ok(())
    }

    pub fn add_default_keybindings(&mut self) {
        let default_keybindings = vec![
            // close
            (KeyCode::Esc, KeyModifiers::NONE),
            (KeyCode::Char('c'), KeyModifiers::CONTROL),
            // navigation
            (KeyCode::Char('a'), KeyModifiers::NONE),
            (KeyCode::Char('d'), KeyModifiers::NONE),
            (KeyCode::Char('w'), KeyModifiers::NONE),
            (KeyCode::Char('s'), KeyModifiers::NONE),
            (KeyCode::Left, KeyModifiers::NONE),
            (KeyCode::Right, KeyModifiers::NONE),
            (KeyCode::Up, KeyModifiers::NONE),
            (KeyCode::Down, KeyModifiers::NONE),
            //
            (KeyCode::Char('h'), KeyModifiers::NONE),
            (KeyCode::Char('f'), KeyModifiers::NONE),
            (KeyCode::Char('p'), KeyModifiers::NONE),
            (KeyCode::Char('q'), KeyModifiers::NONE),
        ];

        let events_for_default_keybindings = vec![
            //close
            ApplicationEvent::Close,
            ApplicationEvent::Close,
            //navigation
            ApplicationEvent::NavigateUp,
            ApplicationEvent::NavigateDown,
            ApplicationEvent::SelectPrevious,
            ApplicationEvent::SelectNext,
            ApplicationEvent::NavigateUp,
            ApplicationEvent::NavigateDown,
            ApplicationEvent::SelectPrevious,
            ApplicationEvent::SelectNext,
            //
            ApplicationEvent::ToggleShowHidden,
            ApplicationEvent::OpenImage,
            ApplicationEvent::PlayMedia,
            ApplicationEvent::DebugEvent,
        ];
        for ((key, modifiers), event) in default_keybindings
            .into_iter()
            .zip(events_for_default_keybindings)
        {
            self.add_keybinding(key, modifiers, event);
        }
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
