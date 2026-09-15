use std::fs::read;
use std::iter::FilterMap;
use std::path::PathBuf;
use owo_colors::OwoColorize;
use crate::commands::enums::{PushRemote, PushTarget};
use crate::configs::{DevConfig, UserConfig};
use url::Url;
use walkdir::{DirEntry, IntoIter, WalkDir};
use crate::lock::Lock;
use crate::models::ticket::Ticket;
use crate::render::styling::parse_hex;

pub(crate) async fn push(target: &PushTarget, remote_type: &PushRemote, custom_remotes: &[Url]) -> anyhow::Result<()> {
    let mut user_remotes = UserConfig::global().remotes.clone();
    let mut devs_remotes= Vec::new();

    let get_dir_entries = |dir: &str| -> FilterMap<IntoIter, fn(walkdir::Result<DirEntry>) -> Option<DirEntry>>{
        WalkDir::new(dir).into_iter().filter_map(|e| e.ok())
    };

    for entry in get_dir_entries("./.bissue/issues/lock") {
        if entry.file_type().is_file() && entry.file_name() == "dev_config.toml" {
            devs_remotes.append(&mut DevConfig::load_from_path(entry.path()).remotes);
        }
    }

    let remotes: Vec<Url> = match &remote_type {
        PushRemote::All => {
            devs_remotes.append(&mut user_remotes);
            devs_remotes
                .iter()
                .filter_map(|s| Url::parse(s).ok())
                .collect()
        }
        PushRemote::Dev => {
            devs_remotes
                .iter()
                .filter_map(|s| Url::parse(s).ok())
                .collect()
        }
        PushRemote::User => {
            user_remotes
                .iter()
                .filter_map(|s| Url::parse(s).ok())
                .collect()
        }
        PushRemote::Custom => {
            custom_remotes
                .iter()
                .filter_map(|s| Url::parse(s.as_ref()).ok())
                .collect()
        }
    };


    if remotes.len() == 0 {
        println!("No remotes to push");
        return Ok(())
    }

    let lock = Lock::load();

    let get_all_tickets = |check_pushed: bool| -> anyhow::Result<Vec<Ticket>>{
        let mut res = Vec::new();
        for entry in get_dir_entries("./.bissue/tickets") {
            if entry.file_type().is_file()
                && entry.path().extension().and_then(|e| e.to_str()) == Some("ticket")
                && (!check_pushed || lock.pushed.contains(&PathBuf::from(entry.path()))){
                let read = read(&entry.path())?;
                res.push(serde_json::from_slice(&read)?);
            }
        }
        Ok(res)
    };

    let tickets: Vec<Ticket> = match &target {
        PushTarget::Last => {
            let read = read(&lock.last_ticket)?;
            vec![serde_json::from_slice(&read)?]
        }
        PushTarget::Single { path } => {
            let read = read(&path)?;
            vec![serde_json::from_slice(&read)?]
        }
        PushTarget::Unpushed => get_all_tickets(true)?,
        PushTarget::All => get_all_tickets(false)?,
    };

    if tickets.len() == 0 {
        println!("No tickets to push");
        return Ok(())
    }

    let (ok_r, ok_g, ok_b) = parse_hex(&UserConfig::global().ok_color);
    let (err_r, err_g, err_b) = parse_hex(&UserConfig::global().err_color);
    let mut ok_cnt = 0;
    let mut err_cnt = 0;

    let client = reqwest::Client::new();
    for remote in remotes {
        for ticket in &tickets {
            let res = client.post(remote.clone())
                .json(ticket)
                .send()
                .await;
            match res {
                Ok(_) => {
                    ok_cnt += 1;
                    println!("{} to: {remote}", "Successfully pushed".bold().truecolor(ok_r, ok_g, ok_b));
                }
                Err(e) => {
                    err_cnt += 1;
                    eprintln!("{}: {}", "Failed to push".bold().truecolor(err_r, err_g, err_b), e);
                }
            }

        }
    }

    println!(
        "Pushing complete!\n{ok_cnt} {}\n{err_cnt} {}",
        "Successfully pushed".bold().truecolor(ok_r, ok_g, ok_b),
        "Failed to push".bold().truecolor(err_r, err_g, err_b),
    );

    Ok(())
}