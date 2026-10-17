use crate::configs::{UserConfig, find_bissue_dir};
use crate::lock::Lock;
use crate::render::styling::parse_hex;
use anyhow::{Context, Result};
use clap::{Args, ValueEnum};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(ValueEnum, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum IssueType {
    Error,
    Warning,
    Fatal,
}

#[derive(Args, Serialize, Deserialize, Clone)]
pub(crate) struct Issue {
    ///Error – not critical error in code, Warning – possible future error or bad practice, Fatal – critical error (immediately crashes)
    pub(crate) issue_type: Option<IssueType>,
    ///Code of issue (example: ERR001)
    pub(crate) code: Option<String>,
    ///Issue short description
    pub(crate) message: Option<String>,
    ///Some optional data about issue
    pub(crate) note: Option<String>,
    ///Help to avoid issue, links to helpful resources, etc.
    pub(crate) advice: Option<String>,
}

impl Issue {
    pub(crate) fn new(
        issue_type: Option<IssueType>,
        code: Option<String>,
        message: Option<String>,
        note: Option<String>,
        advice: Option<String>,
    ) -> Self {
        Self {
            issue_type,
            code,
            message,
            note,
            advice,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.code.is_none() && self.message.is_none()
    }

    pub(crate) fn save(&self, output: Option<PathBuf>) -> Result<PathBuf> {
        let user_config = UserConfig::global();

        let file_name = match &self.code {
            Some(code) if !code.trim().is_empty() => crate::models::sanitize_filename(code),
            _ => chrono::Utc::now()
                .format("issue_%Y-%m-%d_%H-%M-%S")
                .to_string(),
        };
        let path = output.unwrap_or_else(|| {
            find_bissue_dir()
                .join("issues")
                .join(format!("{file_name}.issue"))
        });

        let canon = dunce::canonicalize(&path).unwrap_or(path);

        if let Some(parent) = canon.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Cannot create directory {}", parent.display()))?;
        }

        let file_content =
            serde_json::to_string_pretty(&self).context("Cannot serialize issue file")?;

        std::fs::write(&canon, file_content.as_bytes())
            .with_context(|| format!("Cannot create issue file at {}", canon.display()))?;

        let (ok_r, ok_g, ok_b) = parse_hex(&user_config.ok_color);
        println!(
            "{} {}",
            "Successfully created issue file at path:"
                .truecolor(ok_r, ok_g, ok_b)
                .bold(),
            canon.display().italic()
        );

        let mut lock = Lock::load();
        lock.last_issue = canon.clone();
        lock.write()?;

        Ok(canon)
    }
}
