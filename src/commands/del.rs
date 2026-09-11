use crate::commands::enums::DelTarget;
use crate::commands::list::paths_by_extension;
use std::fs::remove_file;
use std::path::PathBuf;

pub(crate) fn del(del_target: &DelTarget, dir: &PathBuf, extension: &str, last_file_path: &PathBuf){
    match &del_target {
        DelTarget::Last => {
            remove_file(&last_file_path).unwrap_or_else(|_| panic!("Cannot delete file: {:?}", &last_file_path));
            println!("File deleted at path: {}", last_file_path.display())
        }
        DelTarget::Single { path } => {
            remove_file(&path).unwrap_or_else(|_| panic!("Cannot delete file: {:?}", path));
            println!("File deleted at path: {}", path.display())
        }
        DelTarget::All => {
            println!("Files deleted at paths:");
            let mut cnt: u32 = 1;
            for path in paths_by_extension(dir, extension){
                remove_file(&path).unwrap_or_else(|_| panic!("Cannot delete file: {:?}", path));
                println!("{cnt}. {}", path.display());
                cnt += 1;
            }
        }
    }
}