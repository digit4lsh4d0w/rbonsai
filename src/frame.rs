use std::{
    collections::BTreeMap,
    io::{self, Write},
};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{
        Attribute, Attributes, Color, Colors, ContentStyle, Print, ResetColor, SetAttribute,
        SetAttributes, SetColors,
    },
};

use crate::tree::BranchType;

pub type Style = ContentStyle;

pub fn style_for(branch_type: BranchType, bold: bool) -> Style {
    let (foreground_color, attributes) = match branch_type {
        BranchType::Trunk | BranchType::ShootLeft | BranchType::ShootRight => (
            if bold {
                Color::AnsiValue(11)
            } else {
                Color::AnsiValue(3)
            },
            bold_attributes(bold),
        ),
        BranchType::Dying => (Color::AnsiValue(2), bold_attributes(bold)),
        BranchType::Dead => (Color::AnsiValue(10), bold_attributes(bold)),
    };

    Style {
        foreground_color: Some(foreground_color),
        background_color: Some(Color::Reset),
        underline_color: None,
        attributes,
    }
}

pub fn ansi_style(color: u8, attribute: Attribute) -> Style {
    Style {
        foreground_color: Some(Color::AnsiValue(color)),
        background_color: Some(Color::Reset),
        underline_color: None,
        attributes: Attributes::from(attribute),
    }
}

pub fn plain_style(color: Color) -> Style {
    Style {
        foreground_color: Some(color),
        background_color: Some(Color::Reset),
        underline_color: None,
        attributes: Attributes::none(),
    }
}

fn bold_attributes(bold: bool) -> Attributes {
    if bold {
        Attributes::from(Attribute::Bold)
    } else {
        Attributes::none()
    }
}

/// Writes a style using the most compact crossterm commands available.
///
/// Underline color is not used by the project and is intentionally ignored.
pub fn queue_style(out: &mut impl Write, style: Style) -> io::Result<()> {
    // A reset clears both colors and attributes, so it must come first.
    if style.attributes.is_empty() {
        queue!(out, SetAttribute(Attribute::Reset))?;
    } else {
        queue!(out, SetAttributes(style.attributes))?;
    }

    let foreground = style.foreground_color.unwrap_or(Color::Reset);
    let background = style.background_color.unwrap_or(Color::Reset);
    queue!(out, SetColors(Colors::new(foreground, background)))
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
        self.put_str_impl(x, y, text, style, false);
    }

    /// Like [`Frame::put_str`], but spaces also overwrite existing cells.
    pub fn put_str_opaque(&mut self, x: u16, y: u16, text: &str, style: Style) {
        self.put_str_impl(x, y, text, style, true);
    }

    fn put_str_impl(&mut self, x: u16, y: u16, text: &str, style: Style, keep_spaces: bool) {
        for (offset, ch) in text.chars().enumerate() {
            if ch == ' ' && !keep_spaces {
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

    pub fn render_cropped(&self, out: &mut impl Write) -> io::Result<()> {
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

    pub fn render_absolute(&self, out: &mut impl Write) -> io::Result<()> {
        let mut last_style = None;

        for (&(y, x), cell) in &self.cells {
            if last_style != Some(cell.style) {
                queue_style(out, cell.style)?;
                last_style = Some(cell.style);
            }

            queue!(out, MoveTo(x, y), Print(cell.ch))?;
        }

        Ok(())
    }
}

fn flush_run(out: &mut impl Write, run: &mut String, style: Option<Style>) -> io::Result<()> {
    if run.is_empty() {
        return Ok(());
    }

    if let Some(style) = style {
        queue_style(out, style)?;
    }

    queue!(out, Print(run.as_str()))?;
    run.clear();
    Ok(())
}

#[cfg(test)]
mod tests {
    use crossterm::style::Color;

    use super::*;

    fn style() -> Style {
        plain_style(Color::White)
    }

    #[test]
    fn put_str_skips_spaces() {
        let mut frame = Frame::default();
        frame.put_str(0, 0, "a b", style());

        assert_eq!(frame.cells.len(), 2);
    }

    #[test]
    fn opaque_put_str_keeps_spaces() {
        let mut frame = Frame::default();
        frame.put_str_opaque(0, 0, "a b", style());

        assert_eq!(frame.cells.len(), 3);
    }

    #[test]
    fn render_cropped_fills_gaps_with_spaces() {
        let mut frame = Frame::default();
        frame.put_str(0, 0, "a b", style());

        let mut out = Vec::new();
        frame.render_cropped(&mut out).unwrap();
        let rendered = String::from_utf8(out).unwrap();

        assert!(rendered.contains("a b"));
        assert!(rendered.ends_with('\n'));
    }
}
