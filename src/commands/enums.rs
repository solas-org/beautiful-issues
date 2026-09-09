use std::path::PathBuf;
use clap::{Subcommand};
use url::Url;
use crate::models::crash::CrashSpan;

#[derive(Subcommand)]
pub(crate) enum Commands {
    ///Creates .bissue folder in root of your project and initiates there
    Init{
        ///Project root path
        #[arg(default_value_os_t = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))]
        path: PathBuf,
    },

    ///Inspect crash on specified path or the last one
    Inspect {
        ///Path to crash file
        crash_file: Option<PathBuf>,
    },

    ///Commands applied to managing crashes
    Crash{
        #[command(subcommand)]
        crash_subcommand: CrashSubcommands
    },

    ///Commands applied to managing tickets
    Ticket{
        #[command(subcommand)]
        ticket_subcommand: TicketSubcommand
    }
}

#[derive(Subcommand)]
pub(crate) enum CrashSubcommands{
    ///Creates new crash
    New{
        ///Error code like ERR0001
        error_code: String,
        ///Path to file where error occurs
        file_path: PathBuf,
        ///Span in format LINE:COLUMN[:LENGTH] (12:23:5)
        span: CrashSpan,
        
        ///Original error message
        #[arg(short, long)]
        msg: Option<String>,
        ///Original invocation trace of error
        #[arg(short, long)]
        trace: Option<Vec<String>>,
        ///Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    ///Prints all local crashes
    List{
        ///Project root path
        #[arg(default_value_os_t = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))]
        path: PathBuf,
    },
    ///Deletes all local crashes
    Clear{
        ///Project root path
        #[arg(default_value_os_t = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))]
        path: PathBuf,
    },
}

#[derive(Subcommand)]
pub(crate) enum TicketSubcommand{
    ///Creates new ticket
    New{
        ///Crash file that ticket will be based on
        crash_file: Option<PathBuf>,
        ///Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    ///Pushes ticket to error's dev remote or to custom remote
    Push{
        ///Push choice
        #[command(subcommand)]
        target: Option<PushTarget>,
        ///Custom remote url (overrides dev's one)
        #[arg(short, long)]
        remote: Option<Url>,
    },
    ///Prints all local tickets
    List{
        ///Project root path
        #[arg(default_value_os_t = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))]
        path: PathBuf,
    },
    ///Deletes all local tickets
    Clear{
        ///Project root path
        #[arg(default_value_os_t = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))]
        path: PathBuf,
    },
}

#[derive(Subcommand, Clone)]
pub(crate) enum PushTarget{
    ///Push last created ticket [default]
    Last,
    ///Push single file from specified path
    Single {
        #[arg(value_parser)]
        path: PathBuf
    },
    ///Push not pushed tickets
    Unpushed,
    ///Push all tickets
    All,
}