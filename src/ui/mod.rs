mod preview;
mod search;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::Block,
};

use crate::app::App;

const BORDER_STYLE: Style = Style::new().fg(Color::DarkGray);

pub(crate) fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let outer = Block::bordered().border_style(BORDER_STYLE);
    let inner = outer.inner(area).inner(Margin::new(1, 0));
    frame.render_widget(outer, area);

    let (left_area, preview_area) = body_areas(inner);
    let [query_area, results_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(5)]).areas(left_area);

    let cursor = search::render_query(
        frame,
        query_area,
        &app.query,
        app.matches.len(),
        app.commands.len(),
    );
    search::render_results(frame, results_area, app);
    preview::render(
        frame,
        preview_area,
        app.selected_command(),
        app.selected_help(),
        BORDER_STYLE,
    );
    frame.set_cursor_position(cursor);
}

fn body_areas(area: Rect) -> (Rect, Rect) {
    let [left, preview] =
        Layout::horizontal([Constraint::Percentage(44), Constraint::Percentage(56)])
            .spacing(1)
            .areas(area);
    (left, preview)
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend};

    use super::*;

    #[test]
    fn renders_search_results_and_help_panes() {
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();

        terminal
            .draw(|frame| render(frame, &App::default()))
            .unwrap();
        let buffer = terminal.backend().buffer();
        let rendered = buffer
            .content
            .chunks(buffer.area.width as usize)
            .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("Search commands"));
        assert!(rendered.contains("0/0"));
        assert!(rendered.contains("No results"));
        assert!(rendered.contains("Select a command to see usage"));
    }
}
