use crate::configs::UserConfig;
use crate::lock::Lock;
use crate::render::styling::parse_hex;
use crate::render::unwrappers::{SafeUnwrap, create_file};
use chrono::Utc;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use clap::Args;
use crate::commands::enums::Framework;

#[derive(Serialize, Deserialize)]
pub(crate) struct CrashFile {
    pub(crate) crashes: Vec<Crash>,
    pub(crate) context: CrashContext
}

#[derive(Args, Clone, Serialize, Deserialize)]
pub(crate) struct Crash {
    ///Error code like ERR0001
    pub(crate) error_code: String,
    ///Path to file where error occurs
    pub(crate) file_path: PathBuf,
    ///Span in format LINE:COLUMN[:LENGTH] (12:23:5)
    pub(crate) span: CrashSpan,
    ///Original error message
    #[arg(short, long)]
    pub(crate) message: String,
    ///Original invocation trace of error
    #[arg(short, long)]
    pub(crate) trace: Vec<String>,
}

impl FromStr for Crash {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() < 4 {
            return Err("Expected format: 'CODE|FILE_PATH|SPAN|MESSAGE[|TRACE1,TRACE2]'".to_string());
        }

        let error_code = parts[0].trim().to_string();
        let file_path = PathBuf::from(parts[1].trim());
        let span = CrashSpan::from_str(parts[2].trim())?;
        let message = parts[3].trim().to_string();

        let trace = if parts.len() > 4 && !parts[4].trim().is_empty() {
            parts[4]
                .split(',')
                .map(|t| t.trim().to_string())
                .collect()
        } else {
            Vec::new()
        };

        Ok(Crash {
            error_code,
            file_path,
            span,
            message,
            trace,
        })
    }
}

impl CrashFile {
    pub(crate) fn save(&self, output: Option<PathBuf>) {
        let user_config = UserConfig::global();

        let generated_name = Utc::now().format("%Y-%m-%d_%H-%M-%S");
        let path = output.unwrap_or_else(||
            PathBuf::from(format!("./.bissue/crashes/{generated_name}.crash")));

        let file_content = serde_json::to_string_pretty(&self).unwrap_safe("Cannot serialize crash file");

        let mut file = create_file(&path, "Cannot create crash file");

        file.write_all(file_content.as_bytes()).unwrap_safe("Cannot create crash file");

        let (ok_r, ok_g, ok_b) = parse_hex(&*user_config.ok_color);
        println!("{} {}",
                 "Successfully created crash file at path:".truecolor(ok_r,ok_g,ok_b).bold(),
                 &path.display().italic());

        let mut lock = Lock::global_clone();
        lock.last_crash = path;
        Lock::write(&lock);
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct CrashSpan{
    line: usize,
    column: usize,
    len: Option<usize>,
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

        Ok(CrashSpan { line, column: col, len })
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CrashContext{
    target_framework: String,
    os: String,
    timestamp: i64,
}

impl CrashContext{
    pub(crate) fn generate(framework: &Option<Framework>) -> CrashContext{
        let mut target_framework  = Lock::global_ref().get_all_frameworks();
        if framework.is_some(){
            target_framework = framework.as_ref().unwrap().to_string();
        }
        let timestamp = Utc::now().timestamp();
        let os = std::env::consts::OS.to_string();
        CrashContext{target_framework, os, timestamp}
    }
}