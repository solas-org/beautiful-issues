use crate::commands::del::del;
use crate::commands::enums::{CrashSubcommands, DelTarget, IssueSubcommands, PushTarget, TicketSubcommand};
use crate::commands::init::init;
use crate::commands::inspect::inspect;
use crate::commands::list::list;
use crate::commands::push::push;
use crate::configs::find_bissue_dir;
use crate::lock::Lock;
use crate::models::crash::{CrashContext, CrashFile};
use clap::Parser;
use commands::enums::Commands;
use std::path::PathBuf;

mod models;
mod commands;
mod configs;
mod lock;
mod render;

#[derive(Parser)]
#[command(version, about, disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

enum ItemKind {
    Crash,
    Issue,
    Ticket,
}

impl ItemKind {
    fn folder(&self) -> &'static str {
        match self {
            ItemKind::Crash => "crashes",
            ItemKind::Issue => "issues",
            ItemKind::Ticket => "tickets",
        }
    }

    fn extension(&self) -> &'static str {
        match self {
            ItemKind::Crash => "crash",
            ItemKind::Issue => "issue",
            ItemKind::Ticket => "ticket",
        }
    }

    fn get_last_path<'a>(&self, lock: &'a mut Lock) -> &'a mut PathBuf {
        match self {
            ItemKind::Crash => &mut lock.last_crash,
            ItemKind::Issue => &mut lock.last_issue,
            ItemKind::Ticket => &mut lock.last_ticket,
        }
    }
}

fn handle_delete(kind: ItemKind, target: Option<&DelTarget>) -> anyhow::Result<()> {
    let dir = find_bissue_dir().join(kind.folder());
    let mut lock = Lock::load();
    let last_path = kind.get_last_path(&mut lock);

    del(
        target.unwrap_or(&DelTarget::Last),
        &dir,
        kind.extension(),
        last_path,
    )?;

    if !last_path.exists() {
        *last_path = PathBuf::new();
        lock.write()?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { path, frameworks } => {
            init(path, frameworks)?;
        }
        Commands::Inspect { crash_file } => {
            inspect(crash_file.as_deref());
        }
        Commands::Crash { subcommand } => match subcommand {
            CrashSubcommands::New {
                diagnostics,
                framework,
                output,
            } => {
                CrashFile {
                    crashes: diagnostics.clone(),
                    context: CrashContext::generate(framework.as_ref()),
                }
                .save(output.clone())?;
            }
            CrashSubcommands::List => {
                list(&find_bissue_dir().join("crashes"), "crash")?;
            }
            CrashSubcommands::Del { target } => {
                handle_delete(ItemKind::Crash, target.as_ref())?;
            }
        },
        Commands::Issue { subcommand } => match subcommand {
            IssueSubcommands::New { issue, output } => {
                issue.save(output.clone())?;
            }
            IssueSubcommands::List => {
                list(&find_bissue_dir().join("issues"), "issue")?;
            }
            IssueSubcommands::Del { target } => {
                handle_delete(ItemKind::Issue, target.as_ref())?;
            }
        },
        Commands::Ticket { subcommand } => match subcommand {
            TicketSubcommand::New { ticket, output } => {
                ticket.save(output.clone())?;
            }
            TicketSubcommand::List => {
                list(&find_bissue_dir().join("tickets"), "ticket")?;
            }
            TicketSubcommand::Del { target } => {
                handle_delete(ItemKind::Ticket, target.as_ref())?;
            }
            TicketSubcommand::Push {
                target,
                remote_type,
                custom_remotes,
            } => {
                push(
                    target.as_ref().unwrap_or(&PushTarget::Last),
                    remote_type,
                    custom_remotes).await?;
            }
        },
    }

    Ok(())
}
