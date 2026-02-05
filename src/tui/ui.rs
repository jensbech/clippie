use super::app::App;
use super::components::{draw_entry_list, draw_header, draw_preview, draw_status_bar};
use ratatui::prelude::*;

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();

    if size.height < 5 {
        // Terminal too small
        let paragraph = ratatui::widgets::Paragraph::new("Terminal too small");
        f.render_widget(paragraph, size);
        return;
    }

    // Split layout vertically: header | body | status
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header
            Constraint::Min(5),    // Body (list + preview)
            Constraint::Length(1), // Status bar
        ])
        .split(size);

    let header_area = chunks[0];
    let body_area = chunks[1];
    let status_area = chunks[2];

    // Draw header
    draw_header(
        f,
        header_area,
        "History",
        &app.get_entry_count_info(),
        app.loading,
    );

    // Split body into list and preview (50/50 with divider)
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Length(1), Constraint::Percentage(50)])
        .split(body_area);

    let list_area = body_chunks[0];
    let divider_area = body_chunks[1];
    let preview_area = body_chunks[2];

    // Draw list
    let visible_entries = app.get_visible_entries();
    draw_entry_list(
        f,
        list_area,
        visible_entries,
        app.selected_index,
        app.scroll_offset,
        &app.filter_text,
    );

    // Draw divider
    let divider_lines: Vec<_> = (0..list_area.height)
        .map(|_| ratatui::text::Line::from("│"))
        .collect();
    let divider = ratatui::widgets::Paragraph::new(divider_lines)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(divider, divider_area);

    // Draw preview
    let current_entry = app.current_entry();
    draw_preview(f, preview_area, current_entry);

    // Draw status bar
    draw_status_bar(
        f,
        status_area,
        app.is_filtering,
        &app.filter_text,
        &app.get_db_path_short(),
    );
}
