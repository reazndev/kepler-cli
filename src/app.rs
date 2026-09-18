use std::{collections::HashMap, path::PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::catalog::CommandEntry;

#[derive(Default)]
pub(crate) struct App {
    pub(crate) query: String,
    pub(crate) should_exit: bool,
    pub(crate) commands: Vec<CommandEntry>,
    pub(crate) matches: Vec<usize>,
    pub(crate) selected: usize,
    pub(crate) selection: Option<String>,
    help_cache: HashMap<PathBuf, String>,
}

impl App {
    pub(crate) fn new(commands: Vec<CommandEntry>) -> Self {
        let mut app = Self {
            commands,
            ..Self::default()
        };
        app.refresh_matches();
        app
    }

    pub(crate) fn selected_command(&self) -> Option<&CommandEntry> {
        self.matches
            .get(self.selected)
            .and_then(|index| self.commands.get(*index))
    }

    pub(crate) fn selected_help(&self) -> Option<&str> {
        self.selected_command()
            .and_then(|command| self.help_cache.get(&command.path))
            .map(String::as_str)
    }

    pub(crate) fn cache_help(&mut self, path: PathBuf, help: String) {
        self.help_cache.insert(path, help);
    }

    pub(crate) fn help_request(&self) -> Option<PathBuf> {
        self.selected_command()
            .filter(|command| !self.help_cache.contains_key(&command.path))
            .map(|command| command.path.clone())
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Option<PathBuf> {
        if key.kind != KeyEventKind::Press {
            return None;
        }

        match key.code {
            KeyCode::Esc => self.should_exit = true,
            KeyCode::Enter => {
                self.selection = self.selected_command().map(|command| command.name.clone());
                self.should_exit = self.selection.is_some();
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_exit = true;
            }
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.query.push(character);
                self.refresh_matches();
            }
            KeyCode::Backspace => {
                self.query.pop();
                self.refresh_matches();
            }
            KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Down if self.selected + 1 < self.matches.len() => self.selected += 1,
            _ => {}
        }
        (!self.should_exit).then(|| self.help_request()).flatten()
    }

    fn refresh_matches(&mut self) {
        let query = self.query.to_lowercase();
        self.matches = self
            .commands
            .iter()
            .enumerate()
            .filter_map(|(index, command)| {
                command
                    .name
                    .to_lowercase()
                    .contains(&query)
                    .then_some(index)
            })
            .collect();
        self.matches.sort_unstable_by(|left, right| {
            let left_name = self.commands[*left].name.to_lowercase();
            let right_name = self.commands[*right].name.to_lowercase();
            (left_name != query)
                .cmp(&(right_name != query))
                .then_with(|| {
                    (!left_name.starts_with(&query)).cmp(&(!right_name.starts_with(&query)))
                })
                .then_with(|| left_name.cmp(&right_name))
        });
        self.selected = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_query_and_exits() {
        let mut app = App::default();

        let _ = app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE));
        let _ = app.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        let _ = app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

        assert!(app.query.is_empty());
        assert!(app.should_exit);
    }

    #[test]
    fn enter_returns_the_selected_command() {
        let command = CommandEntry {
            name: "rg".into(),
            path: "/usr/bin/rg".into(),
        };
        let mut app = App::new(vec![command]);

        let _ = app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert_eq!(app.selection.as_deref(), Some("rg"));
        assert!(app.should_exit);
    }
}
