use crate::configs::{DevConfig, UserConfig};
use crate::lock::Lock;
use crate::render::styling::parse_hex;
use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::fs::create_dir_all;
use std::path::Path;
use toml_comment::TomlComment;

pub(crate) fn init(project_path: &Path) -> Result<()> {
    let dir = project_path.join(".bissue");

    create_dir_all(&dir).with_context(|| format!("Cannot create directory {}", dir.display()))?;
    create_dir_all(dir.join("crashes"))?;
    create_dir_all(dir.join("tickets"))?;
    create_dir_all(dir.join("issues"))?;
    create_dir_all(dir.join("issues/.pkgs"))?;

    let create_file_if_missing = |name: &str, content: &[u8]| -> Result<()> {
        let path = dir.join(name);
        if !path.exists() {
            std::fs::write(&path, content)
                .with_context(|| format!("Cannot write file at {}", path.display()))?;
        }
        Ok(())
    };

    let dev_config = DevConfig::default();
    let dev_content = dev_config.to_commented_toml();
    create_file_if_missing("dev_config.toml", dev_content.as_bytes())?;

    let user_config = UserConfig::default();
    let user_content = user_config.to_commented_toml();
    create_file_if_missing("user_config.toml", user_content.as_bytes())?;

    let lock = Lock::default();
    let lock_content = toml::to_string(&lock).context("Cannot serialize lock file")?;
    create_file_if_missing("bissue.lock", lock_content.as_bytes())?;

    let (ok_r, ok_g, ok_b) = parse_hex(&user_config.ok_color);

    println!(
        "Beautiful issues have been {} initialized in {} directory",
        "successfully".bold().truecolor(ok_r, ok_g, ok_b),
        dir.display().italic()
    );

    Ok(())
}
