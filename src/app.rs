use std::{
    error::Error,
    io,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use rand::{SeedableRng, rngs::SmallRng};

use crate::{
    base::{self, BaseKind},
    cli::{Config, Mode},
    frame::{Frame, style_for},
    message,
    terminal::{self, Terminal, TerminalGuard},
    tree::{self, Segment, TreeParams},
};

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut rng = SmallRng::seed_from_u64(config.seed.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis() as u64)
    }));
    let mut terminal = Terminal::new();
    let mode = config.mode();

    if mode.live || mode.infinite {
        run_interactive(&config, mode, &mut rng, &mut terminal)
    } else {
        run_static(&config, &mut rng, &mut terminal)
    }
}

fn tree_params(config: &Config, cols: u16, rows: u16) -> TreeParams {
    TreeParams {
        life: config.life,
        multiplier: config.multiplier,
        max_x: cols,
        max_y: rows,
        tree_bottom: config.base.tree_bottom(rows),
    }
}

fn build_frame(
    base_kind: BaseKind,
    tree: &[Segment],
    message: Option<&str>,
    cols: u16,
    rows: u16,
) -> Frame {
    let mut frame = Frame::default();

    for segment in base::base_segments(base_kind, cols, rows) {
        frame.put_str(segment.x, segment.y, segment.text, segment.style);
    }

    for val in tree {
        let x = u16::try_from(val.pos.x).unwrap_or_default();
        let y = u16::try_from(val.pos.y).unwrap_or_default();
        frame.put_str(x, y, val.glyph, style_for(val.branch_type, val.bold));
    }

    if let Some(message) = message {
        message::message_to_frame(&mut frame, message, cols, rows);
    }

    frame
}

fn run_static(
    config: &Config,
    rng: &mut SmallRng,
    terminal: &mut Terminal,
) -> Result<(), Box<dyn Error>> {
    let (cols, rows) = Terminal::size();
    let params = tree_params(config, cols, rows);
    let tree = tree::grow_tree(&params, rng);

    let frame = build_frame(config.base, &tree, config.message.as_deref(), cols, rows);

    terminal.write_frame_cropped(&frame)?;
    Ok(())
}

fn run_interactive(
    config: &Config,
    mode: Mode,
    rng: &mut SmallRng,
    terminal: &mut Terminal,
) -> Result<(), Box<dyn Error>> {
    let _guard = TerminalGuard::enter()?;
    terminal::flush_pending_events()?;

    let should_exit = loop {
        let (cols, rows) = Terminal::size();
        let base_kind = config.base;
        let params = tree_params(config, cols, rows);

        terminal.clear()?;
        for segment in base::base_segments(base_kind, cols, rows) {
            terminal.draw_segment(segment.x, segment.y, segment.text, segment.style)?;
        }
        terminal.flush()?;

        let tree = tree::grow_tree(&params, rng);

        // If the user exited before the tree finished being drawn, exit.
        if !draw_animated(config, mode, terminal, &tree)? {
            break true;
        }

        if let Some(message) = &config.message {
            let mut frame = Frame::default();
            message::message_to_frame(&mut frame, message, cols, rows);
            terminal.draw_frame(&frame)?;
        }

        if !mode.infinite {
            break false;
        }

        if terminal::wait_for_key_press(Duration::from_secs_f64(config.wait)) {
            break true;
        }
    };

    let (_, rows) = Terminal::size();
    if !should_exit {
        terminal::wait_for_any_key();
    }
    terminal.leave(rows)?;

    Ok(())
}

fn draw_animated(
    config: &Config,
    mode: Mode,
    terminal: &mut Terminal,
    tree: &[Segment],
) -> io::Result<bool> {
    let delay = Duration::from_secs_f64(config.time);

    for val in tree {
        let x = u16::try_from(val.pos.x).unwrap_or_default();
        let y = u16::try_from(val.pos.y).unwrap_or_default();
        terminal.draw_segment(x, y, val.glyph, style_for(val.branch_type, val.bold))?;

        if mode.live {
            terminal.flush()?;
            if terminal::wait_for_key_press(delay) {
                terminal.reset_color()?;
                return Ok(false);
            }
        }
    }

    terminal.reset_color()?;
    Ok(true)
}
