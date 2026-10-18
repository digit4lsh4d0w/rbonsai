use std::{
    io::{self, BufWriter, Stdout, Write, stdout},
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{self, MoveTo},
    event::{self, Event},
    execute, queue,
    style::{Print, ResetColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::frame::{self, Frame, Style};

pub struct Terminal {
    out: BufWriter<Stdout>,
    current_style: Option<Style>,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            out: BufWriter::new(stdout()),
            current_style: None,
        }
    }

    pub fn size() -> (u16, u16) {
        terminal::size()
            .ok()
            .filter(|&(cols, rows)| cols > 0 && rows > 0)
            .or_else(|| {
                let cols = std::env::var("COLUMNS").ok()?.parse().ok()?;
                let rows = std::env::var("LINES").ok()?.parse().ok()?;
                Some((cols, rows))
            })
            .unwrap_or((80, 24))
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.current_style = None;
        execute!(self.out, Clear(ClearType::All))
    }

    pub fn draw_segment(&mut self, x: u16, y: u16, text: &str, style: Style) -> io::Result<()> {
        self.set_style(style)?;
        queue!(self.out, MoveTo(x, y), Print(text))
    }

    fn set_style(&mut self, style: Style) -> io::Result<()> {
        if self.current_style != Some(style) {
            frame::queue_style(&mut self.out, style)?;
            self.current_style = Some(style);
        }

        Ok(())
    }

    pub fn draw_frame(&mut self, frame: &Frame) -> io::Result<()> {
        frame.render_absolute(&mut self.out)?;
        queue!(self.out, ResetColor)?;
        self.current_style = None;
        self.out.flush()
    }

    pub fn write_frame_cropped(&mut self, frame: &Frame) -> io::Result<()> {
        frame.render_cropped(&mut self.out)?;
        self.current_style = None;
        self.out.flush()
    }

    pub fn reset_color(&mut self) -> io::Result<()> {
        queue!(self.out, ResetColor)?;
        self.current_style = None;
        self.out.flush()
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }

    pub fn leave(&mut self, rows: u16) -> io::Result<()> {
        execute!(
            self.out,
            LeaveAlternateScreen,
            MoveTo(0, rows.saturating_sub(1))
        )?;
        self.out.flush()
    }
}

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;

        let guard = Self;
        let mut out = stdout();
        execute!(out, cursor::Hide, EnterAlternateScreen)?;
        Ok(guard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut out = stdout();
        let _ = terminal::disable_raw_mode();
        let _ = execute!(out, LeaveAlternateScreen, cursor::Show);
    }
}

pub fn flush_pending_events() -> io::Result<()> {
    while event::poll(Duration::from_millis(10))? {
        let _ = event::read();
    }
    Ok(())
}

pub fn wait_for_key_press(timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return false;
        }

        match event::poll(remaining) {
            Ok(true) => match event::read() {
                Ok(Event::Key(key)) if key.is_press() => return true,
                Ok(_) => {}
                Err(_) => return true,
            },
            Ok(false) => return false,
            Err(_) => return true,
        }
    }
}

pub fn wait_for_any_key() {
    while let Ok(event) = event::read() {
        if matches!(event, Event::Key(key) if key.is_press()) {
            break;
        }
    }
}
