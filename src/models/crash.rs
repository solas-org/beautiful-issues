use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct Crash {
    error_code: String,
    path: String,
    span: CrashSpan,
    message: String,
    invocation_trace: Vec<String>,
    context: CrashContext
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
    timestamp: u8
}