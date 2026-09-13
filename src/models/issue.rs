use crate::configs::UserConfig;
use crate::lock::Lock;
use crate::render::styling::parse_hex;
use crate::render::unwrappers::{SafeUnwrap, create_file};
use clap::{Args, ValueEnum};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

#[derive(ValueEnum, Serialize, Deserialize, Clone)]
pub(crate) enum IssueType{ Error, Warning, Suggestion }

#[derive(Args, Serialize, Deserialize)]
pub(crate) struct Issue {
    issue_type: Option<IssueType>,
    code: Option<String>,
    message: Option<String>,
    note: Option<String>,
    advice: Option<String>,
}

impl Issue {
    pub(crate) fn save(&self, output: Option<PathBuf>){
        let user_config = UserConfig::global();

        let mut generated_name = &String::new();
        generated_name = self.code.as_ref().unwrap_or_else(|| generated_name);
        let path = output.unwrap_or_else(||
            PathBuf::from(format!("./.bissue/issues/{generated_name}.issue")));

        let file_content = serde_json::to_string_pretty(&self).unwrap_safe("Cannot serialize issue file");

        let mut file = create_file(&path, "Cannot create issue file");

        file.write_all(file_content.as_bytes()).unwrap_safe("Cannot create issue file");

        let (ok_r, ok_g, ok_b) = parse_hex(&*user_config.ok_color);
        println!("{} {}",
                 "Successfully created issue file at path:".truecolor(ok_r,ok_g,ok_b).bold(),
                 &path.display().italic());

        let mut lock = Lock::global_clone();
        lock.last_issue = path;
        Lock::write(&lock);
    }
}
