#![allow(unused)]
use std::path::PathBuf;
use std::{error::Error, ffi::OsString};

use clap::{arg, Arg, Command};

use colored::Colorize;
use github::get_json_from_github;
use octocrab::Octocrab;
use serde::Deserialize;
use serde_json::from_str;

use crate::{files::check_if_folder_exists, github::get_files_from_repo};

mod dependencies;
mod files;
mod github;

#[derive(Deserialize)]
struct Config {
    description: String,
    author: String,
}

fn cli() -> Command {
    Command::new("carbon")
        .about("A KubeJS script manager")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(Command::new("list").about("List of scripts"))
        .subcommand(Command::new("add").about("Add a script").args([
            Arg::new("script_name").help("name of the script. run 'list' for available scripts."),
        ]))
        .subcommand(Command::new("info").about("Info about script").args([
            Arg::new("script_name").help("name of the script. run 'list' for available scripts."),
        ]))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli().get_matches();
    let current_dir = std::env::current_dir()?;
    let kubejs_dir = current_dir.join("kubejs");
    let scripts_dirs = [
        "server_scripts",
        "startup_scripts",
        "client_scripts",
        "assets",
    ];
    let x = current_dir.join("kubejs");

    match matches.subcommand() {
        Some(("list", sub_matches)) => {
            println!(
                "[{}] {}",
                "info".blue().bold(),
                format!(
                    "You can find all the packages in this github organization: {}",
                    "https://github.com/carbon-kjs".bold()
                )
            );
            Ok(())
        }

        Some(("add", sub_matches)) => {
            let script_name: &str = sub_matches
                .get_one::<String>("script_name")
                .expect("Script name required.");

            let parts: Vec<&str> = script_name.split('@').collect();

            let version: Option<&str> = parts.get(1).cloned();

            if let Some(ver) = version {
                let valid_version = ver
                    .split('.')
                    .filter(|s| s.chars().all(char::is_numeric))
                    .collect::<Vec<&str>>();
                match valid_version.len() {
                    2 | 3 => {}
                    _ => {
                        println!(
                            "[{}] {}",
                            "error".red().bold(),
                            format!("Version format is not correct. It should look someting like this: 1.0.0 or 3.12")
                        );
                        panic!();
                    }
                }
            }

            println!(
                "[{}] {}",
                "info".blue().bold(),
                format!("Cloning {script_name}.",)
            );

            let branch: &str = match &version {
                Some(ver) => ver,
                None => "",
            };

            let repo_url = format!("https://github.com/carbon-kjs/{}", parts[0]);

            match check_if_folder_exists(&kubejs_dir) {
                Ok(true) => match github::get_files_from_repo(&repo_url, branch, &kubejs_dir) {
                    Ok((
                        temp_dir,
                        carbon_files,
                        startup_scripts,
                        client_scripts,
                        server_scripts,
                        assets,
                    )) => {
                        let files = (
                            temp_dir,
                            carbon_files,
                            startup_scripts,
                            client_scripts,
                            server_scripts,
                            assets,
                        );

                        files::copy_files_to_dir_and_remove_temp_dir(
                            files,
                            &scripts_dirs,
                            &current_dir.join("kubejs"),
                        )
                        .expect("Failed to copy files to dir and remove temp dir.");
                    }
                    Err(e) => {
                        println!(
                                "[{}] {}",
                                "error".red().bold(),
                                format!("This is script does not exist. Make sure you typed the name correctly. {}", e)
                            );

                        return panic!();
                    }
                },
                Ok(false) => {
                    println!("[{}] {}", "error".red().bold(), format!("KubeJS folder does not exist. Install KubeJS and run minecraft. After that, run this command again."));
                }
                Err(e) => {
                    // error message is already printed by check_if_folder_exists()
                }
            }
            Ok(())
        }

        Some(("info", sub_matches)) => {
            let script_name: &str = sub_matches
                .get_one::<String>("script_name")
                .expect("Script name required.");

            let parts: Vec<&str> = script_name.split('@').collect();

            let version: Option<&str> = parts.get(1).cloned();

            if let Some(ver) = version {
                let valid_version = ver
                    .split('.')
                    .filter(|s| s.chars().all(char::is_numeric))
                    .collect::<Vec<&str>>();
                match valid_version.len() {
                    2 | 3 => {}
                    _ => {
                        println!(
                                "[{}] {}",
                                "error".red().bold(),
                                format!("Version format is not correct. It should look someting like this: 1.0.0 or 3.12")
                            );
                        panic!();
                    }
                }
            }

            let branch: &str = match &version {
                Some(ver) => ver,
                None => "master",
            };

            let x = get_json_from_github(parts[0], branch).await?;

            Ok(())
        }

        _ => unreachable!(),
    }
}
