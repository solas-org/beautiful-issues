use crate::commands::del::del_tui;
use crate::commands::enums::{
    Commands, CrashSubcommands, DelTarget, IssueSubcommands, PackageSubcommand, PushTarget,
    TicketSubcommand,
};
use crate::commands::init::init;
use crate::commands::inspect::inspect;
use crate::commands::list::list_tui;
use crate::commands::package::{add, check, del, generate};
use crate::commands::push::push;
use crate::configs::find_bissue_dir;
use crate::lock::Lock;
use crate::models::crash::{CrashContext, CrashFile};
use crate::render::app::{App, AppScreen, run_tui_app};
use crate::render::theme::Theme;
use crate::render::tui::Tui;
use crate::render::views::issue_wizard::IssueWizardModel;
use crate::render::views::list::ListTab;
use crate::render::views::ticket_wizard::TicketWizardModel;
use clap::Parser;

mod commands;
mod configs;
mod lock;
mod models;
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
        Commands::Init { path } => {
            init(path)?;
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
            CrashSubcommands::Del { target, yes } => {
                let lock = Lock::load();
                del_tui(
                    target.as_ref().unwrap_or(&DelTarget::Last),
                    &find_bissue_dir().join("crashes"),
                    "crash",
                    &lock.last_crash,
                    yes,
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
            IssueSubcommands::Del { target, yes } => {
                let lock = Lock::load();
                del_tui(
                    target.as_ref().unwrap_or(&DelTarget::Last),
                    &find_bissue_dir().join("issues"),
                    "issue",
                    &lock.last_issue,
                    yes,
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
            TicketSubcommand::Del { target, yes } => {
                let lock = Lock::load();
                del_tui(
                    target.as_ref().unwrap_or(&DelTarget::Last),
                    &find_bissue_dir().join("tickets"),
                    "ticket",
                    &lock.last_ticket,
                    yes,
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

        Commands::Package { subcommand } => match &subcommand {
            PackageSubcommand::Gen { path, frameworks } => {
                generate(path.as_ref(), frameworks)?;
            }
            PackageSubcommand::Add {
                framework,
                name,
                version,
                path,
            } => add(framework, name, version, path)?,
            PackageSubcommand::Del { framework, name } => {
                del(framework, name)?;
            }
            PackageSubcommand::Check { framework, names } => {
                check(framework, names)?;
            }
        },
    }

    Ok(())
}
