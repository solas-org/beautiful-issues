use crate::models::crash::Diagnostic;
use crate::models::issue::Issue;
use crate::models::ticket::Ticket;
use clap::{Subcommand, ValueEnum};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

#[derive(Subcommand)]
pub(crate) enum Commands {
    ///Create .bissue folder in root of your project and initiate there
    Init {
        ///Project root path
        #[arg(default_value_os_t = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))]
        path: PathBuf,
    },

    ///Inspect crash on specified path or the last one
    Inspect {
        ///Path to crash file
        crash_file: Option<PathBuf>,
    },

    ///Issues are custom errors, warnings or suggestions for helping users understand the problem
    Issue {
        #[command(subcommand)]
        subcommand: IssueSubcommands,
    },

    ///Crashes are auto-generated files that record program issues
    Crash {
        #[command(subcommand)]
        subcommand: CrashSubcommands,
    },

    ///Tickets are user-generated reports of crashes and other problems
    Ticket {
        #[command(subcommand)]
        subcommand: TicketSubcommand,
    },

    ///Packages are archives with devs issues that user can include manually or via hook
    Package {
        #[command(subcommand)]
        subcommand: PackageSubcommand,
    },
}

#[derive(ValueEnum, Clone, Serialize, Deserialize, Display, PartialEq, Eq, Hash)]
pub(crate) enum Framework {
    Dotnet,
    Rust,
}

#[derive(Subcommand)]
pub(crate) enum IssueSubcommands {
    ///Create new issue
    New {
        #[command(flatten)]
        issue: Issue,
        ///Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    ///Print all issues
    List,
    ///Delete issues
    Del {
        #[command(subcommand)]
        target: Option<DelTarget>,

        ///Delete without confirmation
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum CrashSubcommands {
    ///Create new crash
    New {
        ///List of all issues happened in that crash (diagnostic format: CODE|FILE_PATH|SPAN|MESSAGE[|TRACE1,TRACE2,...])
        #[arg(required = true)]
        diagnostics: Vec<Diagnostic>,
        ///Target framework
        #[arg(short, long)]
        framework: Option<Framework>,
        ///Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    ///Print all crashes
    List,
    ///Delete crashes
    Del {
        #[command(subcommand)]
        target: Option<DelTarget>,

        ///Delete without confirmation
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum TicketSubcommand {
    ///Create new ticket
    New {
        #[command(flatten)]
        ticket: Ticket,
        ///Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    ///Push ticket to remote
    Push {
        #[command(subcommand)]
        target: Option<PushTarget>,

        ///Dev remotes are servers that developers of issue left.
        ///User remotes you can change in config
        #[arg(short, long("type"), default_value = "all")]
        remote_type: PushRemote,
        ///Custom remotes for push (used only if remote type is custom)
        #[arg(short, long("custom"))]
        custom_remotes: Vec<Url>,
    },
    ///Print all tickets
    List,
    ///Delete tickets
    Del {
        #[command(subcommand)]
        target: Option<DelTarget>,

        ///Delete without confirmation
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum PackageSubcommand {
    ///Generate tar archive with zstd compression, includes dev config and issues.
    ///Output archive name is bissue.tar.zstd
    Gen {
        ///Custom folder path of generated archive output
        path: Option<PathBuf>,

        ///Package will include dependencies only with selected frameworks
        #[arg(short, long)]
        frameworks: Vec<Framework>,
    },
    ///Add issues from package
    Add {
        ///Framework type
        framework: Framework,
        ///Name of package in framework
        name: String,
        ///Version of package in framework
        version: String,
        ///Path of archive
        path: PathBuf,
    },
    ///Delete issues from package
    Del {
        ///Framework type
        framework: Framework,
        ///Name of package in framework
        name: String,
    },
    ///Return current packages versions and prune not specified in input framework
    Check {
        ///Framework type
        framework: Framework,
        ///Names of packages in framework to check
        names: Vec<String>,
    },
}

#[derive(Subcommand, Clone)]
pub(crate) enum DelTarget {
    ///Delete last [default]
    Last,
    ///Delete single from specified path
    Single {
        #[arg(value_parser)]
        path: PathBuf,
    },
    ///Delete all
    All,
}

#[derive(Subcommand, Clone)]
pub(crate) enum PushTarget {
    ///Push last created ticket [default]
    Last,
    ///Push single file from specified path
    Single {
        #[arg(value_parser)]
        path: PathBuf,
    },
    ///Push not pushed tickets
    Unpushed,
    ///Push all tickets
    All,
}

#[derive(ValueEnum, Clone)]
pub(crate) enum PushRemote {
    All,
    Dev,
    User,
    Custom,
}
