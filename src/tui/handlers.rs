use super::app::{App, DeleteMode, DeletePeriod};
use super::events::Event;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::db::Database;

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
            Event::Tick => {
                app.on_tick();
                false
            }
        }
    }

    fn handle_key(key: KeyEvent, app: &mut App) -> bool {
        if app.is_in_delete_mode() {
            return Self::handle_delete_mode(key, app);
        }

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
                    Ok(_) => {
                        app.show_message("Refreshed ↻");
                    }
                    Err(e) => {
                        app.show_message(format!("Refresh failed: {}", e));
                    }
                }
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
            KeyCode::Char('x') if key.modifiers == KeyModifiers::NONE => {
                app.start_single_delete();
                false
            }
            KeyCode::Delete if key.modifiers == KeyModifiers::NONE => {
                app.start_single_delete();
                false
            }
            KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
                app.start_bulk_delete();
                false
            }
            KeyCode::Char('D') if key.modifiers == KeyModifiers::SHIFT => {
                app.start_bulk_delete();
                false
            }
            _ => false,
        }
    }

    fn handle_delete_mode(key: KeyEvent, app: &mut App) -> bool {
        match &app.delete_mode.clone() {
            DeleteMode::SelectingPeriod => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') if key.modifiers == KeyModifiers::NONE => {
                        app.delete_period_up();
                        false
                    }
                    KeyCode::Down | KeyCode::Char('j') if key.modifiers == KeyModifiers::NONE => {
                        app.delete_period_down();
                        false
                    }
                    KeyCode::Enter => {
                        app.confirm_delete_period();
                        false
                    }
                    KeyCode::Esc | KeyCode::Char('q') if key.modifiers == KeyModifiers::NONE => {
                        app.cancel_delete();
                        false
                    }
                    _ => false
                }
            }

            DeleteMode::ConfirmingSingle => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        Self::perform_single_delete(app);
                        false
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        app.cancel_delete();
                        false
                    }
                    _ => false
                }
            }

            DeleteMode::ConfirmingBulk { period } => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        Self::perform_bulk_delete(app, *period);
                        false
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        app.cancel_delete();
                        false
                    }
                    _ => false
                }
            }

            DeleteMode::ConfirmingAll { confirmation_count } => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        if *confirmation_count >= 2 {
                            // Third confirmation - actually delete
                            Self::perform_delete_all(app);
                        } else {
                            // Increment confirmation count
                            app.delete_mode = DeleteMode::ConfirmingAll {
                                confirmation_count: confirmation_count + 1,
                            };
                        }
                        false
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        app.cancel_delete();
                        false
                    }
                    _ => false
                }
            }

            DeleteMode::None => false,
        }
    }

    fn perform_single_delete(app: &mut App) {
        if let Some(entry) = app.current_entry() {
            let entry_id = entry.id;

            match Database::open(&app.db_path) {
                Ok(db) => {
                    match db.delete_entry_by_id(entry_id) {
                        Ok(true) => {
                            app.show_message("Entry deleted ✓");
                            // Refresh entries
                            let _ = app.refresh();
                        }
                        Ok(false) => {
                            app.show_message("Entry not found");
                        }
                        Err(e) => {
                            app.show_message(format!("Delete failed: {}", e));
                        }
                    }
                }
                Err(e) => {
                    app.show_message(format!("Database error: {}", e));
                }
            }
        }

        app.cancel_delete();
    }

    fn perform_bulk_delete(app: &mut App, period: DeletePeriod) {
        match Database::open(&app.db_path) {
            Ok(db) => {
                let result = match period {
                    DeletePeriod::Hour => db.delete_entries_from_last_hours(1),
                    DeletePeriod::Day => db.delete_entries_from_last_days(1),
                    DeletePeriod::Week => db.delete_entries_from_last_days(7),
                    DeletePeriod::Month => db.delete_entries_from_last_days(30),
                    DeletePeriod::Year => db.delete_entries_from_last_days(365),
                    DeletePeriod::All => {
                        // Should not reach here - All goes through ConfirmingAll
                        app.show_message("Error: Use delete all confirmation");
                        app.cancel_delete();
                        return;
                    }
                };

                match result {
                    Ok(count) => {
                        app.show_message(format!("Deleted {} entries ✓", count));
                        let _ = app.refresh();
                    }
                    Err(e) => {
                        app.show_message(format!("Delete failed: {}", e));
                    }
                }
            }
            Err(e) => {
                app.show_message(format!("Database error: {}", e));
            }
        }

        app.cancel_delete();
    }

    fn perform_delete_all(app: &mut App) {
        match Database::open(&app.db_path) {
            Ok(db) => {
                match db.clear_all() {
                    Ok(count) => {
                        app.show_message(format!("Deleted ALL {} entries ✓", count));
                        let _ = app.refresh();
                    }
                    Err(e) => {
                        app.show_message(format!("Delete failed: {}", e));
                    }
                }
            }
            Err(e) => {
                app.show_message(format!("Database error: {}", e));
            }
        }

        app.cancel_delete();
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
