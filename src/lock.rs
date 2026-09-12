use crate::commands::enums::Framework;
use crate::render::unwrappers::{SafeUnwrap, create_file, read};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct Lock{
    pub(crate) frameworks: Vec<Framework>,
    pub(crate) last_issue: PathBuf,
    pub(crate) last_crash: PathBuf,
    pub(crate) last_ticket: PathBuf,
    pub(crate) pushed: Vec<PathBuf>
}

impl Lock {
    pub(crate) fn write(lock: &Lock){
        let file_content = toml::to_string(&lock).unwrap_safe("Cannot serialize lock file");

        let path = PathBuf::from("./.bissue/bissue.lock");

        let mut file = create_file(&path, "Cannot open lock file");

        file.write_all(file_content.as_bytes()).unwrap_safe("Cannot write lock file content");
    }

    fn load() -> Lock {
        let read = read(&PathBuf::from("./.bissue/bissue.lock"),"Cannot read lock file");
        toml::from_slice(&read).unwrap_safe("Cannot deserialize lock file")
    }

    pub(crate) fn global_clone() -> Lock {
        static INSTANCE: OnceLock<Lock> = OnceLock::new();
        INSTANCE.get_or_init(Self::load).clone()
    }

    pub(crate) fn global_ref() -> &'static Lock {
        static INSTANCE: OnceLock<Lock> = OnceLock::new();
        INSTANCE.get_or_init(Self::load)
    }

    pub(crate) fn get_all_frameworks(&self) -> String{
        let strings: Vec<String> = self.frameworks.iter().map(|f| f.to_string()).collect();
        strings.join("|")
    }
}