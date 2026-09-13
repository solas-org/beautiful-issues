use std::io::Write;
use std::path::PathBuf;
use chrono::Utc;
use clap::Args;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use crate::configs::UserConfig;
use crate::lock::Lock;
use crate::render::styling::parse_hex;
use crate::render::unwrappers::{create_file, SafeUnwrap};

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
    pub(crate) fn save(&self, output: Option<PathBuf>){
        let user_config = UserConfig::global();

        let now = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let mut generated_name = String::new();
        generated_name = self.header.as_ref().unwrap_or_else(|| &generated_name).to_string();
        let path = output.unwrap_or_else(||
            PathBuf::from(format!("./.bissue/issues/{generated_name}_{now}.ticket")));

        let file_content = serde_json::to_string_pretty(&self).unwrap_safe("Cannot serialize ticket file");

        let mut file = create_file(&path, "Cannot create ticket file");

        file.write_all(file_content.as_bytes()).unwrap_safe("Cannot create ticket file");

        let (ok_r, ok_g, ok_b) = parse_hex(&*user_config.ok_color);
        println!("{} {}",
                 "Successfully created ticket file at path:".truecolor(ok_r,ok_g,ok_b).bold(),
                 &path.display().italic());

        let mut lock = Lock::global_clone();
        lock.last_ticket = path;
        Lock::write(&lock);
    }
}