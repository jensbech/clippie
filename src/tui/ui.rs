use super::app::{App, DeleteMode, DeletePeriod};
use super::components::{
    draw_entry_list, draw_header, draw_preview, draw_status_bar,
    draw_delete_period_popup, draw_delete_confirmation_popup, draw_single_delete_confirmation_popup,
};
use super::theme::{BASE_BG, BORDER_FG};
use ratatui::prelude::*;

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.size();

    f.render_widget(
        ratatui::widgets::Block::default().style(Style::default().bg(BASE_BG)),
        size,
    );

    if size.height < 5 {
        let paragraph = ratatui::widgets::Paragraph::new("Terminal too small")
            .style(Style::default().bg(BASE_BG));
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

    let all_entries = app.filtered_entries();
    draw_entry_list(
        f,
        list_area,
        all_entries,
        app.selected_index,
        app.scroll_offset,
        &app.filter_text,
    );

    let divider_lines: Vec<_> = (0..list_area.height)
        .map(|_| ratatui::text::Line::from("│"))
        .collect();
    let divider = ratatui::widgets::Paragraph::new(divider_lines)
        .style(Style::default().fg(BORDER_FG).bg(BASE_BG));
    f.render_widget(divider, divider_area);

    let current_entry = app.current_entry();
    let preview_height = preview_area.height as usize;
    let (total_lines, first_match) = draw_preview(
        f,
        preview_area,
        current_entry,
        &app.filter_text,
        app.preview_scroll,
    );

    if let Some(match_line) = first_match {
        if match_line >= app.preview_scroll + preview_height || match_line < app.preview_scroll {
            app.preview_scroll = match_line.saturating_sub(preview_height / 4);
        }
    }

    let max_scroll = total_lines.saturating_sub(preview_height);
    if app.preview_scroll > max_scroll {
        app.preview_scroll = max_scroll;
    }

    draw_status_bar(
        f,
        status_area,
        app.is_filtering,
        &app.filter_text,
        &app.get_db_path_short(),
    );

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
