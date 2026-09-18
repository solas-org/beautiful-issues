use crate::commands::del::del_tui;
use crate::commands::enums::{
    Commands, CrashSubcommands, DelTarget, IssueSubcommands, PushTarget, TicketSubcommand,
};
use crate::commands::init::init;
use crate::commands::inspect::inspect;
use crate::commands::list::list_tui;
use crate::commands::push::push;
use crate::configs::find_bissue_dir;
use crate::lock::Lock;
use crate::models::crash::{CrashContext, CrashFile};
use crate::render::app::{run_tui_app, App, AppScreen};
use crate::render::theme::Theme;
use crate::render::tui::Tui;
use crate::render::views::issue_wizard::IssueWizardModel;
use crate::render::views::list::ListTab;
use crate::render::views::ticket_wizard::TicketWizardModel;
use clap::Parser;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { path, frameworks } => {
            init(path, frameworks)?;
        }
        Commands::Inspect { crash_file } => {
            inspect(crash_file.as_deref())?;
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
                list_tui(Some(ListTab::Crashes))?;
            }
            CrashSubcommands::Del { target } => {
                let lock = Lock::load();
                del_tui(
                    target.as_ref().unwrap_or(&DelTarget::Last),
                    &find_bissue_dir().join("crashes"),
                    "crash",
                    &lock.last_crash,
                )?;
            }
        },
        Commands::Issue { subcommand } => match subcommand {
            IssueSubcommands::New { issue, output } => {
                if issue.is_empty() {
                    let theme = Theme::default_theme();
                    let tui = Tui::init()?;
                    let app = App::new(AppScreen::IssueWizard(IssueWizardModel::new()), theme);
                    run_tui_app(tui, app)?;
                } else {
                    issue.save(output.clone())?;
                }
            }
            IssueSubcommands::List => {
                list_tui(Some(ListTab::Issues))?;
            }
            IssueSubcommands::Del { target } => {
                let lock = Lock::load();
                del_tui(
                    target.as_ref().unwrap_or(&DelTarget::Last),
                    &find_bissue_dir().join("issues"),
                    "issue",
                    &lock.last_issue,
                )?;
            }
        },
        Commands::Ticket { subcommand } => match subcommand {
            TicketSubcommand::New { ticket, output } => {
                if ticket.header.is_empty() && ticket.description.is_empty() {
                    let theme = Theme::default_theme();
                    let tui = Tui::init()?;
                    let app = App::new(
                        AppScreen::TicketWizard(TicketWizardModel::new(ticket.crash_file.clone())),
                        theme,
                    );
                    run_tui_app(tui, app)?;
                } else {
                    ticket.save(output.clone())?;
                }
            }
            TicketSubcommand::List => {
                list_tui(Some(ListTab::Tickets))?;
            }
            TicketSubcommand::Del { target } => {
                let lock = Lock::load();
                del_tui(
                    target.as_ref().unwrap_or(&DelTarget::Last),
                    &find_bissue_dir().join("tickets"),
                    "ticket",
                    &lock.last_ticket,
                )?;
            }
            TicketSubcommand::Push {
                target,
                remote_type,
                custom_remotes,
            } => {
                push(
                    target.as_ref().unwrap_or(&PushTarget::Last),
                    remote_type,
                    custom_remotes,
                )
                .await?;
            }
        },
    }

    Ok(())
}
