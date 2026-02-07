use super::app::App;
use super::events::Event;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct EventHandler;

impl EventHandler {
    pub fn handle(event: &Event, app: &mut App) -> bool {
        match event {
            Event::Key(key) => Self::handle_key(*key, app),
            Event::Mouse(_) => false,
            Event::Resize(w, h) => {
                app.update_terminal_size(*w as usize, *h as usize);
                false
            }
            Event::Tick => {
                app.on_tick();
                false
            }
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
                match app.refresh() {
                    Ok(_) => app.show_message("Refreshed ↻"),
                    Err(e) => app.show_message(format!("Refresh failed: {}", e)),
                }
                false
            }
            KeyCode::Char('d') if key.modifiers == KeyModifiers::NONE => {
                match app.delete_current_entry() {
                    Ok(true) => app.show_message("Entry deleted"),
                    Ok(false) => app.show_message("No entry to delete"),
                    Err(e) => app.show_message(format!("Delete failed: {}", e)),
                }
                false
            }
            KeyCode::Char('h') | KeyCode::Left if key.modifiers == KeyModifiers::NONE => {
                app.scroll_preview_up();
                false
            }
            KeyCode::Char('l') | KeyCode::Right if key.modifiers == KeyModifiers::NONE => {
                app.scroll_preview_down();
                false
            }
            KeyCode::PageUp => {
                for _ in 0..10 { app.scroll_preview_up(); }
                false
            }
            KeyCode::PageDown => {
                for _ in 0..10 { app.scroll_preview_down(); }
                false
            }
            KeyCode::Char('q') | KeyCode::Esc if key.modifiers == KeyModifiers::NONE => {
                if app.is_filtering || !app.filter_text.is_empty() {
                    app.stop_filtering();
                    false
                } else {
                    true
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_app() -> App {
        App::new(vec![], "/test/db".to_string(), 80, 24)
    }

    #[test]
    fn test_handle_up_key() {
        let mut app = create_test_app();
        app.selected_index = 1;
        let event = Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        EventHandler::handle(&event, &mut app);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_handle_down_key() {
        use chrono::Utc;
        let now = Utc::now();
        let entries = vec![
            crate::db::ClipboardEntry {
                content: "entry1".to_string(),
                created_at: now,
                last_copied: now,
            },
            crate::db::ClipboardEntry {
                content: "entry2".to_string(),
                created_at: now,
                last_copied: now,
            },
        ];
        let mut app = App::new(entries, "/test/db".to_string(), 80, 24);
        let event = Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        EventHandler::handle(&event, &mut app);
        assert_eq!(app.selected_index, 1);
    }

    #[test]
    fn test_filter_mode() {
        let mut app = create_test_app();
        let event = Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE));
        EventHandler::handle(&event, &mut app);
        assert!(app.is_filtering);
    }

    #[test]
    fn test_quit() {
        let mut app = create_test_app();
        let event = Event::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        let should_exit = EventHandler::handle(&event, &mut app);
        assert!(should_exit);
    }

    #[test]
    fn test_preview_scroll() {
        let mut app = create_test_app();
        let event = Event::Key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE));
        EventHandler::handle(&event, &mut app);
        assert_eq!(app.preview_scroll, 1);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE));
        EventHandler::handle(&event, &mut app);
        assert_eq!(app.preview_scroll, 0);
    }

    #[test]
    fn test_escape_filter() {
        let mut app = create_test_app();
        app.start_filtering();
        app.filter_push('t');
        assert!(app.is_filtering);

        let event = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        EventHandler::handle(&event, &mut app);
        assert!(!app.is_filtering);
        assert!(app.filter_text.is_empty());
    }
}
