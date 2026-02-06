use super::app::{App, DeleteMode, DeletePeriod};
use super::components::{
    draw_entry_list, draw_header, draw_preview, draw_status_bar,
    draw_delete_period_popup, draw_delete_confirmation_popup, draw_single_delete_confirmation_popup,
};
use ratatui::prelude::*;

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();

    if size.height < 5 {
        let paragraph = ratatui::widgets::Paragraph::new("Terminal too small");
        f.render_widget(paragraph, size);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(size);

    let header_area = chunks[0];
    let body_area = chunks[1];
    let status_area = chunks[2];

    draw_header(
        f,
        header_area,
        "History",
        &app.get_entry_count_info(),
        app.loading,
    );

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Length(1), Constraint::Percentage(50)])
        .split(body_area);

    let list_area = body_chunks[0];
    let divider_area = body_chunks[1];
    let preview_area = body_chunks[2];

    let visible_entries = app.get_visible_entries();
    draw_entry_list(
        f,
        list_area,
        visible_entries,
        app.selected_index,
        app.scroll_offset,
        &app.filter_text,
    );

    let divider_lines: Vec<_> = (0..list_area.height)
        .map(|_| ratatui::text::Line::from("│"))
        .collect();
    let divider = ratatui::widgets::Paragraph::new(divider_lines)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(divider, divider_area);

    let current_entry = app.current_entry();
    draw_preview(f, preview_area, current_entry, &app.filter_text);

    draw_status_bar(
        f,
        status_area,
        app.is_filtering,
        &app.filter_text,
        &app.get_db_path_short(),
    );

    // Render delete popups on top of everything
    match &app.delete_mode {
        DeleteMode::SelectingPeriod => {
            draw_delete_period_popup(f, size, app.delete_period_index);
        }
        DeleteMode::ConfirmingBulk { period } => {
            draw_delete_confirmation_popup(f, size, *period, false, 0);
        }
        DeleteMode::ConfirmingSingle => {
            if let Some(entry) = app.current_entry() {
                draw_single_delete_confirmation_popup(f, size, entry);
            }
        }
        DeleteMode::ConfirmingAll { confirmation_count } => {
            draw_delete_confirmation_popup(
                f,
                size,
                DeletePeriod::All,
                true,
                *confirmation_count
            );
        }
        DeleteMode::None => {}
    }
}
