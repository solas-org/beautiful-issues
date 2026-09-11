use clap::{Parser};
use commands::enums::Commands;
use crate::commands::init::init;

mod models;
mod commands;
mod config;
mod lock;
mod render;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { path } => {
            init(path);
        }
        &_ => {}
    }
}
