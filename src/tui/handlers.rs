use super::app::App;
use super::events::Event;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct EventHandler;

impl EventHandler {
    /// Handle an event and update app state
    /// Returns true if the application should exit
    pub fn handle(event: &Event, app: &mut App) -> bool {
        match event {
            Event::Key(key) => Self::handle_key(*key, app),
            Event::Mouse(mouse) => Self::handle_mouse(*mouse, app),
            Event::Resize(w, h) => {
                app.update_terminal_size(*w as usize, *h as usize);
                false
            }
            Event::Tick => false,
        }
    }

    fn handle_key(key: KeyEvent, app: &mut App) -> bool {
        if app.is_filtering {
            return Self::handle_filter_mode(key, app);
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') if key.modifiers == KeyModifiers::NONE => {
                app.select_up();
                false
            }
            KeyCode::Down | KeyCode::Char('j') if key.modifiers == KeyModifiers::NONE => {
                app.select_down();
                false
            }
            KeyCode::Enter => {
                app.select_entry();
                true
            }
            KeyCode::Char('/') if key.modifiers == KeyModifiers::NONE => {
                app.start_filtering();
                false
            }
            KeyCode::Char('r') if key.modifiers == KeyModifiers::NONE => {
                app.show_message("Refresh not yet implemented");
                false
            }
            KeyCode::Char('q') if key.modifiers == KeyModifiers::NONE => {
                if app.filter_text.is_empty() {
                    true
                } else {
                    app.stop_filtering();
                    false
                }
            }
            KeyCode::Esc => {
                if app.filter_text.is_empty() {
                    true
                } else {
                    app.stop_filtering();
                    false
                }
            }
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => true,
            _ => false,
        }
    }

    fn handle_filter_mode(key: KeyEvent, app: &mut App) -> bool {
        match key.code {
            KeyCode::Esc => {
                app.stop_filtering();
                false
            }
            KeyCode::Enter => {
                app.confirm_filter();
                false
            }
            KeyCode::Backspace | KeyCode::Delete => {
                app.filter_pop();
                false
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
                app.filter_push(c);
                false
            }
            _ => false,
        }
    }

    fn handle_mouse(_mouse: crossterm::event::MouseEvent, _app: &mut App) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ClipboardEntry;
    use chrono::Utc;

    fn create_test_app() -> App {
        App::new(vec![], "/test/db".to_string(), 80, 24)
    }

    #[test]
    fn test_handle_up_key() {
        let mut app = create_test_app();
        app.selected_index = 1;
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let event = Event::Key(key);
        EventHandler::handle(&event, &mut app);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_handle_down_key() {
        let mut app = create_test_app();
        app.selected_index = 0;
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let event = Event::Key(key);
        EventHandler::handle(&event, &mut app);
        assert_eq!(app.selected_index, 1);
    }
}
