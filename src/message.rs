use std::mem;

use crossterm::style::Color;

use crate::frame::{self, Frame};

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

pub fn message_to_frame(frame: &mut Frame, message: &str, max_x: u16, max_y: u16) {
    let Some(msg) = layout_message(message, max_x, max_y) else {
        return;
    };

    let style = frame::plain_style(Color::White);
    let border = message_border(msg.inner_width);

    frame.put_str_opaque(msg.x, msg.y, &border, style);

    for (row, line) in msg.lines.iter().enumerate() {
        frame.put_str_opaque(
            msg.x,
            msg.y + 1 + row as u16,
            &message_line(line, msg.inner_width),
            style,
        );
    }

    frame.put_str_opaque(msg.x, msg.y + 1 + msg.lines.len() as u16, &border, style);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_at_word_boundaries() {
        assert_eq!(wrap_message("hello world", 5), ["hello", "world"]);
    }

    #[test]
    fn hard_splits_words_that_do_not_fit() {
        assert_eq!(wrap_message("abcdefgh", 3), ["abc", "def", "gh"]);
    }

    #[test]
    fn message_box_is_centered_inside_the_terminal() {
        let msg = layout_message("hi", 80, 24).unwrap();

        assert_eq!(msg.x, 54);
        assert_eq!(msg.y, 15);
        assert_eq!(msg.inner_width, 2);
        assert_eq!(msg.lines, ["hi"]);
    }

    #[test]
    fn tiny_terminal_has_no_message_box() {
        assert!(layout_message("hi", 4, 24).is_none());
        assert!(layout_message("hi", 80, 2).is_none());
    }
}
