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

/// Entry list component
pub fn draw_entry_list(
    f: &mut Frame,
    area: Rect,
    entries: Vec<&ClipboardEntry>,
    selected_index: usize,
    scroll_offset: usize,
    _filter_text: &str,
) {
    let visible_entries: Vec<Line> = entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_selected = (scroll_offset + idx) == selected_index;
            let content_preview = entry.content.replace('\n', "↵").replace('\r', "");
            let content_truncated = if content_preview.len() > 50 {
                format!("{}…", &content_preview[..49])
            } else {
                content_preview
            };

            let date_str = format_relative_date(&entry.last_copied);
            let date_padded = format!("{:>8}", date_str);

            let selector = if is_selected { ">" } else { " " };
            let padding_len: usize = 50_usize.saturating_sub(content_truncated.len());

            let line_text = format!(
                "{} {}{}{}",
                selector,
                content_truncated,
                " ".repeat(padding_len),
                date_padded
            );

            let color = if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };

            Line::from(line_text).patch_style(color)
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

/// Preview panel component
pub fn draw_preview(f: &mut Frame, area: Rect, entry: Option<&ClipboardEntry>) {
    let content = if let Some(e) = entry {
        let header = format!("─ Entry #{} ", e.id);
        let timestamp = format_absolute_date(&e.created_at);
        let copy_count = format!("Copied {} times", e.copy_count);

        let mut lines = vec![
            Line::from(Span::styled(header, Style::default().fg(Color::Gray))),
            Line::from(Span::styled(timestamp, Style::default().fg(Color::Gray))),
            Line::from(Span::styled(copy_count, Style::default().fg(Color::Gray))),
            Line::from(""),
        ];

        // Add content with word wrapping
        for line in e.content.lines() {
            lines.push(Line::from(line));
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
    local.format("%B %-d at %H:%M").to_string()
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
