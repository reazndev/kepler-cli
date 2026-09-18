mod app;
mod catalog;
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
    let mut app = App::new(catalog::scan_installed_commands());
    inspect_selected_help(&mut app);

    while !app.should_exit {
        terminal.draw(|frame| ui::render(frame, &app))?;

        match event::read()? {
            Event::Key(key) => {
                if let Some(path) = app.handle_key(key) {
                    inspect_help(&mut app, path);
                }
            }
            Event::Resize(width, height) => terminal.resize(width, height)?,
            _ => {}
        }
    }

    Ok(())
}

fn inspect_selected_help(app: &mut App) {
    if let Some(path) = app.help_request() {
        inspect_help(app, path);
    }
}

fn inspect_help(app: &mut App, path: std::path::PathBuf) {
    let help = catalog::inspect_help(&path)
        .unwrap_or_else(|error| format!("Could not inspect help: {error}"));
    app.cache_help(path, help);
}
