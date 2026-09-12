use std::fs::{create_dir, exists, File};
use std::io::Write;
use std::path::PathBuf;
use owo_colors::OwoColorize;
use crate::commands::enums::Framework;
use crate::configs::{DevConfig, UserConfig};
use crate::lock::Lock;
use crate::render::styling::parse_hex;

pub(crate) fn init(project_path: &PathBuf, frameworks: &Vec<Framework>) {
    let dir = project_path.join(".bissue");

    let create_folder = |name: &str|{
        let folder_path = dir.join(name);
        if !exists(&folder_path).expect("Cannot check if directory exists") {
            create_dir(&folder_path).expect("Cannot create directory");
        }
    };

    create_folder("");
    create_folder("crashes");
    create_folder("tickets");
    create_folder("issues");
    create_folder("issues/.libs");

    let create_file = |name: &str, content: &[u8]| {
        let path = dir.join(&name);

        if !exists(&path).unwrap_or_else(|_| panic!("Cannot check if {} exists", path.display())) {
            let mut file = File::create(&path)
                .unwrap_or_else(|_| panic!("Cannot create file by path {}", path.display()));
            file.write_all(&content).expect("Cannot write file content");
        }
    };

    {
        let dev_config = DevConfig {
            remotes: Vec::new()
        };

        let file_content = toml::to_string(&dev_config).expect("Cannot serialize dev config");
        create_file("dev_config.toml", file_content.as_bytes());
    }


    let user_config = UserConfig {
        ok_color: "#42f5b6".to_string(),
        err_color: "#f54266".to_string()
    };

    {
        let file_content = toml::to_string(&user_config).expect("Cannot serialize user config");
        create_file("user_config.toml", file_content.as_bytes());
    }

    {
        let mut lock = Lock::default();
        lock.frameworks = frameworks.clone();

        let file_content = toml::to_string(&lock).expect("Cannot serialize lock file");
        create_file("bissue.lock", file_content.as_bytes());
    }

    let (ok_r, ok_g, ok_b) = parse_hex(&*user_config.ok_color);

    println!("Beautiful issues have been {} in {} directory",
             "successfully initialized".bold().truecolor(ok_r,ok_g,ok_b),
             dir.display().italic());
}