use crate::models::crash::CrashSpan;
use clap::{Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use derive_more::Display;
use url::Url;

#[derive(Subcommand)]
pub(crate) enum Commands {
    ///Create .bissue folder in root of your project and initiate there
    Init{
        ///Project root path
        #[arg(default_value_os_t = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))]
        path: PathBuf,
        ///Target frameworks
        frameworks: Vec<Framework>,
    },

    ///Inspect crash on specified path or the last one
    Inspect {
        ///Path to crash file
        crash_file: Option<PathBuf>,
    },

    ///Issues are custom errors, warnings or suggestions for helping users understand the problem
    Issue{
        #[command(subcommand)]
        subcommand: IssueSubcommands
    },

    ///Crashes are auto-generated files that record program issues
    Crash{
        #[command(subcommand)]
        subcommand: CrashSubcommands
    },

    ///Tickets are user-generated reports of crashes and other problems
    Ticket{
        #[command(subcommand)]
        subcommand: TicketSubcommand
    }
}

#[derive(ValueEnum, Clone, Serialize, Deserialize, Display)]
pub(crate) enum Framework{
    Dotnet,
    Rust
}

#[derive(Subcommand)]
pub(crate) enum IssueSubcommands{
    ///Interactively create new issue
    New{
        ///Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    ///Print all issues
    List,
    ///Delete issues
    Del{
        #[command(subcommand)]
        target: Option<DelTarget>,
    },
}

#[derive(Subcommand)]
pub(crate) enum CrashSubcommands{
    ///Create new crash
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
        ///Target framework
        #[arg(short, long)]
        framework: Option<String>,
        ///Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    ///Print all crashes
    List,
    ///Delete crashes
    Del{
        #[command(subcommand)]
        target: Option<DelTarget>,
    },
}

#[derive(Subcommand)]
pub(crate) enum TicketSubcommand{
    ///Interactively create new ticket
    New{
        ///Crash file that ticket will be based on
        crash_file: Option<PathBuf>,
        ///Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    ///Push ticket to error's dev remote or to custom remote
    Push{
        #[command(subcommand)]
        target: Option<PushTarget>,
        ///Custom remote url (overrides dev's one)
        #[arg(short, long)]
        remote: Option<Url>,
    },
    ///Print all tickets
    List,
    ///Delete tickets
    Del{
        #[command(subcommand)]
        target: Option<DelTarget>,
    },
}

#[derive(Subcommand, Clone)]
pub(crate) enum DelTarget{
    ///Delete last [default]
    Last,
    ///Delete single from specified path
    Single {
        #[arg(value_parser)]
        path: PathBuf
    },
    ///Delete all
    All,
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