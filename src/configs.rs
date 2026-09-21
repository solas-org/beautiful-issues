use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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
    #[allow(dead_code)]
    pub(crate) fn load() -> DevConfig {
        let path = find_bissue_dir().join("dev_config.toml");
        if let Ok(read) = std::fs::read(&path)
            && let Ok(cfg) = toml::from_slice(&read)
        {
            return cfg;
        }
        DevConfig::default()
    }

    pub(crate) fn load_from_path(path: &Path) -> DevConfig {
        if let Ok(read) = std::fs::read(path)
            && let Ok(cfg) = toml::from_slice(&read)
        {
            return cfg;
        }
        DevConfig::default()
    }

    #[allow(dead_code)]
    pub(crate) fn global() -> &'static DevConfig {
        static INSTANCE: OnceLock<DevConfig> = OnceLock::new();
        INSTANCE.get_or_init(Self::load)
    }
}

fn default_ok_color() -> String { "#a6e3a1".to_string() }
fn default_err_color() -> String { "#f38ba8".to_string() }
fn default_warn_color() -> String { "#f9e2af".to_string() }
fn default_info_color() -> String { "#89a0e0".to_string() }
fn default_border_color() -> String { "#637296".to_string() }
fn default_text_color() -> String { "#cdd6f4".to_string() }
fn default_text_invert_color() -> String { "#000000".to_string() }
fn default_subtext_color() -> String { "#a6adc8".to_string() }
fn default_accent_color() -> String { "#d7b5ff".to_string() }
fn default_select_symbol() -> String { "››".to_string() }
fn default_choose_symbol() -> String { "✓".to_string() }

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct UserConfig {
    pub(crate) remotes: Vec<String>,
    #[serde(default = "default_ok_color")]
    pub(crate) ok_color: String,
    #[serde(default = "default_err_color")]
    pub(crate) err_color: String,
    #[serde(default = "default_warn_color")]
    pub(crate) warn_color: String,
    #[serde(default = "default_info_color")]
    pub(crate) info_color: String,
    #[serde(default = "default_border_color")]
    pub(crate) border_color: String,
    #[serde(default = "default_text_color")]
    pub(crate) text_color: String,
    #[serde(default = "default_text_invert_color")]
    pub(crate) text_invert_color: String,
    #[serde(default = "default_subtext_color")]
    pub(crate) subtext_color: String,
    #[serde(default = "default_accent_color", alias = "highlight_color")]
    pub(crate) accent_color: String,
    #[serde(default = "default_select_symbol")]
    pub(crate) select_symbol: String,
    #[serde(default = "default_choose_symbol")]
    pub(crate) choose_symbol: String,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            remotes: Vec::new(),
            ok_color: default_ok_color(),
            err_color: default_err_color(),
            warn_color: default_warn_color(),
            info_color: default_info_color(),
            border_color: default_border_color(),
            text_color: default_text_color(),
            text_invert_color: default_text_invert_color(),
            subtext_color: default_subtext_color(),
            accent_color: default_accent_color(),
            select_symbol: default_select_symbol(),
            choose_symbol: default_choose_symbol(),
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
