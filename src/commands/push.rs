use std::fs::read;
use std::path::{Path, PathBuf};
use anyhow::Context;
use owo_colors::OwoColorize;
use crate::commands::enums::{PushRemote, PushTarget};
use crate::configs::{find_bissue_dir, DevConfig, UserConfig};
use url::Url;
use walkdir::{WalkDir};
use crate::lock::Lock;
use crate::models::ticket::{GithubTicket, Ticket};
use crate::render::styling::parse_hex;

pub(crate) async fn push(target: &PushTarget, remote_type: &PushRemote, custom_remotes: &[Url]) -> anyhow::Result<()> {
    let mut user_remotes = UserConfig::global().remotes.clone();
    let mut devs_remotes= Vec::new();

    let bissue_dir = find_bissue_dir();
    let dev_configs_dir = bissue_dir.join("issues/.libs");
    let tickets_dir = bissue_dir.join("tickets");

    for entry in WalkDir::new(&dev_configs_dir).into_iter().flatten() {
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
            Vec::from(custom_remotes)
        }
    };


    if remotes.is_empty() {
        println!("No remotes to push");
        return Ok(())
    }

    let mut lock = Lock::load();

    let load_ticket_file = |path: &Path| -> anyhow::Result<(Ticket, PathBuf)> {
        let canon = dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let bytes = read(&canon).with_context(|| format!("Cannot read ticket at {}", canon.display()))?;
        let ticket: Ticket = serde_json::from_slice(&bytes)?;
        Ok((ticket, canon))
    };

    let tickets_with_paths: Vec<(Ticket, PathBuf)> = match target {
        PushTarget::Last => {
            if lock.last_ticket.as_os_str().is_empty() || !lock.last_ticket.exists() {
                println!("Last ticket does not exist");
                return Ok(());
            }
            vec![load_ticket_file(&lock.last_ticket)?]
        }
        PushTarget::Single { path } => {
            vec![load_ticket_file(path)?]
        }
        PushTarget::All | PushTarget::Unpushed => {
            let only_unpushed = matches!(target, PushTarget::Unpushed);
            let mut collected = Vec::new();

            if tickets_dir.exists() {
                for entry in WalkDir::new(&tickets_dir).into_iter().flatten() {
                    let path = entry.path();
                    if entry.file_type().is_file() && path.extension().and_then(|e| e.to_str()) == Some("ticket") {
                        let canon = dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
                        if (!only_unpushed || !lock.pushed.contains(&canon))
                            && let Ok(item) = load_ticket_file(&canon)
                        {
                            collected.push(item);
                        }
                    }
                }
            }
            collected
        }
    };

    if tickets_with_paths.is_empty() {
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
        let host = match remote.host_str() {
            Some(h) => h,
            None => {
                eprintln!("Skipping invalid remote (no host): {}", remote);
                continue;
            }
        };

        let is_gh = host.contains("api.github.com");

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
        divider = "\n";

        for (idx, (ticket, path)) in tickets_with_paths.iter().enumerate() {
            println!("{}. {}", idx + 1, path.display());
            let req = if !is_gh {
                client.post(remote.clone()).json(ticket)
            } else {
                client
                    .post(remote.clone())
                    .header("User-Agent", "bissue")
                    .header("Accept", "application/vnd.github+json")
                    .header("X-GitHub-Api-Version", "2026-03-10")
                    .bearer_auth(github_token.as_ref().unwrap())
                    .json(&GithubTicket::create(ticket)?)
            };

            match req.send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        if !lock.pushed.contains(path) {
                            lock.pushed.push(path.clone());
                        }
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