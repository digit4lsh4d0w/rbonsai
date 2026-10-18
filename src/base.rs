use std::io::{Write, stdout};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal,
};

use crate::{Config, utility::Style};

pub struct BaseSegment {
    pub x: u16,
    pub y: u16,
    pub text: &'static str,
    pub style: Style,
}

pub fn base_segments(config: &Config, cols: u16, rows: u16) -> Vec<BaseSegment> {
    let mut segments = Vec::new();

    match config.base {
        1 => {
            let lines = [
                ":___________./~~~\\.___________:",
                " \\                           / ",
                "  \\_________________________/ ",
                "  (_)                     (_)",
            ];
            let base_width = lines
                .iter()
                .map(|line| line.chars().count() as u16)
                .max()
                .unwrap_or(0);
            let start_pos = (cols / 2).saturating_sub(base_width / 2);
            let y = rows.saturating_sub(lines.len() as u16);
            let style = |color| Style {
                attribute: Attribute::Bold,
                foreground_color: Color::AnsiValue(color),
                background_color: Color::Reset,
            };

            segments.push(BaseSegment {
                x: start_pos,
                y,
                text: ":",
                style: style(8),
            });
            segments.push(BaseSegment {
                x: start_pos + 1,
                y,
                text: "___________",
                style: style(2),
            });
            segments.push(BaseSegment {
                x: start_pos + 12,
                y,
                text: "./~~~\\.",
                style: style(11),
            });
            segments.push(BaseSegment {
                x: start_pos + 19,
                y,
                text: "___________",
                style: style(2),
            });
            segments.push(BaseSegment {
                x: start_pos + 30,
                y,
                text: ":",
                style: style(8),
            });
            segments.push(BaseSegment {
                x: start_pos,
                y: y + 1,
                text: lines[1],
                style: style(8),
            });
            segments.push(BaseSegment {
                x: start_pos,
                y: y + 2,
                text: lines[2],
                style: style(8),
            });
            segments.push(BaseSegment {
                x: start_pos,
                y: y + 3,
                text: lines[3],
                style: style(8),
            });
        }
        2 => {
            let start_pos = (cols / 2).saturating_sub(15 / 2);
            let y = rows.saturating_sub(3);
            let style = |color| Style {
                attribute: Attribute::NormalIntensity,
                foreground_color: Color::AnsiValue(color),
                background_color: Color::Reset,
            };

            segments.push(BaseSegment {
                x: start_pos,
                y,
                text: "(",
                style: style(8),
            });
            segments.push(BaseSegment {
                x: start_pos + 1,
                y,
                text: "---",
                style: style(2),
            });
            segments.push(BaseSegment {
                x: start_pos + 4,
                y,
                text: "./~~~\\.",
                style: style(11),
            });
            segments.push(BaseSegment {
                x: start_pos + 11,
                y,
                text: "---",
                style: style(2),
            });
            segments.push(BaseSegment {
                x: start_pos + 14,
                y,
                text: ")",
                style: style(8),
            });
            segments.push(BaseSegment {
                x: start_pos,
                y: y + 1,
                text: " (           ) ",
                style: style(8),
            });
            segments.push(BaseSegment {
                x: start_pos,
                y: y + 2,
                text: "  (_________)  ",
                style: style(8),
            });
        }
        _ => {}
    }

    segments
}

pub fn draw_base(config: &Config) {
    let (cols, rows) = terminal::size().unwrap();
    let mut stdout = stdout();

    for segment in base_segments(config, cols, rows) {
        queue!(
            stdout,
            SetAttribute(segment.style.attribute),
            SetForegroundColor(segment.style.foreground_color),
            SetBackgroundColor(segment.style.background_color),
            MoveTo(segment.x, segment.y),
            Print(segment.text),
        )
        .unwrap();
    }

    stdout.flush().unwrap();
}
