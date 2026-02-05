use crate::db::ClipboardEntry;
use crate::tui::fuzzy;
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
    let time_width = 10;
    let content_start = 2;
    let content_max_width = width.saturating_sub(content_start + 1 + time_width);

    let visible_entries: Vec<Line> = entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_selected = (scroll_offset + idx) == selected_index;
            let content_preview = entry.content.replace('\n', "↵").replace('\r', "");

            let content_display = if content_preview.chars().count() > content_max_width {
                let truncated: String = content_preview
                    .chars()
                    .take(content_max_width.saturating_sub(1))
                    .collect();
                format!("{}…", truncated)
            } else {
                content_preview
            };

            let date_str = format_relative_date(&entry.last_copied);
            let selector = if is_selected { ">" } else { " " };

            if filter_text.is_empty() {
                let truncated_if_needed = if content_display.len() > content_max_width {
                    format!("{}…", &content_display[..content_max_width.saturating_sub(1)])
                } else {
                    content_display
                };

                let padded_content = format!("{:width$}", truncated_if_needed, width = content_max_width);

                let full_line = format!(
                    "{} {}{}",
                    selector,
                    padded_content,
                    format!("{:>10}", date_str)
                );

                let color = if is_selected {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };

                Line::from(full_line).patch_style(color)
            } else {
                // Fuzzy match highlighting
                let fuzzy_result = fuzzy::fuzzy_match(&content_display, filter_text);
                let mut spans: Vec<Span> = vec![Span::raw(format!("{} ", selector))];

                if fuzzy_result.matched {
                    // Build spans with highlighting for matched positions (character-based)
                    let chars: Vec<char> = content_display.chars().collect();
                    let mut last_pos = 0;

                    for (match_start, match_len) in &fuzzy_result.match_positions {
                        // Add unmatched text before this match (character-based)
                        if *match_start > last_pos {
                            let unmatched: String = chars[last_pos..*match_start].iter().collect();
                            spans.push(Span::raw(unmatched));
                        }
                        // Add matched text with highlighting (character-based)
                        let matched: String = chars[*match_start..(*match_start + match_len)].iter().collect();
                        spans.push(Span::styled(
                            matched,
                            Style::default().bg(Color::Yellow).fg(Color::Black),
                        ));
                        last_pos = *match_start + match_len;
                    }
                    // Add remaining unmatched text (character-based)
                    if last_pos < chars.len() {
                        let remaining: String = chars[last_pos..].iter().collect();
                        spans.push(Span::raw(remaining));
                    }
                } else {
                    // No match (shouldn't happen if filter is applied correctly, but be safe)
                    spans.push(Span::raw(content_display));
                }

                let current_content_len: usize = spans.iter().map(|s| s.content.len()).sum();
                let padding_needed = content_max_width.saturating_sub(current_content_len - 2);

                if padding_needed > 0 {
                    spans.push(Span::raw(" ".repeat(padding_needed)));
                }

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

/// Preview panel component - simplified
pub fn draw_preview(f: &mut Frame, area: Rect, entry: Option<&ClipboardEntry>, filter_text: &str) {
    let content = if let Some(e) = entry {
        let mut lines = vec![];

        let timestamp = format_absolute_date(&e.created_at);
        lines.push(Line::from(Span::styled(
            format!("─ {}", timestamp),
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(""));

        for content_line in e.content.lines() {
            if filter_text.is_empty() {
                lines.push(Line::from(content_line));
            } else {
                let mut spans = vec![];
                let line_lower = content_line.to_lowercase();
                let filter_lower = filter_text.to_lowercase();

                let mut last_pos = 0;
                for (match_idx, _) in line_lower.match_indices(&filter_lower) {
                    if match_idx > last_pos {
                        spans.push(Span::raw(content_line[last_pos..match_idx].to_string()));
                    }
                    spans.push(Span::styled(
                        content_line[match_idx..match_idx + filter_lower.len()].to_string(),
                        Style::default().bg(Color::Yellow).fg(Color::Black),
                    ));
                    last_pos = match_idx + filter_lower.len();
                }
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
            Span::styled("🔍 ", Style::default().fg(Color::Cyan)),
            Span::styled("Filter: ", Style::default().fg(Color::Yellow).bold()),
            Span::raw(filter_text),
            Span::styled("_", Style::default().fg(Color::Yellow)),
            Span::styled("  ⏎ ", Style::default().fg(Color::Green)),
            Span::styled("confirm", Style::default().fg(Color::Green).dim()),
            Span::styled("  ⎋ ", Style::default().fg(Color::Red)),
            Span::styled("cancel", Style::default().fg(Color::Red).dim()),
        ])
    } else {
        Line::from(vec![
            Span::styled("⏎", Style::default().fg(Color::Green).bold()),
            Span::raw(" copy  "),
            Span::styled("─", Style::default().fg(Color::Gray)),
            Span::raw(" "),
            Span::styled("/", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" filter  "),
            Span::styled("─", Style::default().fg(Color::Gray)),
            Span::raw(" "),
            Span::styled("r", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" refresh  "),
            Span::styled("─", Style::default().fg(Color::Gray)),
            Span::raw(" "),
            Span::styled("q", Style::default().fg(Color::Magenta).bold()),
            Span::raw(" quit  "),
            Span::styled("┃", Style::default().fg(Color::Gray)),
            Span::raw(" "),
            Span::styled(format!("📂 {}", db_path_short), Style::default().fg(Color::Gray).dim()),
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
