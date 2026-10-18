use std::{
    collections::BTreeMap,
    error::Error,
    io::{Write, stdout},
    mem,
    time::{Duration, Instant},
};

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyEvent, KeyEventKind},
    queue,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal::size,
};
use rand::{RngExt, rngs::SmallRng};

use crate::bonsai::BranchType;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub attribute: Attribute,
    pub foreground_color: Color,
    pub background_color: Color,
}

#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    style: Style,
}

#[derive(Default)]
pub struct Frame {
    cells: BTreeMap<(u16, u16), Cell>,
}

impl Frame {
    pub fn put_str(&mut self, x: u16, y: u16, text: &str, style: Style) {
        for (offset, ch) in text.chars().enumerate() {
            if ch == ' ' {
                continue;
            }

            let Some(cx) = u16::try_from(offset)
                .ok()
                .and_then(|offset| x.checked_add(offset))
            else {
                break;
            };

            self.cells.insert((y, cx), Cell { ch, style });
        }
    }

    pub fn render_cropped(&self, out: &mut impl Write) -> std::io::Result<()> {
        if self.cells.is_empty() {
            return Ok(());
        }

        let (mut min_x, mut min_y, mut max_x, mut max_y) = (u16::MAX, u16::MAX, 0, 0);
        for &(y, x) in self.cells.keys() {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }

        for y in min_y..=max_y {
            let Some((&(_, last_x), _)) = self.cells.range((y, min_x)..=(y, max_x)).next_back()
            else {
                queue!(out, ResetColor, Print('\n'))?;
                continue;
            };

            let mut run = String::new();
            let mut run_style = None;

            for x in min_x..=last_x {
                match self.cells.get(&(y, x)) {
                    Some(cell) => {
                        if run_style != Some(cell.style) {
                            flush_run(out, &mut run, run_style)?;
                            run_style = Some(cell.style);
                        }
                        run.push(cell.ch);
                    }
                    None => run.push(' '),
                }
            }

            flush_run(out, &mut run, run_style)?;
            queue!(out, ResetColor, Print('\n'))?;
        }

        Ok(())
    }
}

fn flush_run(out: &mut impl Write, run: &mut String, style: Option<Style>) -> std::io::Result<()> {
    if run.is_empty() {
        return Ok(());
    }

    if let Some(style) = style {
        queue!(
            out,
            SetAttribute(style.attribute),
            SetForegroundColor(style.foreground_color),
            SetBackgroundColor(style.background_color),
        )?;
    }

    queue!(out, Print(run.as_str()))?;
    run.clear();
    Ok(())
}

pub fn set_deltas(
    branch_type: BranchType,
    life: i32,
    age: i32,
    multiplier: i32,
    rng: &mut SmallRng,
) -> (i32, i32) {
    match branch_type {
        BranchType::Trunk => {
            if age <= 2 || life < 4 {
                (rng.random_range(-1..=1), 0)
            } else if age < multiplier.saturating_mul(3) {
                let dy = if age % (multiplier / 2).max(1) == 0 {
                    -1
                } else {
                    0
                };
                let dx = match rng.random_range(0..10) {
                    0 => -2,
                    1..=3 => -1,
                    4..=5 => 0,
                    6..=8 => 1,
                    _ => 2,
                };
                (dx, dy)
            } else {
                let dy = if rng.random_range(0..10) > 2 { -1 } else { 0 };
                (rng.random_range(-1..=1), dy)
            }
        }
        BranchType::ShootLeft => {
            let dy = match rng.random_range(0..10) {
                0..=1 => -1,
                2..=7 => 0,
                _ => 1,
            };
            let dx = match rng.random_range(0..10) {
                0..=1 => -2,
                2..=5 => -1,
                6..=8 => 0,
                _ => 1,
            };
            (dx, dy)
        }
        BranchType::ShootRight => {
            let dy = match rng.random_range(0..10) {
                0..=1 => -1,
                2..=7 => 0,
                _ => 1,
            };
            let dx = match rng.random_range(0..10) {
                0..=1 => 2,
                2..=5 => 1,
                6..=8 => 0,
                _ => -1,
            };
            (dx, dy)
        }
        BranchType::Dying => {
            let dy = match rng.random_range(0..10) {
                0..=1 => -1,
                2..=8 => 0,
                _ => 1,
            };
            let dx = match rng.random_range(0..15) {
                0 => -3,
                1..=2 => -2,
                3..=5 => -1,
                6..=8 => 0,
                9..=11 => 1,
                12..=13 => 2,
                _ => 3,
            };
            (dx, dy)
        }
        BranchType::Dead => {
            let dy = match rng.random_range(0..10) {
                0..=2 => -1,
                3..=6 => 0,
                _ => 1,
            };
            (rng.random_range(-1..=1), dy)
        }
    }
}

pub fn choose_string(branch_type: BranchType, life: i32, dx: i32, dy: i32) -> &'static str {
    if life < 4 {
        return "&";
    }

    match branch_type {
        BranchType::Trunk => match (dx, dy) {
            (0, 0) => "/~",
            _ if dx < 0 => "\\|",
            (0, _) => "/|\\",
            _ if dx > 0 => "|/",
            _ => "?", // Fallback
        },
        BranchType::ShootLeft => match (dx, dy) {
            _ if dy > 0 => "\\",
            (0, 0) => "\\_",
            _ if dx < 0 => "\\|",
            (0, _) => "/|",
            _ if dx > 0 => "/",
            _ => "?", // Fallback
        },
        BranchType::ShootRight => match (dx, dy) {
            _ if dy > 0 => "/",
            (0, 0) => "_/",
            _ if dx < 0 => "\\|",
            (0, _) => "/|",
            _ if dx > 0 => "/",
            _ => "?", // Fallback
        },
        BranchType::Dying | BranchType::Dead => "&",
    }
}

pub fn choose_color(branch_type: BranchType, rng: &mut SmallRng) -> Style {
    let (attribute, foreground_color) = match branch_type {
        BranchType::Trunk | BranchType::ShootLeft | BranchType::ShootRight => {
            if rng.random_range(0..2) == 0 {
                (Attribute::Bold, Color::AnsiValue(11))
            } else {
                (Attribute::Reset, Color::AnsiValue(3))
            }
        }
        BranchType::Dying => {
            let attribute = if rng.random_range(0..10) == 0 {
                Attribute::Bold
            } else {
                Attribute::Reset
            };
            (attribute, Color::AnsiValue(2))
        }
        BranchType::Dead => {
            let attribute = if rng.random_range(0..3) == 0 {
                Attribute::Bold
            } else {
                Attribute::Reset
            };
            (attribute, Color::AnsiValue(10))
        }
    };

    Style {
        attribute,
        foreground_color,
        background_color: Color::Reset,
    }
}

fn wrap_message(message: &str, max_width: usize) -> Vec<String> {
    let max_width = max_width.max(1);
    let mut lines = Vec::new();
    let mut current = String::with_capacity(max_width);
    let mut current_len = 0;

    for word in message.split_whitespace() {
        let word_len = word.chars().count();

        // Hard-split words that cannot fit on a line at all.
        if word_len > max_width {
            if current_len > 0 {
                lines.push(mem::take(&mut current));
                current.reserve(max_width);
                current_len = 0;
            }

            let chars: Vec<char> = word.chars().collect();
            lines.extend(
                chars
                    .chunks(max_width)
                    .map(|chunk| chunk.iter().copied().collect()),
            );
            continue;
        }

        if current_len > 0 && current_len + 1 + word_len > max_width {
            lines.push(mem::take(&mut current));
            current.reserve(max_width);
            current_len = 0;
        } else if current_len > 0 {
            current.push(' ');
            current_len += 1;
        }

        current.push_str(word);
        current_len += word_len;
    }

    if current_len > 0 {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

struct MessageBox {
    x: u16,
    y: u16,
    inner_width: usize,
    lines: Vec<String>,
}

fn message_style() -> Style {
    Style {
        attribute: Attribute::Reset,
        foreground_color: Color::White,
        background_color: Color::Reset,
    }
}

fn layout_message(message: &str, max_x: u16, max_y: u16) -> Option<MessageBox> {
    if max_x < 5 || max_y < 3 {
        return None;
    }

    let preferred_width = usize::from(max_x / 4).max(1);
    let hard_limit = usize::from(max_x.saturating_sub(4)).max(1);
    let longest_word = message
        .split_whitespace()
        .map(|word| word.chars().count())
        .max()
        .unwrap_or(0);

    let mut lines = wrap_message(message, preferred_width.max(longest_word.min(hard_limit)));
    lines.truncate(usize::from(max_y - 2));

    let inner_width = lines.iter().map(|line| line.chars().count()).max()?.max(1);
    let box_width = inner_width + 4;
    let box_height = lines.len() + 2;

    let x = ((u32::from(max_x) * 7 / 10) as u16)
        .saturating_sub(2)
        .min(max_x.saturating_sub(box_width as u16));
    let y = ((u32::from(max_y) * 7 / 10) as u16)
        .saturating_sub(1)
        .min(max_y.saturating_sub(box_height as u16));

    Some(MessageBox {
        x,
        y,
        inner_width,
        lines,
    })
}

fn message_border(inner_width: usize) -> String {
    format!("+{}+", "-".repeat(inner_width + 2))
}

fn message_line(line: &str, inner_width: usize) -> String {
    format!("| {line:<inner_width$} |")
}

pub fn create_message_window(message: &str) -> Result<(), Box<dyn Error>> {
    let (max_x, max_y) = size()?;
    let Some(msg) = layout_message(message, max_x, max_y) else {
        return Ok(());
    };

    let style = message_style();
    let border = message_border(msg.inner_width);
    let mut stdout = stdout();

    queue!(
        stdout,
        SetAttribute(style.attribute),
        SetForegroundColor(style.foreground_color),
        SetBackgroundColor(style.background_color),
        MoveTo(msg.x, msg.y),
        Print(&border),
    )?;

    for (row, line) in msg.lines.iter().enumerate() {
        queue!(
            stdout,
            MoveTo(msg.x, msg.y + 1 + row as u16),
            Print(message_line(line, msg.inner_width)),
        )?;
    }

    queue!(
        stdout,
        MoveTo(msg.x, msg.y + 1 + msg.lines.len() as u16),
        Print(&border),
        ResetColor,
    )?;

    stdout.flush()?;
    Ok(())
}

pub fn message_to_frame(frame: &mut Frame, message: &str, max_x: u16, max_y: u16) {
    let Some(msg) = layout_message(message, max_x, max_y) else {
        return;
    };

    let style = message_style();
    let border = message_border(msg.inner_width);

    frame.put_str(msg.x, msg.y, &border, style);

    for (row, line) in msg.lines.iter().enumerate() {
        frame.put_str(
            msg.x,
            msg.y + 1 + row as u16,
            &message_line(line, msg.inner_width),
            style,
        );
    }

    frame.put_str(msg.x, msg.y + 1 + msg.lines.len() as u16, &border, style);
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
                Ok(Event::Key(KeyEvent {
                    kind: KeyEventKind::Press,
                    ..
                })) => return true,
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
        if matches!(
            event,
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                ..
            })
        ) {
            break;
        }
    }
}
