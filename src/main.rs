use std::{
    error::Error,
    io::{BufWriter, Write, stdout},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use crossterm::{
    cursor::{self, MoveTo},
    event, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::{SeedableRng, rngs::SmallRng};

mod base;
mod bonsai;
mod utility;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Whether the tree generation should pause after each step
    /// to allow the user to watch it grow
    #[arg(short, long)]
    pub live: bool,
    /// In live mode, wait time in seconds between each step of growth
    #[arg(short, long, default_value_t = 0.03)]
    pub time: f64,
    /// Infinite mode: keep growing trees
    #[arg(short, long)]
    pub infinite: bool,
    /// In infinite mode, the wait time in seconds between each tree
    #[arg(short, long, default_value_t = 4.)]
    pub wait: f64,
    /// Screensaver mode: equivalent to -li and quit on any keypress
    #[arg(short = 'S', long)]
    pub screensaver: bool,
    /// Attach message next to tree
    #[arg(short, long)]
    pub message: Option<String>,
    /// Ascii art plant base to use.
    #[arg(short, long, default_value_t = 1)]
    pub base: u8,
    /// The branch multiplier; higher -> less branches
    #[arg(short = 'M', long, default_value_t = 3)]
    pub multiplier: i32,
    /// The starting life of the tree
    /// higher -> bigger tree
    #[arg(short = 'L', long, default_value_t = 32)]
    pub life: i32,
    /// Random number seed for reproducable trees
    #[arg(short, long)]
    pub seed: Option<u64>,
    /// Whether there should be debug prints
    #[arg(short, long)]
    pub verbose: bool,
    // TODO: Add support for saving to file, loading from file
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = stdout();
        let _ = disable_raw_mode();
        let _ = execute!(stdout, LeaveAlternateScreen, cursor::Show);
    }
}

fn terminal_size() -> (u16, u16) {
    crossterm::terminal::size()
        .ok()
        .filter(|&(cols, rows)| cols > 0 && rows > 0)
        .or_else(|| {
            let cols = std::env::var("COLUMNS").ok()?.parse().ok()?;
            let rows = std::env::var("LINES").ok()?.parse().ok()?;
            Some((cols, rows))
        })
        .unwrap_or((80, 24))
}

fn render_frame(
    args: &Config,
    tree: &[bonsai::Val],
    cols: u16,
    rows: u16,
) -> Result<(), Box<dyn Error>> {
    let mut frame = utility::Frame::default();

    for segment in base::base_segments(args, cols, rows) {
        frame.put_str(segment.x, segment.y, segment.text, segment.style);
    }

    for val in tree {
        let x = u16::try_from(val.pos.x).unwrap_or_default();
        let y = u16::try_from(val.pos.y).unwrap_or_default();
        frame.put_str(x, y, val.char, val.style);
    }

    if let Some(message) = &args.message {
        utility::message_to_frame(&mut frame, message, cols, rows);
    }

    let mut out = BufWriter::new(stdout());
    frame.render_cropped(&mut out)?;
    out.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = Config::parse();

    if args.screensaver {
        args.live = true;
        args.infinite = true;
    }
    args.multiplier = args.multiplier.max(1);
    args.time = args.time.max(0.0);
    args.wait = args.wait.max(0.0);

    let seed = args.seed.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs())
    });

    let mut rng = SmallRng::seed_from_u64(seed);

    // Static output is the default; the alternate screen is only needed for
    // animated/infinite modes.
    let interactive = args.live || args.infinite || args.screensaver;
    if !interactive {
        let (cols, rows) = terminal_size();
        let tree = bonsai::grow_tree_at(&args, &mut rng, cols, rows, false);
        return render_frame(&args, &tree, cols, rows);
    }

    let mut stdout = stdout();

    enable_raw_mode()?;
    let _guard = TerminalGuard;
    execute!(stdout, cursor::Hide, EnterAlternateScreen)?;

    // Flush any pending events
    while event::poll(Duration::from_millis(10))? {
        let _ = event::read();
    }

    let should_exit = loop {
        bonsai::init(&args);
        let tree = bonsai::grow_tree(&args, &mut rng);

        // if the user exited before the tree is finished being drawn, should
        // exit program
        if !bonsai::draw_tree(&args, &tree) {
            break true;
        }

        if let Some(message) = &args.message {
            utility::create_message_window(message)?;
        }

        if !args.infinite {
            break false;
        }

        if utility::wait_for_key_press(Duration::from_secs_f64(args.wait)) {
            break true;
        }
    };

    let (_, rows) = terminal_size();
    if should_exit {
        execute!(stdout, LeaveAlternateScreen)?;
        execute!(stdout, MoveTo(0, rows.saturating_sub(1)))?;
    } else {
        utility::wait_for_any_key();
        execute!(stdout, LeaveAlternateScreen)?;
        execute!(stdout, MoveTo(0, rows.saturating_sub(1)))?;
    }

    Ok(())
}
