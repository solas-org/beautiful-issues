use crate::commands::enums::Framework;
use crate::configs::{UserConfig, find_bissue_dir};
use crate::lock::Lock;
use crate::models::issue::Issue;
use crate::render::styling::parse_hex;
use anyhow::{Context, Result};
use chrono::Utc;
use clap::Args;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
pub(crate) struct CrashFile {
    pub(crate) crashes: Vec<Diagnostic>,
    pub(crate) context: CrashContext,
}

#[derive(Args, Clone, Serialize, Deserialize)]
pub(crate) struct Diagnostic {
    ///Error code like ERR0001
    pub(crate) issue_code: String,
    ///Path to file where error occurs
    pub(crate) file_path: PathBuf,
    ///Span in format LINE:COLUMN[:LENGTH] (12:23:5)
    pub(crate) span: CrashSpan,
    ///Original invocation trace of error
    pub(crate) trace: Vec<String>,
}

impl FromStr for Diagnostic {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() < 4 {
            return Err("Expected format: 'CODE|FILE_PATH|SPAN[|TRACE1,TRACE2,...]'".to_string());
        }

        let error_code = parts[0].trim().to_string();
        let file_path = PathBuf::from(parts[1].trim());
        let span = CrashSpan::from_str(parts[2].trim())?;

        let trace = if parts.len() > 4 && !parts[4].trim().is_empty() {
            parts[4].split(',').map(|t| t.trim().to_string()).collect()
        } else {
            Vec::new()
        };

        Ok(Diagnostic {
            issue_code: error_code,
            file_path,
            span,
            trace,
        })
    }
}

impl Diagnostic {
    pub(crate) fn get_issue(&self) -> Result<Issue> {
        let bissue_dir = find_bissue_dir().join("issues");
        let target_filename = format!("{}.issue", self.issue_code);

        let issue_path = WalkDir::new(&bissue_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && e.file_name().to_str() == Some(&target_filename))
            .map(|e| e.into_path())
            .nth(0)
            .context(format!(
                "Couldn't find issue with code: {} in folder {}",
                self.issue_code,
                bissue_dir.display()
            ))?;

        Ok(serde_json::from_slice(&*fs::read(issue_path)?)?)
    }
}

impl CrashFile {
    pub(crate) fn save(&self, output: Option<PathBuf>) -> Result<PathBuf> {
        let user_config = UserConfig::global();

        let generated_name = Utc::now().format("%Y-%m-%d_%H-%M-%S");
        let path = output.unwrap_or_else(|| {
            find_bissue_dir()
                .join("crashes")
                .join(format!("{generated_name}.crash"))
        });

        let canon = dunce::canonicalize(&path).unwrap_or(path);

        if let Some(parent) = canon.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Cannot create directory {}", parent.display()))?;
        }

        let file_content =
            serde_json::to_string_pretty(&self).context("Cannot serialize crash file")?;

        std::fs::write(&canon, file_content.as_bytes())
            .with_context(|| format!("Cannot create crash file at {}", canon.display()))?;

        let (ok_r, ok_g, ok_b) = parse_hex(&user_config.ok_color);
        println!(
            "{} {}",
            "Successfully created crash file at path:"
                .truecolor(ok_r, ok_g, ok_b)
                .bold(),
            canon.display().italic()
        );

        let mut lock = Lock::load();
        lock.last_crash = canon.clone();
        lock.write()?;

        Ok(canon)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct CrashSpan {
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) len: Option<usize>,
}

impl Display for CrashSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self.len {
            Some(len) => format!("{}:{}:{len}", self.line, self.column),
            None => format!("{}:{}", self.line, self.column),
        };
        write!(f, "{}", str)
    }
}

impl FromStr for CrashSpan {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();

        if parts.len() < 2 || parts.len() > 3 {
            return Err("should be LINE:COL or LINE:COL:LEN".to_string());
        }

        let line = parts[0]
            .parse::<usize>()
            .map_err(|_| format!("wrong line number '{}'", parts[0]))?;

        let col = parts[1]
            .parse::<usize>()
            .map_err(|_| format!("wrong column number '{}'", parts[1]))?;

        let len = if parts.len() == 3 {
            Some(
                parts[2]
                    .parse::<usize>()
                    .map_err(|_| format!("wrong length '{}'", parts[2]))?,
            )
        } else {
            None
        };

        Ok(CrashSpan {
            line,
            column: col,
            len,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CrashContext {
    pub(crate) target_framework: String,
    pub(crate) os: String,
    pub(crate) timestamp: i64,
}

impl CrashContext {
    pub(crate) fn generate(target_framework: &Framework) -> CrashContext {
        let timestamp = Utc::now().timestamp();
        let os = std::env::consts::OS.to_string();
        CrashContext {
            target_framework: target_framework.to_string(),
            os,
            timestamp,
        }
    }
}
