use std::fs::create_dir_all;
use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::path::Path;
use crate::commands::enums::Framework;
use crate::configs::{DevConfig, UserConfig};
use crate::lock::Lock;
use crate::render::styling::parse_hex;

pub(crate) fn init(project_path: &Path, frameworks: &[Framework]) -> Result<()> {
    let dir = project_path.join(".bissue");

    create_dir_all(&dir)
        .with_context(|| format!("Cannot create directory {}", dir.display()))?;
    create_dir_all(dir.join("crashes"))?;
    create_dir_all(dir.join("tickets"))?;
    create_dir_all(dir.join("issues"))?;
    create_dir_all(dir.join("issues/.libs"))?;

    let create_file_if_missing = |name: &str, content: &[u8]| -> Result<()> {
        let path = dir.join(name);
        if !path.exists() {
            std::fs::write(&path, content)
                .with_context(|| format!("Cannot write file at {}", path.display()))?;
        }
        Ok(())
    };

    let dev_config = DevConfig {
        remotes: Vec::new(),
    };
    let dev_content = toml::to_string(&dev_config).context("Cannot serialize dev config")?;
    create_file_if_missing("dev_config.toml", dev_content.as_bytes())?;

    let user_config = UserConfig {
        remotes: Vec::new(),
        ok_color: "#42f5b6".to_string(),
        err_color: "#f54266".to_string(),
    };
    let user_content = toml::to_string(&user_config).context("Cannot serialize user config")?;
    create_file_if_missing("user_config.toml", user_content.as_bytes())?;

    let lock = Lock {
        frameworks: frameworks.to_vec(),
        ..Default::default()
    };
    let lock_content = toml::to_string(&lock).context("Cannot serialize lock file")?;
    create_file_if_missing("bissue.lock", lock_content.as_bytes())?;

    let (ok_r, ok_g, ok_b) = parse_hex(&user_config.ok_color);

    println!(
        "Beautiful issues have been {} in {} directory",
        "successfully initialized".bold().truecolor(ok_r, ok_g, ok_b),
        dir.display().italic()
    );

    Ok(())
}