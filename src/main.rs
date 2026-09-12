extern crate core;

use crate::commands::del::del;
use crate::commands::enums::{CrashSubcommands, DelTarget, IssueSubcommands, TicketSubcommand};
use crate::commands::init::init;
use crate::commands::list::list;
use crate::lock::Lock;
use crate::models::crash::{CrashContext, CrashFile};
use crate::render::unwrappers::exists;
use clap::Parser;
use commands::enums::Commands;
use std::path::PathBuf;

mod models;
mod commands;
mod configs;
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
                    issues,
                    framework,
                    output } => {
                    CrashFile
                    {
                        crashes: issues.clone(),
                        context: CrashContext::generate(&framework)
                    }.save(output.clone())
                }
                CrashSubcommands::List => list(&PathBuf::from("./.bissue/crashes"), "crash"),
                CrashSubcommands::Del { target } => {
                    let lock = &mut Lock::global_clone();
                    del(target.as_ref().unwrap_or_else(|| &DelTarget::Last),
                        &PathBuf::from("./.bissue/crashes"), "crash", &lock.last_crash);

                    if !exists(&lock.last_crash, "Cannot check if last crash path exists")
                    {
                        lock.last_crash = PathBuf::new();
                        Lock::write(&lock);
                    }
                }
            }
        }
        Commands::Issue {subcommand} => {
            match subcommand {
                IssueSubcommands::New { output } => {
                    //TODO
                }
                IssueSubcommands::List => list(&PathBuf::from("./.bissue/issues"), "issue"),
                IssueSubcommands::Del { target } => {
                    let lock = &mut Lock::global_clone();
                    del(target.as_ref().unwrap_or_else(|| &DelTarget::Last),
                        &PathBuf::from("./.bissue/issues"), "issue", &lock.last_issue);

                    if !exists(&lock.last_issue, "Cannot check if last issue path exists")
                    {
                        lock.last_issue = PathBuf::new();
                        Lock::write(&lock);
                    }
                }
            }
        }

        Commands::Ticket { subcommand } => {
            match subcommand {
                TicketSubcommand::New { crash_file, output } => {
                    //TODO
                }
                TicketSubcommand::List => list(&PathBuf::from("./.bissue/tickets"), "ticket"),
                TicketSubcommand::Del { target } => {
                    let lock = &mut Lock::global_clone();
                    del(target.as_ref().unwrap_or_else(|| &DelTarget::Last),
                        &PathBuf::from("./.bissue/tickets"), "ticket", &lock.last_ticket);

                    if !exists(&lock.last_ticket, "Cannot check if last ticket path exists")
                    {
                        lock.last_ticket = PathBuf::new();
                        Lock::write(&lock);
                    }
                }
                TicketSubcommand::Push { target, remote } => {}
            }
        }
    }
}
