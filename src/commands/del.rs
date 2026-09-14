use crate::commands::enums::DelTarget;
use crate::commands::list::paths_by_extension;
use std::path::Path;
use owo_colors::OwoColorize;
use crate::configs::UserConfig;
use crate::render::styling::parse_hex;
use anyhow::{bail, Context, Result};

pub(crate) fn del(del_target: &DelTarget, dir: &Path, extension: &str, last_file_path: &Path) -> Result<()> {
    let user_config = UserConfig::global();
    let (err_r, err_g, err_b) = parse_hex(&user_config.err_color);
    let (ok_r, ok_g, ok_b) = parse_hex(&user_config.ok_color);

    match del_target {
        DelTarget::Last => {
            if last_file_path.as_os_str().is_empty() {
                eprintln!(
                    "{}\nWarning: if you deleted previous last created file, bissue can't delete file before last",
                    "Last file doesn't exist!".bold().truecolor(err_r, err_g, err_b)
                );
                bail!("Last file doesn't exist");
            }

            std::fs::remove_file(last_file_path)
                .with_context(|| format!("Cannot delete file at {}", last_file_path.display()))?;
            println!("File deleted at path: {}", last_file_path.display().italic());
        }
        DelTarget::Single { path } => {
            std::fs::remove_file(path)
                .with_context(|| format!("Cannot delete file at {}", path.display()))?;
            println!(
                "File {} deleted: {}",
                "successfully".truecolor(ok_r, ok_g, ok_b).bold(),
                path.display().italic()
            );
        }
        DelTarget::All => {
            let paths = paths_by_extension(dir, extension)?;
            if paths.is_empty() {
                println!("Nothing to delete");
                return Ok(());
            }

            for (idx, path) in paths.iter().enumerate() {
                std::fs::remove_file(path)
                    .with_context(|| format!("Cannot delete file at {}", path.display()))?;
                println!("{}. {}", idx + 1, path.display().italic());
            }

            println!(
                "{} {}",
                "Successfully deleted:".truecolor(ok_r, ok_g, ok_b).bold(),
                paths.len().bold()
            );
        }
    }
    Ok(())
}