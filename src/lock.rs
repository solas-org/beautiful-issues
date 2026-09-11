use std::fs::{read, File};
use std::io::Write;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::commands::enums::Framework;

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct Lock{
    pub(crate) frameworks: Vec<Framework>,
    pub(crate) last_issue: PathBuf,
    pub(crate) last_crash: PathBuf,
    pub(crate) last_ticket: PathBuf,
    pub(crate) pushed: Vec<PathBuf>
}

impl Lock {
    pub(crate) fn write(lock: &Lock){
        let file_content = toml::to_string(&lock).expect("Cannot serialize lock file");

        let path = PathBuf::from("./.bissue/bissue.lock");

        let mut file = File::create(&path).expect("Cannot open lock file");

        file.write_all(file_content.as_bytes()).expect("Cannot write lock file content");
    }

    pub(crate) fn read() -> Lock{
        let read = read(PathBuf::from("./.bissue/bissue.lock")).expect("Cannot read lock file");
        toml::from_slice(&read).expect("Cannot deserialize lock file")
    }

    pub(crate) fn get_all_frameworks(&self) -> String{
        let strings: Vec<String> = self.frameworks.iter().map(|f| f.to_string()).collect();
        strings.join("|")
    }
}