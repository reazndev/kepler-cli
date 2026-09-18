use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    sync::mpsc,
    time::Duration,
};

use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode};
use ratatui::{
    Frame, Terminal, TerminalOptions, Viewport, backend::CrosstermBackend, layout::Rect,
};

const INLINE_HEIGHT: u16 = 20;
const CURSOR_TIMEOUT: Duration = Duration::from_secs(2);

pub(crate) struct TtyTerminal {
    inner: Terminal<CrosstermBackend<File>>,
    anchor_y: u16,
    width: u16,
}

impl TtyTerminal {
    pub(crate) fn draw(&mut self, render: impl FnOnce(&mut Frame)) -> io::Result<()> {
        self.inner.draw(render).map(|_| ())
    }

    pub(crate) fn resize(&mut self, width: u16, height: u16) -> io::Result<()> {
        let area = viewport_area(self.anchor_y, width, height);
        let shrinking = width < self.width;
        self.inner.resize(area)?;
        if shrinking {
            // Ratatui clears at row zero on a width reduction; the second call restores our anchor.
            self.inner.resize(area)?;
        }
        self.width = width;
        Ok(())
    }

    pub(crate) fn show_cursor(&mut self) -> io::Result<()> {
        self.inner.show_cursor()
    }
}

pub(crate) fn install_panic_hook() {
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        previous_hook(panic_info);
    }));
}

pub(crate) fn open_terminal() -> io::Result<TtyTerminal> {
    let tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
    enable_raw_mode()?;
    open_raw_terminal(tty).inspect_err(|_| {
        let _ = disable_raw_mode();
    })
}

pub(crate) fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()
}

fn open_raw_terminal(mut tty: File) -> io::Result<TtyTerminal> {
    let (width, height) = terminal::size()?;
    let viewport_height = INLINE_HEIGHT.min(height);
    let cursor_y = cursor_position(&mut tty)?;
    let overflow = cursor_y
        .saturating_add(viewport_height)
        .saturating_sub(height);

    if overflow > 0 {
        tty.write_all("\r\n".repeat(overflow as usize).as_bytes())?;
        tty.flush()?;
    }

    let top = cursor_y.saturating_sub(overflow);
    let inner = Terminal::with_options(
        CrosstermBackend::new(tty),
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, top, width, viewport_height)),
        },
    )?;
    Ok(TtyTerminal {
        inner,
        anchor_y: top,
        width,
    })
}

fn viewport_area(anchor_y: u16, width: u16, height: u16) -> Rect {
    let viewport_height = INLINE_HEIGHT.min(height);
    let top = anchor_y.min(height.saturating_sub(viewport_height));
    Rect::new(0, top, width, viewport_height)
}

fn cursor_position(tty: &mut File) -> io::Result<u16> {
    tty.write_all(b"\x1b[6n")?;
    tty.flush()?;

    let mut reader = tty.try_clone()?;
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::spawn(move || sender.send(read_cursor_position(&mut reader)));
    receiver.recv_timeout(CURSOR_TIMEOUT).map_err(|_| {
        io::Error::new(
            io::ErrorKind::TimedOut,
            "terminal did not report its cursor position",
        )
    })?
}

fn read_cursor_position(reader: &mut File) -> io::Result<u16> {
    let mut response = Vec::with_capacity(16);
    loop {
        let mut byte = [0];
        reader.read_exact(&mut byte)?;
        response.push(byte[0]);
        if byte[0] == b'R' || response.len() == response.capacity() {
            break;
        }
    }

    let response = std::str::from_utf8(&response)
        .ok()
        .and_then(|value| value.strip_prefix("\x1b["))
        .and_then(|value| value.strip_suffix('R'))
        .and_then(|value| value.split_once(';'))
        .and_then(|(row, _)| row.parse::<u16>().ok());
    response.and_then(|row| row.checked_sub(1)).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid cursor position response",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_viewport_visible_across_resizes() {
        assert_eq!(viewport_area(10, 100, 40), Rect::new(0, 10, 100, 20));
        assert_eq!(viewport_area(10, 60, 12), Rect::new(0, 0, 60, 12));
    }
}
