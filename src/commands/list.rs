use crate::render::unwrappers::read_dir;
use std::path::PathBuf;
use owo_colors::OwoColorize;

pub(crate) fn list(dir: &PathBuf, extension: &str){
    let paths = paths_by_extension(dir, extension);
    let mut cnt: u32 = 1;
    if paths.len() == 0 {
        println!("There is no files with .{extension} extension");
    }
    for path in &paths{
        println!("{cnt}. {}", path.file_name().unwrap().display().italic());
        cnt += 1;
    }
}

pub(crate) fn paths_by_extension(dir: &PathBuf, extension: &str) -> Vec<PathBuf>{
    let mut res: Vec<PathBuf> = Vec::new();

    for file in read_dir(&dir, "Cannot read directory") {
        if let Ok(file) = file{
            let path = &file.path();
            if path.extension().unwrap_or_default() == extension {
                res.push(file.path());
            }
        };
    }
    res
}

