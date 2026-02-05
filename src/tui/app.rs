use crate::db::ClipboardEntry;
use crate::error::Result;
use chrono::Utc;

#[derive(Debug)]
pub struct App {
    /// All clipboard entries from database
    pub entries: Vec<ClipboardEntry>,
    /// Currently selected entry index (in filtered list)
    pub selected_index: usize,
    /// Scroll offset (in filtered list)
    pub scroll_offset: usize,
    /// Filter text for searching
    pub filter_text: String,
    /// Whether currently filtering
    pub is_filtering: bool,
    /// Temporary message to display
    pub message: Option<String>,
    /// Whether data is still loading
    pub loading: bool,
    /// Selected entry to return (on exit)
    pub selected_entry: Option<String>,
    /// Terminal dimensions
    pub terminal_width: usize,
    pub terminal_height: usize,
    /// Database path (for display)
    pub db_path: String,
}

impl App {
    pub fn new(
        entries: Vec<ClipboardEntry>,
        db_path: String,
        terminal_width: usize,
        terminal_height: usize,
    ) -> Self {
        App {
            entries,
            selected_index: 0,
            scroll_offset: 0,
            filter_text: String::new(),
            is_filtering: false,
            message: None,
            loading: false,
            selected_entry: None,
            terminal_width,
            terminal_height,
            db_path,
        }
    }

    /// Get filtered entries based on current filter text
    pub fn filtered_entries(&self) -> Vec<&ClipboardEntry> {
        if self.filter_text.is_empty() {
            self.entries.iter().collect()
        } else {
            self.entries
                .iter()
                .filter(|e| e.content.to_lowercase().contains(&self.filter_text.to_lowercase()))
                .collect()
        }
    }

    /// Get the currently selected entry
    pub fn current_entry(&self) -> Option<&ClipboardEntry> {
        let filtered = self.filtered_entries();
        filtered.get(self.selected_index).copied()
    }

    /// Move selection up
    pub fn select_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
        }
    }

    /// Move selection down
    pub fn select_down(&mut self) {
        let filtered = self.filtered_entries();
        if self.selected_index < filtered.len().saturating_sub(1) {
            self.selected_index += 1;
            let usable_height = self.get_list_height();
            if self.selected_index >= self.scroll_offset + usable_height {
                self.scroll_offset = self.selected_index - usable_height + 1;
            }
        }
    }

    /// Start filtering mode
    pub fn start_filtering(&mut self) {
        self.is_filtering = true;
        self.filter_text.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Stop filtering mode
    pub fn stop_filtering(&mut self) {
        self.is_filtering = false;
        self.filter_text.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Add character to filter
    pub fn filter_push(&mut self, ch: char) {
        self.filter_text.push(ch);
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Remove character from filter
    pub fn filter_pop(&mut self) {
        self.filter_text.pop();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Confirm filtering
    pub fn confirm_filter(&mut self) {
        self.is_filtering = false;
    }

    /// Select the current entry
    pub fn select_entry(&mut self) -> Option<String> {
        let content = self.current_entry().map(|entry| entry.content.clone());
        if let Some(ref c) = content {
            self.selected_entry = Some(c.clone());
        }
        content
    }

    /// Get the height available for the list
    pub fn get_list_height(&self) -> usize {
        // Header: 2 lines (title + separator)
        // Status bar: 1 line
        // Spacing: 1 line
        // Total reserved: 4 lines
        let reserved = 4;
        self.terminal_height.saturating_sub(reserved)
    }

    /// Get visible entries for rendering
    pub fn get_visible_entries(&self) -> Vec<&ClipboardEntry> {
        let filtered = self.filtered_entries();
        let list_height = self.get_list_height();
        let end = (self.scroll_offset + list_height).min(filtered.len());

        if self.scroll_offset >= filtered.len() {
            vec![]
        } else {
            filtered[self.scroll_offset..end].to_vec()
        }
    }

    /// Get entry count info
    pub fn get_entry_count_info(&self) -> String {
        let filtered = self.filtered_entries();
        let count = filtered.len();
        let total = self.entries.len();

        if self.filter_text.is_empty() {
            format!("{} entries", count)
        } else {
            format!("{} entries, {} matches", total, count)
        }
    }

    /// Refresh entries from database (to be called when data loads)
    pub fn set_entries(&mut self, entries: Vec<ClipboardEntry>) {
        self.entries = entries;
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.message = Some("Refreshed".to_string());
    }

    /// Show a message
    pub fn show_message(&mut self, msg: impl Into<String>) {
        self.message = Some(msg.into());
    }

    /// Clear the message
    pub fn clear_message(&mut self) {
        self.message = None;
    }

    /// Update terminal dimensions
    pub fn update_terminal_size(&mut self, width: usize, height: usize) {
        self.terminal_width = width;
        self.terminal_height = height;
    }

    /// Get the short database path for display
    pub fn get_db_path_short(&self) -> String {
        let parts: Vec<&str> = self.db_path.split('/').collect();
        if parts.len() >= 2 {
            format!("{}/{}", parts[parts.len() - 2], parts[parts.len() - 1])
        } else {
            self.db_path.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new(vec![], "/test/db".to_string(), 80, 24);
        assert_eq!(app.entries.len(), 0);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_filter_text() {
        let mut app = App::new(vec![], "/test/db".to_string(), 80, 24);
        app.filter_push('t');
        assert_eq!(app.filter_text, "t");
        app.filter_push('e');
        assert_eq!(app.filter_text, "te");
        app.filter_pop();
        assert_eq!(app.filter_text, "t");
    }
}
