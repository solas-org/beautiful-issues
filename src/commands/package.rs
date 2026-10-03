use crate::commands::enums::Framework;
use crate::configs::{UserConfig, find_bissue_dir};
use crate::lock::Lock;
use crate::models::package::Package;
use crate::render::styling::parse_hex;
use owo_colors::OwoColorize;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use tar::{Archive, Builder};
use version_compare;
use version_compare::{Cmp, compare};
use zstd::Encoder;

pub(crate) fn generate(output: Option<&PathBuf>) -> anyhow::Result<()> {
    let bissue_dir = find_bissue_dir();
    //TODO add option to include only selected framework issues
    let path = output.unwrap_or(&bissue_dir).join("bissue.tar.zstd");

    let file = fs::File::create(&path)?;
    let enc = Encoder::new(file, 3)?.auto_finish();
    let mut tar = Builder::new(enc);

    tar.append_dir_all("issues", bissue_dir.join("issues"))?;
    tar.append_path_with_name(bissue_dir.join("dev_config.toml"), "dev_config.toml")?;
    tar.finish()?;

    let (ok_r, ok_g, ok_b) = parse_hex(&UserConfig::global().ok_color);
    println!(
        "Archive have {} generated at path {}",
        "successfully".bold().truecolor(ok_r, ok_g, ok_b),
        path.display().italic()
    );

    Ok(())
}

pub(crate) fn add(
    framework: &Framework,
    name: &String,
    version: &String,
    path: &PathBuf,
) -> anyhow::Result<()> {
    let mut lock = Lock::load();
    let found = get_pkg_pos(&lock, framework, name);
    if let Some(p) = found {
        if compare(&lock.packages[p].version, version) != Ok(Cmp::Lt) {
            println!("Package with same version have already installed");
            return Ok(());
        }
    }

    let pkg_dir = find_bissue_dir().join("issues").join(".pkgs");
    let target_dir = pkg_dir.join(format!("{}@{}@{}", &framework, &name, &version));
    let internal_pkg_dir = target_dir.join("issues").join(".pkgs");

    fs::create_dir_all(&pkg_dir)?;

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)?;
    }

    let file = fs::File::open(&path)?;
    let dec = zstd::Decoder::new(file)?;
    let mut archive = Archive::new(dec);

    archive.unpack(&target_dir)?;

    let global_dependencies = &mut lock.dependencies;
    let mut dependencies = Vec::new();
    //TODO add package already in deps
    if internal_pkg_dir.exists() {
        for entry in fs::read_dir(&internal_pkg_dir)? {
            let entry = entry?;
            let entry_path = entry.path();

            if !entry_path.is_dir() {
                continue;
            }

            let file_name = entry.file_name().to_string_lossy().to_string();
            let dest_path = pkg_dir.join(&file_name);

            let parts: Vec<&str> = file_name.split("@").collect();

            let base_name = parts[1].to_string();
            let dep_version = parts[2];

            dependencies.push(base_name.clone());

            if !global_dependencies.contains_key(framework) {
                global_dependencies.insert(framework.clone(), Vec::new());
            }

            if !global_dependencies[framework].contains(&base_name) {
                let names = global_dependencies.get_mut(framework).unwrap();
                names.push(format!("{base_name}@{dep_version}"));
            } else if compare(
                dep_version,
                global_dependencies[framework]
                    .iter()
                    .find(|&x| x == &base_name)
                    .unwrap_or(&"0".to_string()),
            ) != Ok(Cmp::Lt)
            {
                continue;
            }

            if dest_path.exists() {
                fs::remove_dir_all(&dest_path)?;
            }

            fs::rename(&entry_path, &dest_path)?;
        }

        fs::remove_dir_all(&internal_pkg_dir)?;
    }

    let found = get_pkg_pos(&lock, framework, name);

    match &found {
        None => {
            lock.packages.push(Package {
                name: name.clone(),
                framework: framework.clone(),
                version: version.clone(),
                dependencies,
            });
        }
        Some(p) => {
            lock.packages[*p] = Package {
                name: name.clone(),
                framework: framework.clone(),
                version: version.clone(),
                dependencies,
            };
        }
    }

    lock.write()?;

    let (ok_r, ok_g, ok_b) = parse_hex(&UserConfig::global().ok_color);
    println!(
        "Package {} have {} unpacked from {}",
        name.bold(),
        "successfully".bold().truecolor(ok_r, ok_g, ok_b),
        path.display().italic()
    );

    Ok(())
}

pub(crate) fn del(framework: &Framework, name: &String) -> anyhow::Result<()> {
    let mut lock = Lock::load();
    let found = get_pkg_pos(&lock, framework, name);
    let version;
    let mut unique_deps: Vec<String>;

    let (err_r, err_g, err_b) = parse_hex(&UserConfig::global().err_color);

    match &found {
        None => {
            println!(
                "There is {} package in {} framework with name {}",
                "no".bold().truecolor(err_r, err_g, err_b),
                framework.bold(),
                name.bold(),
            );
            return Ok(());
        }
        Some(p) => {
            version = lock.packages[*p].version.clone();

            let mut current_deps = Vec::new();
            let mut other_deps = Vec::new();
            for (fr, deps) in &lock.dependencies {
                if fr == framework {
                    current_deps = deps.clone();
                } else {
                    other_deps.extend(deps.clone());
                }
            }

            let other_set: HashSet<_> = other_deps.into_iter().collect();

            unique_deps = current_deps
                .into_iter()
                .filter(|item| !other_set.contains(item))
                .collect();

            unique_deps.dedup();

            let empty = &mut Vec::new();
            let framework_deps = lock.dependencies.get_mut(framework).unwrap_or(empty);

            for dep in &unique_deps {
                framework_deps.retain(|x| x != dep);
            }

            unique_deps = unique_deps
                .iter()
                .map(|x| framework.to_string() + "@" + x)
                .collect();

            lock.packages.remove(*p);
            lock.write()?;
        }
    }

    let pkgs_dir = find_bissue_dir().join("issues").join(".pkgs");

    let del_pkg_dir = pkgs_dir.join(format!("{}@{}@{}", &framework, &name, &version));

    fs::remove_dir_all(del_pkg_dir)?;

    for dir in unique_deps {
        fs::remove_dir_all(pkgs_dir.join(dir))?;
    }

    println!(
        "Package {} have been {} successfully",
        name.bold(),
        "deleted".bold().truecolor(err_r, err_g, err_b),
    );

    Ok(())
}

pub(crate) fn check(framework: &Framework, names: &Vec<String>) -> anyhow::Result<()> {
    let lock = Lock::load();

    for pkg in &lock.packages {
        if pkg.framework == *framework && !names.contains(&pkg.name) {
            if UserConfig::global().auto_prune {
                del(framework, &pkg.name)?;
            }
            continue;
        }

        let found = get_pkg_pos(&lock, &pkg.framework, &pkg.name);

        match found {
            None => {
                println!("NONE")
            }
            Some(p) => {
                println!("{}", lock.packages[p].version);
            }
        }
    }
    Ok(())
}

fn get_pkg_pos(lock: &Lock, framework: &Framework, name: &String) -> Option<usize> {
    lock.packages
        .iter()
        .position(|x| x.name == *name && x.framework == *framework)
}
