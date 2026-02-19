use super::app::{App, DeleteMode, DeletePeriod};
use super::components::{
    dim_background, draw_confirm_quit_popup, draw_entry_list, draw_header, draw_preview,
    draw_search_bar, draw_status_bar,
    draw_delete_period_popup, draw_delete_confirmation_popup, draw_single_delete_confirmation_popup,
};
use ratatui::prelude::*;

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.size();

    if size.height < 5 {
        let paragraph = ratatui::widgets::Paragraph::new("Terminal too small");
        f.render_widget(paragraph, size);
        return;
    }

    let show_search_bar = app.is_filtering || !app.filter_text.is_empty();

    let constraints = if show_search_bar {
        vec![
            Constraint::Min(5),
            Constraint::Length(1),
            Constraint::Length(1),
        ]
    } else {
        vec![
            Constraint::Min(5),
            Constraint::Length(1),
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(size);

    let body_area = chunks[0];

    // Draw the bordered header/body area
    draw_header(
        f,
        body_area,
        "History",
        &app.get_entry_count_info(),
        app.loading,
    );

    // Inner area inside the border
    let inner = body_area.inner(&ratatui::layout::Margin { vertical: 1, horizontal: 1 });

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Length(1), Constraint::Percentage(50)])
        .split(inner);

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

    let divider_lines: Vec<_> = (0..divider_area.height)
        .map(|_| ratatui::text::Line::from("│"))
        .collect();
    let divider = ratatui::widgets::Paragraph::new(divider_lines)
        .style(Style::default().fg(Color::Rgb(60, 60, 80)));
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

    // Draw search bar if active
    if show_search_bar {
        let match_count = app.filtered_entries().len();
        draw_search_bar(
            f,
            chunks[1],
            &app.filter_text,
            app.is_filtering,
            match_count,
        );
        draw_status_bar(
            f,
            chunks[2],
            app.is_filtering,
            &app.filter_text,
            app.confirm_quit,
            app.is_in_delete_mode(),
            app.message.as_deref(),
        );
    } else {
        draw_status_bar(
            f,
            chunks[1],
            app.is_filtering,
            &app.filter_text,
            app.confirm_quit,
            app.is_in_delete_mode(),
            app.message.as_deref(),
        );
    }

    // Render overlays on top of everything
    if app.confirm_quit {
        dim_background(f);
        draw_confirm_quit_popup(f, size);
    }

    match &app.delete_mode {
        DeleteMode::SelectingPeriod => {
            dim_background(f);
            draw_delete_period_popup(f, size, app.delete_period_index);
        }
        DeleteMode::ConfirmingBulk { period } => {
            dim_background(f);
            draw_delete_confirmation_popup(f, size, *period, false, 0);
        }
        DeleteMode::ConfirmingSingle => {
            if let Some(entry) = app.current_entry() {
                dim_background(f);
                draw_single_delete_confirmation_popup(f, size, entry);
            }
        }
        DeleteMode::ConfirmingAll { confirmation_count } => {
            dim_background(f);
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
