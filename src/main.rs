mod app;
mod base;
mod cli;
mod frame;
mod message;
mod terminal;
mod tree;

use clap::Parser;

use crate::cli::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse();
    app::run(config)
}
