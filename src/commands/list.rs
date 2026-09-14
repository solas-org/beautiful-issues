use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};
use url::Url;

pub(crate) fn list(dir: &Path, extension: &str) -> Result<()> {
    let paths = paths_by_extension(dir, extension)?;
    if paths.is_empty() {
        println!("There are no files with .{extension} extension");
        return Ok(());
    }
    for (idx, path) in paths.iter().enumerate() {
        let file_url = match Url::from_file_path(path) {
            Ok(url) => url.to_string(),
            Err(_) => format!("file:///{}", path.display()),
        };
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        println!(
            "{}. \x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
            idx + 1,
            file_url,
            file_name.italic()
        );
    }
    Ok(())
}

pub(crate) fn paths_by_extension(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut res: Vec<PathBuf> = Vec::new();

    if !dir.exists() {
        return Ok(res);
    }

    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("Cannot read directory {}", dir.display()))?;

    for file in entries.flatten() {
        let path = file.path();
        if path.extension().and_then(|e| e.to_str()) == Some(extension) {
            let canonical = dunce::canonicalize(&path).unwrap_or(path);
            res.push(canonical);
        }
    }
    Ok(res)
}

