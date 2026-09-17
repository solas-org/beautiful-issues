use std::fs::read;
use std::iter::FilterMap;
use std::path::{PathBuf};
use owo_colors::OwoColorize;
use crate::commands::enums::{PushRemote, PushTarget};
use crate::configs::{DevConfig, UserConfig};
use url::Url;
use walkdir::{DirEntry, IntoIter, WalkDir};
use crate::lock::Lock;
use crate::models::ticket::{GithubTicket, Ticket};
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

    let mut lock = Lock::load();

    let get_all_tickets = |check_pushed: bool| -> anyhow::Result<(Vec<Ticket>, Vec<PathBuf>)>{
        let mut tickets = Vec::new();
        let mut paths = Vec::new();
        for entry in get_dir_entries("./.bissue/tickets") {
            if entry.file_type().is_file()
                && entry.path().extension().and_then(|e| e.to_str()) == Some("ticket")
                && (!check_pushed || lock.pushed.contains(&PathBuf::from(entry.path()))){
                let read = read(&entry.path())?;
                tickets.push(serde_json::from_slice(&read)?);
                paths.push(entry.path().to_path_buf())
            }
        }
        Ok((tickets, paths))
    };

    let (tickets, tickets_paths): (Vec<Ticket>, Vec<PathBuf>) = match &target {
        PushTarget::Last => {
            let read = read(&lock.last_ticket)?;
            (vec![serde_json::from_slice(&read)?], vec![lock.last_ticket.clone()])
        }
        PushTarget::Single { path } => {
            let read = read(&path)?;
            (vec![serde_json::from_slice(&read)?], vec![path.clone()])
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
    let mut divider = "";
    for remote in remotes {

        divider = "\n";
        let host = remote.host_str().expect("Invalid url: no host.");
        let is_gh = host.contains("github.com");

        let github_token = if is_gh {
            std::env::var("GITHUB_TOKEN").ok()
        } else {
            None
        };

        if is_gh && github_token.is_none() {
            eprintln!(
                "{}: GITHUB_TOKEN environment variable is not set.\n\
                You need to set GitHub classic token in persistent environment variable in order to push to {} remote.\n\
                More info available here: https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens",
                "Error".bold().truecolor(err_r, err_g, err_b), remote
            );
            continue
        }

        println!("{divider}› Pushing to remote: {}...", remote);

        for (tk_cnt, ticket) in tickets.iter().enumerate() {
            println!("{}. {}", tk_cnt+1, &tickets_paths[tk_cnt].display());
            let res = if !is_gh {
                client.post(remote.clone())
                    .json(ticket)
                    .send()
                    .await
                    .map_err(anyhow::Error::from)
            } else {
                client
                    .post(remote.clone())
                    .header("User-Agent", "bissue")
                    .header("Accept", "application/vnd.github+json")
                    .header("X-GitHub-Api-Version", "2026-03-10")
                    .bearer_auth(github_token.as_ref().unwrap())
                    .json(&GithubTicket::create(&ticket)?)
                    .send()
                    .await
                    .map_err(anyhow::Error::from)

            };

            match res {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        lock.pushed.push(tickets_paths[tk_cnt].clone());
                        ok_cnt += 1;
                        println!(
                            "{} to: {remote}",
                            "Successfully pushed ticket".bold().truecolor(ok_r, ok_g, ok_b)
                        );
                    } else {
                        let error_body = response.text().await.unwrap_or_default();
                        err_cnt += 1;
                        eprintln!(
                            "{}: HTTP {} - {}",
                            "Failed to push ticket".bold().truecolor(err_r, err_g, err_b),
                            status,
                            error_body
                        );
                    }
                }
                Err(e) => {
                    err_cnt += 1;
                    eprintln!(
                        "{}: Network error: {}",
                        "Failed to push ticket".bold().truecolor(err_r, err_g, err_b),
                        e
                    );
                }
            }
        }
    }

    lock.write()?;

    println!(
        "{divider}Pushing complete!\n{ok_cnt} {}\n{err_cnt} {}",
        "Tickets successfully pushed".bold().truecolor(ok_r, ok_g, ok_b),
        "Tickets failed to push".bold().truecolor(err_r, err_g, err_b),
    );

    Ok(())
}