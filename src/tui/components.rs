use crate::db::ClipboardEntry;
use crate::tui::fuzzy;
use chrono::{DateTime, Local, Utc};
use ratatui::{
    prelude::*,
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Clear, Paragraph},
    layout::{Alignment, Margin},
};
use crate::tui::app::DeletePeriod;

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
                // content_display is already truncated above, just pad it
                let padded_content = format!("{:width$}", content_display, width = content_max_width);

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

/// Preview panel component - with text wrapping
pub fn draw_preview(f: &mut Frame, area: Rect, entry: Option<&ClipboardEntry>, filter_text: &str) {
    let width = area.width as usize;
    let content = if let Some(e) = entry {
        let mut lines = vec![];

        let timestamp = format_absolute_date(&e.created_at);
        lines.push(Line::from(Span::styled(
            format!("─ {}", timestamp),
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(""));

        for content_line in e.content.lines() {
            // Wrap long lines based on available width
            let wrapped_lines = wrap_text(content_line, width);

            for wrapped_line in wrapped_lines {
                if filter_text.is_empty() {
                    lines.push(Line::from(wrapped_line));
                } else {
                    let mut spans = vec![];
                    let line_lower = wrapped_line.to_lowercase();
                    let filter_lower = filter_text.to_lowercase();

                    let mut last_pos = 0;
                    for (match_idx, _) in line_lower.match_indices(&filter_lower) {
                        if match_idx > last_pos {
                            spans.push(Span::raw(wrapped_line[last_pos..match_idx].to_string()));
                        }
                        spans.push(Span::styled(
                            wrapped_line[match_idx..match_idx + filter_lower.len()].to_string(),
                            Style::default().bg(Color::Yellow).fg(Color::Black),
                        ));
                        last_pos = match_idx + filter_lower.len();
                    }
                    if last_pos < wrapped_line.len() {
                        spans.push(Span::raw(wrapped_line[last_pos..].to_string()));
                    }

                    if spans.is_empty() {
                        lines.push(Line::from(wrapped_line));
                    } else {
                        lines.push(Line::from(spans));
                    }
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

/// Wrap text to fit within a given width (character-based)
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 || text.is_empty() {
        return vec![text.to_string()];
    }

    let mut lines = vec![];
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            // First word on line
            if word.chars().count() > width {
                // Word is longer than width, just put it on its own line
                lines.push(word.to_string());
            } else {
                current_line = word.to_string();
            }
        } else if (current_line.chars().count() + 1 + word.chars().count()) <= width {
            // Word fits on current line
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            // Word doesn't fit, start new line
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
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
            Span::styled("x", Style::default().fg(Color::Red).bold()),
            Span::raw(" delete  "),
            Span::styled("D", Style::default().fg(Color::Red).bold()),
            Span::raw(" bulk  "),
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

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Draw popup overlay for delete period selection
pub fn draw_delete_period_popup(
    f: &mut Frame,
    area: Rect,
    selected_index: usize,
) {
    // Center popup
    let popup_area = centered_rect(50, 40, area);

    // Clear background
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Delete History ")
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(Color::Black).fg(Color::White));

    f.render_widget(Clear, popup_area);
    f.render_widget(block, popup_area);

    // Content area (inside border)
    let inner = popup_area.inner(&Margin { vertical: 2, horizontal: 2 });

    let periods = vec![
        ("Last Hour", "Delete entries from the past hour"),
        ("Last Day", "Delete entries from the past 24 hours"),
        ("Last Week", "Delete entries from the past 7 days"),
        ("Last Month", "Delete entries from the past 30 days"),
        ("Last Year", "Delete entries from the past 365 days"),
        ("ALL ENTRIES", "⚠ Delete EVERYTHING (requires 3 confirmations)"),
    ];

    let mut lines = vec![
        Line::from(Span::styled(
            "Select time period to delete:",
            Style::default().fg(Color::Gray)
        )),
        Line::from(""),
    ];

    for (idx, (label, description)) in periods.iter().enumerate() {
        let is_selected = idx == selected_index;
        let prefix = if is_selected { "> " } else { "  " };
        let style = if is_selected {
            Style::default().fg(Color::Cyan).bold()
        } else if idx == 5 {
            Style::default().fg(Color::Red)
        } else {
            Style::default()
        };

        lines.push(Line::from(Span::styled(
            format!("{}{}", prefix, label),
            style,
        )));

        if is_selected {
            lines.push(Line::from(Span::styled(
                format!("  {}", description),
                Style::default().fg(Color::Gray).italic(),
            )));
        }

        lines.push(Line::from(""));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("⏎ ", Style::default().fg(Color::Green)),
        Span::raw("select  "),
        Span::styled("⎋ ", Style::default().fg(Color::Red)),
        Span::raw("cancel"),
    ]));

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);
}

/// Draw confirmation popup for bulk delete
pub fn draw_delete_confirmation_popup(
    f: &mut Frame,
    area: Rect,
    period: DeletePeriod,
    is_all: bool,
    confirmation_count: u8,
) {
    let popup_area = centered_rect(60, 30, area);

    let title = if is_all {
        format!(" CONFIRM DELETION ({}/3) ", confirmation_count + 1)
    } else {
        " Confirm Deletion ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(Color::Black).fg(Color::Red));

    f.render_widget(Clear, popup_area);
    f.render_widget(block, popup_area);

    let inner = popup_area.inner(&Margin { vertical: 2, horizontal: 2 });

    let warning_style = Style::default().fg(Color::Red).bold();

    let mut lines = vec![
        Line::from(Span::styled("⚠ WARNING", warning_style)),
        Line::from(""),
    ];

    if is_all {
        lines.push(Line::from(Span::styled(
            "You are about to delete ALL clipboard history!",
            warning_style,
        )));
        lines.push(Line::from(Span::styled(
            "This action CANNOT be undone!",
            warning_style,
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Confirmation {}/3", confirmation_count + 1),
            Style::default().fg(Color::Yellow),
        )));
    } else {
        lines.push(Line::from(vec![
            Span::raw("Delete entries from: "),
            Span::styled(period.display(), Style::default().fg(Color::Yellow).bold()),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "This action cannot be undone.",
            Style::default().fg(Color::Gray),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("y", Style::default().fg(Color::Red).bold()),
        Span::raw(" confirm  "),
        Span::styled("n", Style::default().fg(Color::Green).bold()),
        Span::raw(" cancel"),
    ]));

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(paragraph, inner);
}

/// Draw confirmation popup for single entry delete
pub fn draw_single_delete_confirmation_popup(
    f: &mut Frame,
    area: Rect,
    entry: &ClipboardEntry,
) {
    let popup_area = centered_rect(60, 30, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Delete Entry ")
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(Color::Black).fg(Color::Yellow));

    f.render_widget(Clear, popup_area);
    f.render_widget(block, popup_area);

    let inner = popup_area.inner(&Margin { vertical: 2, horizontal: 2 });

    let preview = if entry.content.len() > 100 {
        format!("{}...", &entry.content[..100])
    } else {
        entry.content.clone()
    }.replace('\n', "↵");

    let lines = vec![
        Line::from(Span::styled(
            "Delete this clipboard entry?",
            Style::default().bold(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            preview,
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("y", Style::default().fg(Color::Red).bold()),
            Span::raw(" delete  "),
            Span::styled("n", Style::default().fg(Color::Green).bold()),
            Span::raw(" cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(paragraph, inner);
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
