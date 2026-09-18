use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::Style,
    widgets::{Block, Paragraph},
};

pub(super) fn render(frame: &mut Frame, area: Rect, border_style: Style) {
    frame.render_widget(Block::bordered().border_style(border_style), area);
    let inner = area.inner(Margin::new(1, 1));
    let [_, message_area, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(inner);
    frame.render_widget(
        Paragraph::new("Select a command to see usage").alignment(Alignment::Center),
        message_area,
    );
}
