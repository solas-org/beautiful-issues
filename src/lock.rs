use std::fs::{read, File};
use std::io::Write;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct Lock{
    pub(crate) pushed: Vec<PathBuf>
}

impl Lock {
    pub(crate) fn write(lock: Lock, mut path: PathBuf){
        let file_content = toml::to_string(&lock).expect("Cannot serialize lock file");

        path.push("bissue.lock");

        let mut file = File::open(&path).unwrap_or_else(|_| File::create(&path).expect("Cannot open lock file"));

        file.write_all(file_content.as_bytes()).expect("Cannot write lock file content");
    }

    pub(crate) fn read() -> Lock{
        let read = read("bissue.lock").expect("Cannot read lock file");
        toml::from_slice(&read).expect("Cannot deserialize lock file")
    }
}