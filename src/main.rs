mod app;
mod shell;
mod ui;

use app::App;
use crossterm::event::{self, Event};
use shell::TtyTerminal;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    shell::install_panic_hook();
    let mut terminal = shell::open_terminal()?;
    let result = run(&mut terminal);
    let _ = terminal.show_cursor();
    let restore_result = shell::restore_terminal();
    result?;
    Ok(restore_result?)
}

fn run(terminal: &mut TtyTerminal) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::default();

    while !app.should_exit {
        terminal.draw(|frame| ui::render(frame, &app))?;

        match event::read()? {
            Event::Key(key) => app.handle_key(key),
            Event::Resize(width, height) => terminal.resize(width, height)?,
            _ => {}
        }
    }

    Ok(())
}
