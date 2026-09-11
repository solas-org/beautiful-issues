use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::lock::Lock;

#[derive(Serialize, Deserialize)]
pub(crate) struct Crash {
    pub(crate) error_code: String,
    pub(crate) file_path: PathBuf,
    pub(crate) span: CrashSpan,
    pub(crate) message: String,
    pub(crate) invocation_trace: Vec<String>,
    pub(crate) context: CrashContext
}

impl Crash {
    pub(crate) fn save(&self, output: Option<PathBuf>) {
        let generated_name = Utc::now().format("%Y-%m-%d_%H-%M-%S");
        let path = output.unwrap_or_else(||
            PathBuf::from(format!("./.bissue/crashes/{}_{}.crash", &self.error_code, generated_name)));

        let file_content = serde_json::to_string_pretty(&self).expect("Cannot serialize crash file");

        let mut file = File::create(&path).expect("Cannot open crash file");

        file.write_all(file_content.as_bytes()).expect("Cannot write crash file content");

        let mut lock = Lock::read();
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
    timestamp: i64
}

impl CrashContext{
    pub(crate) fn generate(target_framework: String) -> CrashContext{
        let timestamp = Utc::now().timestamp();
        let os = std::env::consts::OS.to_string();
        CrashContext{target_framework, os, timestamp}
    }
}