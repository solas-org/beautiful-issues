use std::fs::{File, ReadDir};
use crate::configs::UserConfig;
use crate::render::styling::parse_hex;
use owo_colors::OwoColorize;
use std::path::PathBuf;

pub(crate) fn read(path: &PathBuf, panic_message: &str) -> Vec<u8>{
    let user_config = UserConfig::global();
    let (err_r, err_g, err_b) = parse_hex(&*user_config.err_color);

    std::fs::read(&path).unwrap_or_else(|e| panic!(
        "{}: {}\nBecause of: {e}",
        panic_message.bold().truecolor(err_r, err_g, err_b),
        &path.display().italic()))
}

pub(crate) fn exists(path: &PathBuf, panic_message: &str) -> bool{
    let user_config = UserConfig::global();
    let (err_r, err_g, err_b) = parse_hex(&*user_config.err_color);

    std::fs::exists(&path).unwrap_or_else(|e| panic!(
        "{}: {}\nBecause of: {e}",
        panic_message.bold().truecolor(err_r, err_g, err_b),
        &path.display().italic()))
}

pub(crate) fn create_dir(path: &PathBuf, panic_message: &str){
    let user_config = UserConfig::global();
    let (err_r, err_g, err_b) = parse_hex(&*user_config.err_color);

    std::fs::create_dir(&path).unwrap_or_else(|e| panic!(
        "{}: {}\nBecause of: {e}",
        panic_message.bold().truecolor(err_r, err_g, err_b),
        &path.display().italic()))
}

pub(crate) fn read_dir(path: &PathBuf, panic_message: &str) -> ReadDir{
    let user_config = UserConfig::global();
    let (err_r, err_g, err_b) = parse_hex(&*user_config.err_color);

    std::fs::read_dir(&path).unwrap_or_else(|e| panic!(
        "{}: {}\nBecause of: {e}",
        panic_message.bold().truecolor(err_r, err_g, err_b),
        &path.display().italic()))
}

pub(crate) fn create_file(path: &PathBuf, panic_message: &str) -> File{
    let user_config = UserConfig::global();
    let (err_r, err_g, err_b) = parse_hex(&*user_config.err_color);

    File::create(&path).unwrap_or_else(|e| panic!(
        "{}: {}\nBecause of: {e}",
        panic_message.bold().truecolor(err_r, err_g, err_b),
        &path.display().italic()))
}

pub(crate) fn remove_file(path: &PathBuf, panic_message: &str){
    let user_config = UserConfig::global();
    let (err_r, err_g, err_b) = parse_hex(&*user_config.err_color);

    std::fs::remove_file(&path).unwrap_or_else(|e| panic!(
        "{}: {}\nBecause of: {e}",
        panic_message.bold().truecolor(err_r, err_g, err_b),
        &path.display().italic()))
}

pub trait SafeUnwrap<T> {
    fn unwrap_safe(self, panic_message: &str) -> T;
}

impl<T, E: std::fmt::Display> SafeUnwrap<T> for Result<T, E> {
    fn unwrap_safe(self, panic_message: &str) -> T {
        let user_config = UserConfig::global();
        let (err_r, err_g, err_b) = parse_hex(&user_config.err_color);

        self.unwrap_or_else(|e| {
            panic!(
                "{}\nBecause of: {e}",
                panic_message.bold().truecolor(err_r, err_g, err_b)
            )
        })
    }
}