fn main() -> Result<(), Box<dyn std::error::Error>> {
    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| {
                let block = ratatui::widgets::Block::bordered().title("Kepler CLI");
                let paragraph = ratatui::widgets::Paragraph::new("Press any key to exit")
                    .centered()
                    .block(block);
                frame.render_widget(paragraph, frame.area());
            })?;

            if crossterm::event::read()?.is_key_press() {
                break Ok(());
            }
        }
    })
}
