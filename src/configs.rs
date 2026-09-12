use serde::{Deserialize, Serialize};
use std::fs::read;
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Serialize, Deserialize)]
pub(crate) struct DevConfig {
    pub(crate) remotes: Vec<String>,
}

impl DevConfig {
    fn load() -> DevConfig {
        let read =
            read(PathBuf::from("./.bissue/dev_config.toml")).expect("Cannot read dev config file");
        toml::from_slice(&read).expect("Cannot deserialize dev config file")
    }

    pub(crate) fn global() -> &'static DevConfig {
        static INSTANCE: OnceLock<DevConfig> = OnceLock::new();
        INSTANCE.get_or_init(Self::load)
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct UserConfig {
    pub(crate) ok_color: String,
    pub(crate) err_color: String,
}

impl UserConfig {
    fn load() -> UserConfig {
        let read = read(PathBuf::from("./.bissue/user_config.toml"))
            .expect("Cannot read user config file");
        toml::from_slice(&read).expect("Cannot deserialize user config file")
    }

    pub(crate) fn global() -> &'static UserConfig {
        static INSTANCE: OnceLock<UserConfig> = OnceLock::new();
        INSTANCE.get_or_init(Self::load)
    }
}
