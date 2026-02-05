use crate::db::ClipboardEntry;
use chrono::{DateTime, Local, Utc};
use ratatui::{
    prelude::*,
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

/// Header component
pub fn draw_header(f: &mut Frame, area: Rect, title: &str, subtitle: &str, loading: bool) {
    let display_subtitle = if loading { "Loading..." } else { subtitle };

    let header_text = if display_subtitle.is_empty() {
        Line::from(vec![
            Span::styled("clipboard", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" - "),
            Span::styled(title, Style::default().bold()),
        ])
    } else {
        Line::from(vec![
            Span::styled("clipboard", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" - "),
            Span::styled(title, Style::default().bold()),
            Span::raw(" ("),
            Span::styled(display_subtitle, Style::default().fg(Color::Gray)),
            Span::raw(")"),
        ])
    };

    let divider = "─".repeat(area.width as usize);

    let lines = vec![header_text, Line::from(Span::styled(divider, Style::default().fg(Color::Gray)))];

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, area);
}

/// Entry list component with search highlighting
pub fn draw_entry_list(
    f: &mut Frame,
    area: Rect,
    entries: Vec<&ClipboardEntry>,
    selected_index: usize,
    scroll_offset: usize,
    filter_text: &str,
) {
    let width = area.width as usize;
    let time_width = 12; // Width for time column on the right
    let content_width = width.saturating_sub(3 + time_width); // 3 for selector and spacing

    let visible_entries: Vec<Line> = entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_selected = (scroll_offset + idx) == selected_index;
            let content_preview = entry.content.replace('\n', "↵").replace('\r', "");

            // Truncate content to available width
            let content_display = if content_preview.len() > content_width {
                format!("{}…", &content_preview[..content_width.saturating_sub(1)])
            } else {
                content_preview
            };

            let date_str = format_relative_date(&entry.last_copied);

            let selector = if is_selected { ">" } else { " " };

            // Build the line with proper spacing and highlighting
            let mut line_text = format!("{} ", selector);

            if filter_text.is_empty() {
                line_text.push_str(&content_display);
            } else {
                // Build highlighted content
                let content_lower = content_display.to_lowercase();
                let filter_lower = filter_text.to_lowercase();

                let mut last_pos = 0;
                let mut highlighted = String::new();

                for (match_idx, _) in content_lower.match_indices(&filter_lower) {
                    // Add non-matching part
                    if match_idx > last_pos {
                        highlighted.push_str(&content_display[last_pos..match_idx]);
                    }
                    // Mark highlighted part with markers (will be styled separately)
                    highlighted.push('█'); // Visual marker for highlight start
                    highlighted.push_str(&content_display[match_idx..match_idx + filter_lower.len()]);
                    highlighted.push('█'); // Visual marker for highlight end
                    last_pos = match_idx + filter_lower.len();
                }
                // Add remaining part
                if last_pos < content_display.len() {
                    highlighted.push_str(&content_display[last_pos..]);
                }

                line_text = format!("{} {}", selector, build_highlighted_line(&content_display, filter_text, is_selected));
            }

            if filter_text.is_empty() {
                // Simple case without filtering
                let padding_len: usize = content_width.saturating_sub(content_display.len() + 2);
                let full_line = format!(
                    "{} {}{}{}",
                    selector,
                    content_display,
                    " ".repeat(padding_len),
                    format!("{:>10}", date_str)
                );

                let color = if is_selected {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };

                Line::from(full_line).patch_style(color)
            } else {
                // Build line with highlighting
                let mut spans: Vec<Span> = vec![Span::raw(format!("{} ", selector))];

                let content_lower = content_display.to_lowercase();
                let filter_lower = filter_text.to_lowercase();

                let mut last_pos = 0;
                for (match_idx, _) in content_lower.match_indices(&filter_lower) {
                    // Add non-matching part
                    if match_idx > last_pos {
                        spans.push(Span::raw(content_display[last_pos..match_idx].to_string()));
                    }
                    // Add highlighted matching part
                    spans.push(Span::styled(
                        content_display[match_idx..match_idx + filter_lower.len()].to_string(),
                        Style::default().bg(Color::Yellow).fg(Color::Black),
                    ));
                    last_pos = match_idx + filter_lower.len();
                }
                // Add remaining part
                if last_pos < content_display.len() {
                    spans.push(Span::raw(content_display[last_pos..].to_string()));
                }

                // Add padding
                let current_len: usize = spans.iter().map(|s| s.content.len()).sum::<usize>() + 1;
                let padding_needed = content_width.saturating_sub(current_len);
                if padding_needed > 0 {
                    spans.push(Span::raw(" ".repeat(padding_needed)));
                }

                // Add time
                spans.push(Span::styled(
                    format!("{:>10}", date_str),
                    Style::default().fg(Color::Gray),
                ));

                let color = if is_selected {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };

                Line::from(spans).patch_style(color)
            }
        })
        .collect();

    if visible_entries.is_empty() {
        let message = if entries.is_empty() {
            "No clipboard history found."
        } else {
            "No matches."
        };

        let paragraph = Paragraph::new(message).style(Style::default().fg(Color::Gray));
        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new(visible_entries).block(Block::default());
        f.render_widget(paragraph, area);
    }
}

/// Helper to build a highlighted line (used for complex highlighting)
fn build_highlighted_line(content: &str, filter_text: &str, _is_selected: bool) -> String {
    content.to_string()
}

/// Preview panel component - simplified
pub fn draw_preview(f: &mut Frame, area: Rect, entry: Option<&ClipboardEntry>, filter_text: &str) {
    let content = if let Some(e) = entry {
        let mut lines = vec![];

        // Add timestamp header
        let timestamp = format_absolute_date(&e.created_at);
        lines.push(Line::from(Span::styled(
            format!("─ {}", timestamp),
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(""));

        // Add content with search highlighting
        for content_line in e.content.lines() {
            if filter_text.is_empty() {
                lines.push(Line::from(content_line));
            } else {
                // Build highlighted line
                let mut spans = vec![];
                let line_lower = content_line.to_lowercase();
                let filter_lower = filter_text.to_lowercase();

                let mut last_pos = 0;
                for (match_idx, _) in line_lower.match_indices(&filter_lower) {
                    // Add non-matching part
                    if match_idx > last_pos {
                        spans.push(Span::raw(content_line[last_pos..match_idx].to_string()));
                    }
                    // Add highlighted matching part
                    spans.push(Span::styled(
                        content_line[match_idx..match_idx + filter_lower.len()].to_string(),
                        Style::default().bg(Color::Yellow).fg(Color::Black),
                    ));
                    last_pos = match_idx + filter_lower.len();
                }
                // Add remaining part
                if last_pos < content_line.len() {
                    spans.push(Span::raw(content_line[last_pos..].to_string()));
                }

                if spans.is_empty() {
                    lines.push(Line::from(content_line));
                } else {
                    lines.push(Line::from(spans));
                }
            }
        }

        lines
    } else {
        vec![Line::from(Span::styled(
            "No entry selected",
            Style::default().fg(Color::Gray),
        ))]
    };

    let paragraph = Paragraph::new(content);
    f.render_widget(paragraph, area);
}

/// Status bar component
pub fn draw_status_bar(
    f: &mut Frame,
    area: Rect,
    is_filtering: bool,
    filter_text: &str,
    db_path_short: &str,
) {
    let content = if is_filtering {
        Line::from(vec![
            Span::styled("Filter: ", Style::default().fg(Color::Yellow)),
            Span::raw(filter_text),
            Span::styled("_", Style::default().fg(Color::Gray)),
            Span::styled("  (Enter to confirm, Esc to cancel)", Style::default().fg(Color::Gray)),
        ])
    } else {
        Line::from(vec![
            Span::styled("[Enter]", Style::default().bold()),
            Span::raw(" copy "),
            Span::styled("[/]", Style::default().bold()),
            Span::raw(" filter "),
            Span::styled("[r]", Style::default().bold()),
            Span::raw(" refresh "),
            Span::styled("[q]", Style::default().bold()),
            Span::raw(" quit "),
            Span::raw(" | "),
            Span::styled(format!("DB: {}", db_path_short), Style::default().fg(Color::Gray)),
        ])
    };

    let paragraph = Paragraph::new(content);
    f.render_widget(paragraph, area);
}

/// Format relative date (e.g., "2m ago", "1h ago")
fn format_relative_date(date: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*date);

    if duration.num_seconds() < 60 {
        "now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{}d ago", duration.num_days())
    } else if duration.num_weeks() < 5 {
        format!("{}w ago", duration.num_weeks())
    } else {
        format!("{}mo ago", duration.num_days() / 30)
    }
}

/// Format absolute date (e.g., "January 1st at 14:30")
fn format_absolute_date(date: &DateTime<Utc>) -> String {
    let local = date.with_timezone(&Local);
    local.format("%b %d at %H:%M").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_relative_date_now() {
        let date = Utc::now();
        let formatted = format_relative_date(&date);
        assert_eq!(formatted, "now");
    }

    #[test]
    fn test_format_relative_date_minutes_ago() {
        let date = Utc::now() - chrono::Duration::minutes(5);
        let formatted = format_relative_date(&date);
        assert_eq!(formatted, "5m ago");
    }
}
