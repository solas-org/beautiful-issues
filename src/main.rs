use clap::{Parser};
use commands::enums::Commands;

mod models;
mod commands;
mod config;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();

    match &cli {
        &_ => {}
    }
}
