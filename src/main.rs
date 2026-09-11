use std::fs::exists;
use std::path::PathBuf;
use clap::{Parser};
use commands::enums::Commands;
use crate::commands::enums::{CrashSubcommands, DelTarget};
use crate::commands::init::init;
use crate::commands::list::list;
use crate::commands::del::del;
use crate::lock::Lock;
use crate::models::crash::{Crash, CrashContext};

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
        Commands::Init { path, frameworks } => {
            init(&path, &frameworks);
        }
        Commands::Inspect {crash_file} => {
            //TODO
        }
        Commands::Crash {subcommand} => {
            match subcommand{
                CrashSubcommands::New {
                    error_code,
                    file_path,
                    span,
                    msg,
                    trace,
                    framework,
                    output } => {
                    Crash
                    {
                        error_code: error_code.clone(),
                        file_path: file_path.clone(),
                        span: span.clone(),
                        message: msg.clone().unwrap_or_default(),
                        invocation_trace: trace.clone().unwrap_or_default(),
                        context: CrashContext::generate(framework.clone().unwrap_or_else
                        (|| Lock::read().get_all_frameworks()))
                    }.save(output.clone())
                }
                CrashSubcommands::List => {
                    list(&PathBuf::from("./.bissue/crashes"), "crash");
                }
                CrashSubcommands::Del { target } => {
                    let lock = &mut Lock::read();
                    del(target.as_ref().unwrap_or_else(|| &DelTarget::Last),
                        &PathBuf::from("./.bissue/crashes"), "crash", &lock.last_crash);

                    if !exists(&lock.last_crash).expect("Cannot check if last crash path exists") {
                        lock.last_crash = PathBuf::new();
                        Lock::write(&lock);
                    }
                }
            }
        }
        _ => {}
    }
}
