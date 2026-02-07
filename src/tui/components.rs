use crate::db::ClipboardEntry;
use crate::tui::fuzzy;
use chrono::{DateTime, Local, Utc};
use once_cell::sync::Lazy;
use ratatui::{
    prelude::*,
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Clear, Paragraph},
    layout::{Alignment, Margin},
};
use regex::Regex;
use crate::tui::app::DeletePeriod;

static EMAIL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap()
});
static URL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://[^\s<>\[\]()]+").unwrap()
});
static IP_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap()
});
static SECRET_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(password|secret|token|api[_-]?key|auth)[=:]\s*\S+").unwrap()
});
static UUID_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}").unwrap()
});

#[derive(Clone, Copy)]
enum PatternType {
    Email,
    Url,
    Ip,
    Secret,
    Uuid,
}

impl PatternType {
    fn color(self) -> Color {
        match self {
            PatternType::Email => Color::Cyan,
            PatternType::Url => Color::Blue,
            PatternType::Ip => Color::Green,
            PatternType::Secret => Color::Red,
            PatternType::Uuid => Color::Magenta,
        }
    }
}

fn find_patterns(text: &str) -> Vec<(usize, usize, PatternType)> {
    let patterns: &[(_, PatternType)] = &[
        (&*EMAIL_RE, PatternType::Email),
        (&*URL_RE, PatternType::Url),
        (&*IP_RE, PatternType::Ip),
        (&*SECRET_RE, PatternType::Secret),
        (&*UUID_RE, PatternType::Uuid),
    ];

    let mut matches: Vec<_> = patterns.iter()
        .flat_map(|(re, ptype)| re.find_iter(text).map(move |m| (m.start(), m.end(), *ptype)))
        .collect();

    matches.sort_by_key(|(start, _, _)| *start);

    let mut result = vec![];
    let mut last_end = 0;
    for (start, end, ptype) in matches {
        if start >= last_end {
            result.push((start, end, ptype));
            last_end = end;
        }
    }
    result
}

fn highlight_patterns(text: &str) -> Vec<Span<'static>> {
    let patterns = find_patterns(text);
    if patterns.is_empty() {
        return vec![Span::raw(text.to_string())];
    }

    let mut spans = vec![];
    let mut last_end = 0;

    for (start, end, ptype) in patterns {
        if start > last_end {
            spans.push(Span::raw(text[last_end..start].to_string()));
        }
        spans.push(Span::styled(
            text[start..end].to_string(),
            Style::default().fg(ptype.color()),
        ));
        last_end = end;
    }

    if last_end < text.len() {
        spans.push(Span::raw(text[last_end..].to_string()));
    }

    spans
}

fn highlight_search(text: &str, query: &str) -> Vec<Span<'static>> {
    if query.is_empty() {
        return highlight_patterns(text);
    }

    let chars: Vec<char> = text.chars().collect();
    let chars_lower: Vec<char> = text.to_lowercase().chars().collect();
    let query_chars: Vec<char> = query.to_lowercase().chars().collect();

    if chars_lower.len() < query_chars.len() {
        return highlight_patterns(text);
    }

    let mut spans = vec![];
    let mut last_end = 0;
    let max_i = chars_lower.len() - query_chars.len();

    let mut i = 0;
    while i <= max_i {
        if chars_lower[i..i + query_chars.len()] == query_chars[..] {
            if i > last_end {
                spans.push(Span::raw(chars[last_end..i].iter().collect::<String>()));
            }
            spans.push(Span::styled(
                chars[i..i + query_chars.len()].iter().collect::<String>(),
                Style::default().bg(Color::Yellow).fg(Color::Black),
            ));
            last_end = i + query_chars.len();
            i = last_end;
        } else {
            i += 1;
        }
    }

    if last_end < chars.len() {
        spans.push(Span::raw(chars[last_end..].iter().collect::<String>()));
    }

    if spans.is_empty() {
        highlight_patterns(text)
    } else {
        spans
    }
}

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
    f.render_widget(Paragraph::new(lines), area);
}

pub fn draw_entry_list(
    f: &mut Frame,
    area: Rect,
    entries: Vec<&ClipboardEntry>,
    selected_index: usize,
    scroll_offset: usize,
    filter_text: &str,
) {
    let width = area.width as usize;
    let content_max_width = width.saturating_sub(13);

    let visible_entries: Vec<Line> = entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_selected = (scroll_offset + idx) == selected_index;
            let content_preview = entry.content.replace('\n', "↵").replace('\r', "");

            let content_display = if content_preview.chars().count() > content_max_width {
                let truncated: String = content_preview.chars().take(content_max_width.saturating_sub(1)).collect();
                format!("{}…", truncated)
            } else {
                content_preview
            };

            let date_str = format_relative_date(&entry.last_copied);
            let selector = if is_selected { ">" } else { " " };
            let style = if is_selected { Style::default().fg(Color::Cyan) } else { Style::default() };

            if filter_text.is_empty() {
                let full_line = format!("{} {:width$}{:>10}", selector, content_display, date_str, width = content_max_width);
                Line::from(full_line).patch_style(style)
            } else {
                let fuzzy_result = fuzzy::fuzzy_match(&content_display, filter_text);
                let mut spans: Vec<Span> = vec![Span::raw(format!("{} ", selector))];

                if fuzzy_result.matched {
                    let chars: Vec<char> = content_display.chars().collect();
                    let mut last_pos = 0;

                    for (match_start, match_len) in &fuzzy_result.match_positions {
                        if *match_start > last_pos {
                            spans.push(Span::raw(chars[last_pos..*match_start].iter().collect::<String>()));
                        }
                        spans.push(Span::styled(
                            chars[*match_start..(*match_start + match_len)].iter().collect::<String>(),
                            Style::default().bg(Color::Yellow).fg(Color::Black),
                        ));
                        last_pos = *match_start + match_len;
                    }
                    if last_pos < chars.len() {
                        spans.push(Span::raw(chars[last_pos..].iter().collect::<String>()));
                    }
                } else {
                    spans.push(Span::raw(content_display));
                }

                let current_len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
                let padding = content_max_width.saturating_sub(current_len.saturating_sub(2));
                if padding > 0 {
                    spans.push(Span::raw(" ".repeat(padding)));
                }

                spans.push(Span::styled(format!("{:>10}", date_str), Style::default().fg(Color::Gray)));
                Line::from(spans).patch_style(style)
            }
        })
        .collect();

    if visible_entries.is_empty() {
        let message = if entries.is_empty() { "No clipboard history found." } else { "No matches." };
        f.render_widget(Paragraph::new(message).style(Style::default().fg(Color::Gray)), area);
    } else {
        f.render_widget(Paragraph::new(visible_entries).block(Block::default()), area);
    }
}

pub fn draw_preview(
    f: &mut Frame,
    area: Rect,
    entry: Option<&ClipboardEntry>,
    filter_text: &str,
    scroll_offset: usize,
) -> (usize, Option<usize>) {
    let width = area.width.saturating_sub(2) as usize;
    let height = area.height as usize;

    let (lines, first_match_line) = if let Some(e) = entry {
        let mut lines = vec![];
        let mut first_match: Option<usize> = None;

        lines.push(Line::from(Span::styled(
            format!("─ {}", format_absolute_date(&e.created_at)),
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(""));

        for content_line in e.content.lines() {
            for wrapped_line in wrap_text(content_line, width) {
                let line = if filter_text.is_empty() {
                    Line::from(highlight_patterns(&wrapped_line))
                } else {
                    if first_match.is_none() && wrapped_line.to_lowercase().contains(&filter_text.to_lowercase()) {
                        first_match = Some(lines.len());
                    }
                    Line::from(highlight_search(&wrapped_line, filter_text))
                };
                lines.push(line);
            }
        }

        (lines, first_match)
    } else {
        (vec![Line::from(Span::styled("No entry selected", Style::default().fg(Color::Gray)))], None)
    };

    let total_lines = lines.len();
    let visible_lines: Vec<Line> = lines.into_iter().skip(scroll_offset).take(height).collect();

    let content_area = Rect { x: area.x, y: area.y, width: area.width.saturating_sub(1), height: area.height };
    f.render_widget(Paragraph::new(visible_lines), content_area);

    if total_lines > height {
        let scrollbar_area = Rect { x: area.x + area.width.saturating_sub(1), y: area.y, width: 1, height: area.height };
        draw_scrollbar(f, scrollbar_area, scroll_offset, total_lines, height);
    }

    (total_lines, first_match_line)
}

fn draw_scrollbar(f: &mut Frame, area: Rect, offset: usize, total: usize, visible: usize) {
    let height = area.height as usize;
    if height == 0 || total <= visible {
        return;
    }

    let thumb_height = ((visible as f64 / total as f64) * height as f64).max(1.0) as usize;
    let max_offset = total.saturating_sub(visible);
    let thumb_pos = if max_offset > 0 {
        ((offset as f64 / max_offset as f64) * (height.saturating_sub(thumb_height)) as f64) as usize
    } else {
        0
    };

    let scrollbar_lines: Vec<Line> = (0..height)
        .map(|i| {
            let ch = if i >= thumb_pos && i < thumb_pos + thumb_height { "█" } else { "░" };
            Line::from(Span::styled(ch, Style::default().fg(Color::Gray)))
        })
        .collect();

    f.render_widget(Paragraph::new(scrollbar_lines), area);
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 || text.is_empty() {
        return vec![text.to_string()];
    }

    let mut lines = vec![];
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            if word.chars().count() > width {
                lines.push(word.to_string());
            } else {
                current_line = word.to_string();
            }
        } else if (current_line.chars().count() + 1 + word.chars().count()) <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

pub fn draw_status_bar(f: &mut Frame, area: Rect, is_filtering: bool, filter_text: &str, _db_path: &str) {
    let content = if is_filtering {
        Line::from(vec![
            Span::styled("🔍 Filter: ", Style::default().fg(Color::Yellow).bold()),
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
            Span::raw(" copy "),
            Span::styled("/", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" filter "),
            Span::styled("d", Style::default().fg(Color::Red).bold()),
            Span::raw(" del "),
            Span::styled("r", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" refresh "),
            Span::styled("h/l", Style::default().fg(Color::Blue).bold()),
            Span::raw(" scroll "),
            Span::styled("x", Style::default().fg(Color::Red).bold()),
            Span::raw(" del "),
            Span::styled("D", Style::default().fg(Color::Red).bold()),
            Span::raw(" bulk "),
            Span::styled("q", Style::default().fg(Color::Magenta).bold()),
            Span::raw(" quit"),
        ])
    };

    f.render_widget(Paragraph::new(content), area);
}

fn format_relative_date(date: &DateTime<Utc>) -> String {
    let duration = Utc::now().signed_duration_since(*date);

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

fn format_absolute_date(date: &DateTime<Utc>) -> String {
    date.with_timezone(&Local).format("%b %d at %H:%M").to_string()
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
        assert_eq!(format_relative_date(&Utc::now()), "now");
    }

    #[test]
    fn test_format_relative_date_minutes_ago() {
        let date = Utc::now() - chrono::Duration::minutes(5);
        assert_eq!(format_relative_date(&date), "5m ago");
    }

    #[test]
    fn test_find_patterns_email() {
        let patterns = find_patterns("Contact: user@example.com");
        assert_eq!(patterns.len(), 1);
        assert!(matches!(patterns[0].2, PatternType::Email));
    }

    #[test]
    fn test_find_patterns_url() {
        let patterns = find_patterns("Visit https://example.com");
        assert_eq!(patterns.len(), 1);
        assert!(matches!(patterns[0].2, PatternType::Url));
    }

    #[test]
    fn test_wrap_text() {
        let wrapped = wrap_text("hello world test", 10);
        assert_eq!(wrapped.len(), 2);
    }

    #[test]
    fn test_highlight_search() {
        let spans = highlight_search("Hello World", "world");
        assert_eq!(spans.len(), 2);
    }

    #[test]
    fn test_highlight_search_unicode() {
        let spans = highlight_search("Héllo Wörld", "wörld");
        assert_eq!(spans.len(), 2);
    }

    #[test]
    fn test_highlight_search_empty_text() {
        let spans = highlight_search("", "query");
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn test_highlight_search_query_longer_than_text() {
        let spans = highlight_search("ab", "abcdef");
        assert_eq!(spans.len(), 1);
    }
}
