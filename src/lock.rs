use crate::commands::enums::Framework;
use crate::configs::find_bissue_dir;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct Lock {
    pub(crate) frameworks: Vec<Framework>,
    pub(crate) last_issue: PathBuf,
    pub(crate) last_crash: PathBuf,
    pub(crate) last_ticket: PathBuf,
    pub(crate) pushed: Vec<PathBuf>,
}

impl Lock {
    pub(crate) fn path() -> PathBuf {
        find_bissue_dir().join("bissue.lock")
    }

    pub(crate) fn write(&self) -> Result<()> {
        let file_content = toml::to_string(self).context("Cannot serialize lock file")?;
        let path = Self::path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&path, file_content).context("Cannot write lock file content")?;
        Ok(())
    }

    pub(crate) fn load() -> Lock {
        let path = Self::path();
        if let Ok(bytes) = std::fs::read(&path)
            && let Ok(lock) = toml::from_slice(&bytes)
        {
            return lock;
        }
        Lock::default()
    }

    pub(crate) fn get_all_frameworks(&self) -> String {
        let strings: Vec<String> = self.frameworks.iter().map(|f| f.to_string()).collect();
        strings.join("|")
    }
}