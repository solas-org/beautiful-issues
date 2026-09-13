use crate::render::unwrappers::{read_dir, SafeUnwrap};
use std::path::PathBuf;
use owo_colors::OwoColorize;

pub(crate) fn list(dir: &PathBuf, extension: &str){
    let paths = paths_by_extension(dir, extension);
    let mut cnt: u32 = 1;
    if paths.len() == 0 {
        println!("There is no files with .{extension} extension");
    }
    for path in &paths{
        let display = path.display().to_string();
        let file_url = if display.starts_with('/') {
            format!("file://{}", display)
        } else {
            format!("file:///{}", display)
        };
        println!("{cnt}. \x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", file_url, path.file_name().unwrap().display().italic());
        cnt += 1;
    }
}

pub(crate) fn paths_by_extension(dir: &PathBuf, extension: &str) -> Vec<PathBuf>{
    let mut res: Vec<PathBuf> = Vec::new();

    for file in read_dir(&dir, "Cannot read directory") {
        if let Ok(file) = file{
            let path = &file.path();
            if path.extension().unwrap_or_default() == extension {
                res.push(dunce::canonicalize(file.path()).unwrap_safe("Cannot canonicalize path"));
            }
        };
    }
    res
}

