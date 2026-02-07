use crate::db::{ClipboardEntry, Database};
use crate::tui::fuzzy;

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteMode {
    /// Not in delete mode
    None,
    /// Selecting time period for bulk delete
    SelectingPeriod,
    /// Confirming bulk delete
    ConfirmingBulk { period: DeletePeriod },
    /// Confirming single entry delete
    ConfirmingSingle,
    /// Confirming "all" deletion (tracks confirmation count)
    ConfirmingAll { confirmation_count: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeletePeriod {
    Hour,
    Day,
    Week,
    Month,
    Year,
    All,
}

impl DeletePeriod {
    pub fn to_days(&self) -> Option<i64> {
        match self {
            Self::Hour => Some(1),
            Self::Day => Some(1),
            Self::Week => Some(7),
            Self::Month => Some(30),
            Self::Year => Some(365),
            Self::All => None,
        }
    }

    pub fn display(&self) -> &str {
        match self {
            Self::Hour => "Last Hour",
            Self::Day => "Last Day",
            Self::Week => "Last Week",
            Self::Month => "Last Month",
            Self::Year => "Last Year",
            Self::All => "ALL ENTRIES",
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub entries: Vec<ClipboardEntry>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub filter_text: String,
    pub is_filtering: bool,
    pub message: Option<String>,
    pub loading: bool,
    pub selected_entry: Option<String>,
    pub terminal_width: usize,
    pub terminal_height: usize,
    pub db_path: String,
    pub preview_scroll: usize,
    tick_count: usize,
    /// Delete mode state
    pub delete_mode: DeleteMode,
    /// Selected period index (for period selection popup)
    pub delete_period_index: usize,
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
            preview_scroll: 0,
            tick_count: 0,
            delete_mode: DeleteMode::None,
            delete_period_index: 0,
        }
    }

    pub fn filtered_entries(&self) -> Vec<&ClipboardEntry> {
        if self.filter_text.is_empty() {
            self.entries.iter().collect()
        } else {
            let mut filtered: Vec<(usize, &ClipboardEntry)> = self.entries
                .iter()
                .enumerate()
                .filter_map(|(idx, e)| {
                    let result = fuzzy::fuzzy_match(&e.content, &self.filter_text);
                    if result.matched { Some((idx, e)) } else { None }
                })
                .collect();

            filtered.sort_by(|a, b| {
                let a_exact = fuzzy::fuzzy_match(&a.1.content, &self.filter_text).is_exact;
                let b_exact = fuzzy::fuzzy_match(&b.1.content, &self.filter_text).is_exact;
                match (a_exact, b_exact) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                }
            });

            filtered.into_iter().map(|(_, e)| e).collect()
        }
    }

    pub fn current_entry(&self) -> Option<&ClipboardEntry> {
        self.filtered_entries().get(self.selected_index).copied()
    }

    pub fn select_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.preview_scroll = 0;
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
        }
    }

    pub fn select_down(&mut self) {
        let filtered = self.filtered_entries();
        if self.selected_index < filtered.len().saturating_sub(1) {
            self.selected_index += 1;
            self.preview_scroll = 0;
            let usable_height = self.get_list_height();
            if self.selected_index >= self.scroll_offset + usable_height {
                self.scroll_offset = self.selected_index - usable_height + 1;
            }
        }
    }

    pub fn start_filtering(&mut self) {
        self.is_filtering = true;
        self.filter_text.clear();
        self.reset_selection();
    }

    pub fn stop_filtering(&mut self) {
        self.is_filtering = false;
        self.filter_text.clear();
        self.reset_selection();
    }

    pub fn filter_push(&mut self, ch: char) {
        self.filter_text.push(ch);
        self.reset_selection();
    }

    pub fn filter_pop(&mut self) {
        self.filter_text.pop();
        self.reset_selection();
    }

    pub fn confirm_filter(&mut self) {
        self.is_filtering = false;
    }

    fn reset_selection(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.preview_scroll = 0;
    }

    pub fn select_entry(&mut self) -> Option<String> {
        if let Some(entry) = self.current_entry() {
            let content = entry.content.clone();
            self.selected_entry = Some(content.clone());
            return Some(content);
        }
        None
    }

    pub fn get_list_height(&self) -> usize {
        self.terminal_height.saturating_sub(4)
    }

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

    pub fn get_entry_count_info(&self) -> String {
        let count = self.filtered_entries().len();
        let total = self.entries.len();
        if self.filter_text.is_empty() {
            format!("{} entries", count)
        } else {
            format!("{} entries, {} matches", total, count)
        }
    }

    pub fn show_message(&mut self, msg: impl Into<String>) {
        self.message = Some(msg.into());
    }

    pub fn update_terminal_size(&mut self, width: usize, height: usize) {
        self.terminal_width = width;
        self.terminal_height = height;
    }

    pub fn get_db_path_short(&self) -> String {
        self.db_path.clone()
    }

    pub fn refresh(&mut self) -> crate::error::Result<()> {
        let db = Database::open(&self.db_path)?;
        let new_entries = db.get_all_entries()?;

        let changed = new_entries.len() != self.entries.len()
            || new_entries.iter().zip(&self.entries).any(|(a, b)| {
                a.content != b.content || a.last_copied != b.last_copied
            });

        if changed {
            self.entries = new_entries;
            self.selected_index = 0;
            self.scroll_offset = 0;
        }

        Ok(())
    }

    pub fn on_tick(&mut self) {
        self.tick_count += 1;
        if self.tick_count >= 50 {
            self.tick_count = 0;
            let _ = self.refresh();
        }
    }

    pub fn delete_current_entry(&mut self) -> crate::error::Result<bool> {
        if let Some(entry) = self.current_entry() {
            let content = entry.content.clone();
            let db = Database::open(&self.db_path)?;
            if db.delete_entry_by_content(&content)? {
                self.entries.retain(|e| e.content != content);
                let filtered_len = self.filtered_entries().len();
                if self.selected_index >= filtered_len && filtered_len > 0 {
                    self.selected_index = filtered_len - 1;
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
    }

    pub fn scroll_preview_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(1);
    }

    #[allow(dead_code)]
    pub fn reset_preview_scroll(&mut self) {
        self.preview_scroll = 0;
    }

    #[allow(dead_code)]
    pub fn get_preview_height(&self) -> usize {
        self.terminal_height.saturating_sub(4)
    }

    pub fn start_bulk_delete(&mut self) {
        self.delete_mode = DeleteMode::SelectingPeriod;
        self.delete_period_index = 0;
    }

    pub fn start_single_delete(&mut self) {
        if self.current_entry().is_some() {
            self.delete_mode = DeleteMode::ConfirmingSingle;
        }
    }

    pub fn cancel_delete(&mut self) {
        self.delete_mode = DeleteMode::None;
        self.delete_period_index = 0;
    }

    pub fn delete_period_up(&mut self) {
        if self.delete_period_index > 0 {
            self.delete_period_index -= 1;
        }
    }

    pub fn delete_period_down(&mut self) {
        let max = 5;
        if self.delete_period_index < max {
            self.delete_period_index += 1;
        }
    }

    pub fn confirm_delete_period(&mut self) {
        let period = match self.delete_period_index {
            0 => DeletePeriod::Hour,
            1 => DeletePeriod::Day,
            2 => DeletePeriod::Week,
            3 => DeletePeriod::Month,
            4 => DeletePeriod::Year,
            5 => DeletePeriod::All,
            _ => DeletePeriod::Day,
        };

        if period == DeletePeriod::All {
            self.delete_mode = DeleteMode::ConfirmingAll { confirmation_count: 0 };
        } else {
            self.delete_mode = DeleteMode::ConfirmingBulk { period };
        }
    }

    pub fn is_in_delete_mode(&self) -> bool {
        self.delete_mode != DeleteMode::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_entry(content: &str) -> ClipboardEntry {
        ClipboardEntry {
            id: 1,
            content: content.to_string(),
            created_at: Utc::now(),
            last_copied: Utc::now(),
        }
    }

    #[test]
    fn test_app_creation() {
        let app = App::new(vec![], "/test/db".to_string(), 80, 24);
        assert_eq!(app.entries.len(), 0);
        assert_eq!(app.selected_index, 0);
        assert_eq!(app.preview_scroll, 0);
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

    #[test]
    fn test_select_up_down() {
        let entries = vec![
            create_test_entry("one"),
            create_test_entry("two"),
            create_test_entry("three"),
        ];
        let mut app = App::new(entries, "/test/db".to_string(), 80, 24);

        assert_eq!(app.selected_index, 0);
        app.select_down();
        assert_eq!(app.selected_index, 1);
        app.select_down();
        assert_eq!(app.selected_index, 2);
        app.select_down();
        assert_eq!(app.selected_index, 2);

        app.select_up();
        assert_eq!(app.selected_index, 1);
        app.select_up();
        assert_eq!(app.selected_index, 0);
        app.select_up();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_filtering_mode() {
        let mut app = App::new(vec![], "/test/db".to_string(), 80, 24);
        assert!(!app.is_filtering);

        app.start_filtering();
        assert!(app.is_filtering);
        assert!(app.filter_text.is_empty());

        app.filter_push('t');
        app.confirm_filter();
        assert!(!app.is_filtering);
        assert_eq!(app.filter_text, "t");

        app.stop_filtering();
        assert!(app.filter_text.is_empty());
    }

    #[test]
    fn test_preview_scroll() {
        let mut app = App::new(vec![], "/test/db".to_string(), 80, 24);
        assert_eq!(app.preview_scroll, 0);

        app.scroll_preview_down();
        assert_eq!(app.preview_scroll, 1);
        app.scroll_preview_down();
        assert_eq!(app.preview_scroll, 2);

        app.scroll_preview_up();
        assert_eq!(app.preview_scroll, 1);
        app.scroll_preview_up();
        assert_eq!(app.preview_scroll, 0);
        app.scroll_preview_up();
        assert_eq!(app.preview_scroll, 0);

        app.preview_scroll = 5;
        app.reset_preview_scroll();
        assert_eq!(app.preview_scroll, 0);
    }

    #[test]
    fn test_get_list_height() {
        let app = App::new(vec![], "/test/db".to_string(), 80, 24);
        assert_eq!(app.get_list_height(), 20);
    }

    #[test]
    fn test_entry_count_info() {
        let entries = vec![
            create_test_entry("hello"),
            create_test_entry("world"),
        ];
        let mut app = App::new(entries, "/test/db".to_string(), 80, 24);
        assert_eq!(app.get_entry_count_info(), "2 entries");

        app.filter_text = "hello".to_string();
        assert_eq!(app.get_entry_count_info(), "2 entries, 1 matches");
    }
}
