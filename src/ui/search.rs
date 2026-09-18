use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Position, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

pub(super) fn render_query(frame: &mut Frame, area: Rect, query: &str) -> Position {
    let [prompt_area, input_area] =
        Layout::horizontal([Constraint::Length(2), Constraint::Min(0)]).areas(area);
    frame.render_widget(
        Paragraph::new(Span::styled("> ", Style::new().fg(Color::Cyan))),
        prompt_area,
    );

    let (input, scroll, cursor) = query_view(query, input_area.width);
    frame.render_widget(Paragraph::new(input).scroll((0, scroll)), input_area);
    Position::new(input_area.x + cursor, input_area.y)
}

fn query_view(query: &str, area_width: u16) -> (Line<'_>, u16, u16) {
    if query.is_empty() {
        return (
            Line::styled("Search commands", Style::new().fg(Color::DarkGray)),
            0,
            0,
        );
    }

    let query_width = Span::raw(query).width().min(u16::MAX as usize) as u16;
    let visible_width = area_width.saturating_sub(1);
    let scroll = query_width.saturating_sub(visible_width);
    (Line::raw(query), scroll, query_width - scroll)
}

pub(super) fn render_results(frame: &mut Frame, area: Rect) {
    render_empty(frame, area, "No results");
}

fn render_empty(frame: &mut Frame, area: Rect, message: &str) {
    let [_, message_area, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new(message).alignment(Alignment::Center),
        message_area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrolls_long_queries_to_the_active_end() {
        let (_, scroll, cursor) = query_view("0123456789", 6);

        assert_eq!(scroll, 5);
        assert_eq!(cursor, 5);
    }

    #[test]
    fn measures_query_in_terminal_cells() {
        let (_, scroll, cursor) = query_view("a\u{301}界", 8);

        assert_eq!(scroll, 0);
        assert_eq!(cursor, 3);
    }
}
