use crossterm::style::Attribute;

use crate::frame::{self, Style};

const FIRST_BASE: &[(u16, u16, &str, u8)] = &[
    (0, 0, ":", 8),
    (1, 0, "___________", 2),
    (12, 0, "./~~~\\.", 11),
    (19, 0, "___________", 2),
    (30, 0, ":", 8),
    (0, 1, " \\                           / ", 8),
    (0, 2, "  \\_________________________/ ", 8),
    (0, 3, "  (_)                     (_)", 8),
];

const SECOND_BASE: &[(u16, u16, &str, u8)] = &[
    (0, 0, "(", 8),
    (1, 0, "---", 2),
    (4, 0, "./~~~\\.", 11),
    (11, 0, "---", 2),
    (14, 0, ")", 8),
    (0, 1, " (           ) ", 8),
    (0, 2, "  (_________)  ", 8),
];

pub struct BaseSegment {
    pub x: u16,
    pub y: u16,
    pub text: &'static str,
    pub style: Style,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum BaseKind {
    #[value(name = "1")]
    First,
    #[value(name = "2")]
    Second,
}

impl BaseKind {
    pub fn tree_bottom(self, rows: u16) -> u16 {
        let art_height = match self {
            Self::First => spans_height(FIRST_BASE),
            Self::Second => spans_height(SECOND_BASE),
        };

        rows.saturating_sub(art_height.saturating_add(1))
    }
}

pub fn base_segments(kind: BaseKind, cols: u16, rows: u16) -> Vec<BaseSegment> {
    let mut segments = Vec::new();

    match kind {
        BaseKind::First => {
            let start_x = centered_start(cols, spans_width(FIRST_BASE));
            let start_y = rows.saturating_sub(spans_height(FIRST_BASE));
            extend_segments(&mut segments, start_x, start_y, FIRST_BASE, Attribute::Bold);
        }
        BaseKind::Second => {
            let start_x = centered_start(cols, spans_width(SECOND_BASE));
            let start_y = rows.saturating_sub(spans_height(SECOND_BASE));
            extend_segments(
                &mut segments,
                start_x,
                start_y,
                SECOND_BASE,
                Attribute::NormalIntensity,
            );
        }
    }

    segments
}

fn centered_start(cols: u16, width: u16) -> u16 {
    (cols / 2).saturating_sub(width / 2)
}

fn extend_segments(
    segments: &mut Vec<BaseSegment>,
    start_x: u16,
    start_y: u16,
    spans: &[(u16, u16, &'static str, u8)],
    attribute: Attribute,
) {
    segments.extend(spans.iter().map(|&(dx, dy, text, color)| BaseSegment {
        x: start_x.saturating_add(dx),
        y: start_y.saturating_add(dy),
        text,
        style: frame::ansi_style(color, attribute),
    }));
}

fn spans_width(spans: &[(u16, u16, &str, u8)]) -> u16 {
    spans
        .iter()
        .map(|&(dx, _, text, _)| dx.saturating_add(text.chars().count() as u16))
        .max()
        .unwrap_or(0)
}

fn spans_height(spans: &[(u16, u16, &str, u8)]) -> u16 {
    spans
        .iter()
        .map(|&(_, dy, _, _)| dy.saturating_add(1))
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_bottom_stays_above_the_base_art() {
        assert_eq!(BaseKind::First.tree_bottom(40), 35);
        assert_eq!(BaseKind::Second.tree_bottom(40), 36);
    }

    #[test]
    fn base_is_horizontally_centered() {
        let segments = base_segments(BaseKind::First, 80, 24);
        let min_x = segments.iter().map(|segment| segment.x).min().unwrap();

        assert_eq!(min_x, (80 / 2) - (31 / 2));
    }

    #[test]
    fn base_width_is_derived_from_the_art() {
        assert_eq!(spans_width(FIRST_BASE), 31);
        assert_eq!(spans_width(SECOND_BASE), 15);
    }
}
