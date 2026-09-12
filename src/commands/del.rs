use crate::commands::enums::DelTarget;
use crate::commands::list::paths_by_extension;
use std::path::PathBuf;
use owo_colors::OwoColorize;
use crate::configs::UserConfig;
use crate::render::styling::parse_hex;
use crate::render::unwrappers::remove_file;

pub(crate) fn del(del_target: &DelTarget, dir: &PathBuf, extension: &str, last_file_path: &PathBuf){
    let user_config = UserConfig::global();
    let (err_r, err_g, err_b) = parse_hex(&*user_config.err_color);
    let (ok_r, ok_g, ok_b) = parse_hex(&*user_config.ok_color);

    match &del_target {
        DelTarget::Last => {
            if last_file_path == "" {
                panic!("{}\nWarning: if you deleted previous last created file, bissue can't delete file before last",
                       "Last file doesn't exist!".bold().truecolor(err_r, err_g, err_b));
            }

            remove_file(&last_file_path, "Cannot delete file");
            println!("File deleted at path: {}", last_file_path.display().italic());
        }
        DelTarget::Single { path } => {
            remove_file(&path, "Cannot delete file");
            println!("File {} deleted: {}","successfully".truecolor(ok_r,ok_g,ok_b).bold(), path.display().italic());
        }
        DelTarget::All => {
            let mut cnt: u32 = 1;
            for path in paths_by_extension(dir, extension){
                remove_file(&path, "Cannot delete file");
                println!("{cnt}. {}", path.display().italic());
                cnt += 1;
            }

            cnt -= 1;
            if cnt == 0 {
                println!("Nothing to delete");
                return
            }
            println!("{} {}", "Successfully deleted:".truecolor(ok_r,ok_g,ok_b).bold(), cnt.bold())
        }
    }
}