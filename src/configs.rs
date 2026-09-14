use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;

pub(crate) fn find_bissue_dir() -> PathBuf {
    if let Ok(mut cur) = std::env::current_dir() {
        loop {
            let candidate = cur.join(".bissue");
            if candidate.is_dir() {
                return candidate;
            }
            if !cur.pop() {
                break;
            }
        }
    }
    PathBuf::from("./.bissue")
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct DevConfig {
    pub(crate) remotes: Vec<String>,
}

impl DevConfig {
    pub(crate) fn load() -> DevConfig {
        let path = find_bissue_dir().join("dev_config.toml");
        if let Ok(read) = std::fs::read(&path)
            && let Ok(cfg) = toml::from_slice(&read)
        {
            return cfg;
        }
        DevConfig::default()
    }

    pub(crate) fn global() -> &'static DevConfig {
        static INSTANCE: OnceLock<DevConfig> = OnceLock::new();
        INSTANCE.get_or_init(Self::load)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct UserConfig {
    pub(crate) remotes: Vec<String>,
    pub(crate) ok_color: String,
    pub(crate) err_color: String,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            remotes: Vec::new(),
            ok_color: "#42f5b6".to_string(),
            err_color: "#f54266".to_string(),
        }
    }
}

impl UserConfig {
    pub(crate) fn load() -> UserConfig {
        let path = find_bissue_dir().join("user_config.toml");
        if let Ok(read) = std::fs::read(&path)
            && let Ok(cfg) = toml::from_slice(&read)
        {
            return cfg;
        }
        UserConfig::default()
    }

    pub(crate) fn global() -> &'static UserConfig {
        static INSTANCE: OnceLock<UserConfig> = OnceLock::new();
        INSTANCE.get_or_init(Self::load)
    }
}
