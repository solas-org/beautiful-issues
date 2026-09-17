use anyhow::{Context, Result};
use chrono::Utc;
use clap::Args;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::configs::{find_bissue_dir, UserConfig};
use crate::lock::Lock;
use crate::render::styling::parse_hex;

#[derive(Args, Clone, Serialize, Deserialize)]
pub(crate) struct Ticket {
    ///One-line description for your problem
    header: String,
    ///Full description of your problem
    description: String,
    ///Specific crash file that ticket will be based on
    crash_file: Option<PathBuf>,
    ///List of all diagnostics in crash [default: all]
    diagnostics: Vec<u32>,
}

impl Ticket {
    pub(crate) fn save(&self, output: Option<PathBuf>) -> Result<PathBuf> {
        let user_config = UserConfig::global();

        let now = Utc::now().format("%Y-%m-%d_%H-%M-%S");
        let prefix = self.header.trim();

        let path = output.unwrap_or_else(|| {
            find_bissue_dir().join("tickets").join(format!("{prefix}_{now}.ticket"))
        });

        let canon = dunce::canonicalize(&path).unwrap_or(path);

        if let Some(parent) = canon.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Cannot create directory {}", parent.display()))?;
        }

        let file_content = serde_json::to_string_pretty(&self)
            .context("Cannot serialize ticket file")?;

        std::fs::write(&canon, file_content.as_bytes())
            .with_context(|| format!("Cannot create ticket file at {}", canon.display()))?;

        let (ok_r, ok_g, ok_b) = parse_hex(&user_config.ok_color);
        println!(
            "{} {}",
            "Successfully created ticket file at path:".truecolor(ok_r, ok_g, ok_b).bold(),
            canon.display().italic()
        );

        let mut lock = Lock::load();
        lock.last_ticket = canon.clone();
        lock.write()?;

        Ok(canon)
    }
}

#[derive(Serialize)]
pub(crate) struct GithubTicket{
    title: String,
    body: String,
    labels: Vec<String>
}

impl GithubTicket {
    pub(crate) fn create(ticket: &Ticket) -> Result<GithubTicket>{
        let title = ticket.header.clone();
        let body = ticket.description.clone();

        Ok(GithubTicket{title, body, labels: Vec::new()})
    }
}