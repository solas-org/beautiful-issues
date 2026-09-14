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
    header: Option<String>,
    ///Full description of your problem
    description: Option<String>,
    ///Specific crash file that ticket will be based on
    crash_file: Option<PathBuf>,
    ///List of all diagnostics in crash [default: all]
    diagnostics: Vec<u32>,
}

impl Ticket {
    pub(crate) fn save(&self, output: Option<PathBuf>) -> Result<PathBuf> {
        let user_config = UserConfig::global();

        let now = Utc::now().format("%Y-%m-%d_%H-%M-%S");
        let prefix = self
            .header
            .as_deref()
            .map(|h| h.trim())
            .filter(|h| !h.is_empty())
            .unwrap_or("ticket");

        let path = output.unwrap_or_else(|| {
            find_bissue_dir().join(format!("tickets/{prefix}_{now}.ticket"))
        });

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Cannot create directory {}", parent.display()))?;
        }

        let file_content = serde_json::to_string_pretty(&self)
            .context("Cannot serialize ticket file")?;

        std::fs::write(&path, file_content.as_bytes())
            .with_context(|| format!("Cannot create ticket file at {}", path.display()))?;

        let (ok_r, ok_g, ok_b) = parse_hex(&user_config.ok_color);
        println!(
            "{} {}",
            "Successfully created ticket file at path:".truecolor(ok_r, ok_g, ok_b).bold(),
            path.display().italic()
        );

        let mut lock = Lock::load();
        lock.last_ticket = path.clone();
        lock.write()?;

        Ok(path)
    }
}