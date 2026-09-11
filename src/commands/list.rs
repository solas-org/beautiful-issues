use std::path::{PathBuf};

pub(crate) fn list(dir: &PathBuf, extension: &str){
    let paths = paths_by_extension(dir, extension);
    let mut cnt: u32 = 1;
    for path in &paths{
        println!("{cnt}. {}", path.file_name().unwrap().display());
        cnt += 1;
    }
}

pub(crate) fn paths_by_extension(dir: &PathBuf, extension: &str) -> Vec<PathBuf>{
    let mut res: Vec<PathBuf> = Vec::new();
    for file in dir.read_dir().expect("Cannot read folder") {
        if let Ok(file) = file{
            let path = &file.path();
            if path.extension().unwrap_or_default() == extension {
                res.push(file.path());
            }
        };
    }
    res
}

