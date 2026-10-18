use clap::Parser;

use crate::base::BaseKind;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Whether the tree generation should pause after each step
    /// to allow the user to watch it grow
    #[arg(short, long)]
    pub live: bool,
    /// In live mode, wait time in seconds between each step of growth
    #[arg(short, long, default_value_t = 0.03, value_parser = parse_non_negative_f64)]
    pub time: f64,
    /// Infinite mode: keep growing trees
    #[arg(short, long)]
    pub infinite: bool,
    /// In infinite mode, the wait time in seconds between each tree
    #[arg(short, long, default_value_t = 4., value_parser = parse_non_negative_f64)]
    pub wait: f64,
    /// Screensaver mode: equivalent to -li and quit on any keypress
    #[arg(short = 'S', long)]
    pub screensaver: bool,
    /// Attach message next to tree
    #[arg(short, long)]
    pub message: Option<String>,
    /// Ascii art plant base to use.
    #[arg(short, long, value_enum, default_value = "1")]
    pub base: BaseKind,
    /// The branch multiplier; higher -> less branches
    #[arg(
        short = 'M',
        long,
        default_value_t = 3,
        value_parser = clap::value_parser!(i32).range(1..)
    )]
    pub multiplier: i32,
    /// The starting life of the tree
    /// higher -> bigger tree
    #[arg(short = 'L', long, default_value_t = 32)]
    pub life: i32,
    /// Random number seed for reproducable trees
    #[arg(short, long)]
    pub seed: Option<u64>,
    // TODO: Add support for saving to file, loading from file
}

#[derive(Clone, Copy)]
pub struct Mode {
    pub live: bool,
    pub infinite: bool,
}

impl Config {
    pub fn mode(&self) -> Mode {
        Mode {
            live: self.live || self.screensaver,
            infinite: self.infinite || self.screensaver,
        }
    }
}

fn parse_non_negative_f64(value: &str) -> Result<f64, String> {
    let value = value.parse::<f64>().map_err(|error| error.to_string())?;

    if value.is_finite() && value >= 0.0 {
        Ok(value)
    } else {
        Err("expected a finite non-negative number".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn defaults_are_parsed_by_clap() {
        let config = Config::try_parse_from(["rbonsai"]).unwrap();

        assert_eq!(config.base, BaseKind::First);
        assert_eq!(config.multiplier, 3);
        assert!((config.time - 0.03).abs() < f64::EPSILON);
        assert!((config.wait - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn screensaver_enables_live_and_infinite_modes() {
        let config = Config::try_parse_from(["rbonsai", "-S"]).unwrap();
        let mode = config.mode();

        assert!(mode.live);
        assert!(mode.infinite);
    }

    #[test]
    fn base_accepts_numeric_values() {
        let config = Config::try_parse_from(["rbonsai", "-b", "2"]).unwrap();

        assert_eq!(config.base, BaseKind::Second);
    }

    #[test]
    fn rejects_invalid_base() {
        assert!(Config::try_parse_from(["rbonsai", "-b", "9"]).is_err());
    }

    #[test]
    fn rejects_non_positive_multiplier() {
        assert!(Config::try_parse_from(["rbonsai", "-M", "0"]).is_err());
    }

    #[test]
    fn rejects_negative_and_non_finite_durations() {
        assert!(Config::try_parse_from(["rbonsai", "-t", "-1"]).is_err());
        assert!(Config::try_parse_from(["rbonsai", "-w", "inf"]).is_err());
    }
}
