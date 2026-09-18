use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::Style,
    widgets::{Block, Paragraph, Wrap},
};

use crate::catalog::CommandEntry;

pub(super) fn render(
    frame: &mut Frame,
    area: Rect,
    command: Option<&CommandEntry>,
    help: Option<&str>,
    border_style: Style,
) {
    frame.render_widget(Block::bordered().border_style(border_style), area);
    let inner = area.inner(Margin::new(1, 1));
    if let Some(command) = command {
        let [help_area, path_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(inner);
        let help = help.unwrap_or("Help unavailable");
        frame.render_widget(Paragraph::new(help).wrap(Wrap { trim: false }), help_area);
        frame.render_widget(
            Paragraph::new(command.path.display().to_string()),
            path_area,
        );
        return;
    }
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
