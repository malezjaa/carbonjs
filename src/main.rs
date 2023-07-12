#![allow(unused)]
use std::path::PathBuf;
use std::time::Duration;
use std::{error::Error, ffi::OsString};

use clap::{arg, Arg, Command};

use colored::Colorize;

use serde::Deserialize;
use serde_json::from_str;
extern crate simplelog;

use simplelog::*;

mod api;
mod config;
mod files;
mod manager;

#[derive(Debug, Deserialize)]
pub struct PackageInfo {
    repository: String,
    version: String,
    description: String,
    author: String,
    name: String,
}

fn cli() -> Command {
    Command::new("carbon")
        .about("A KubeJS script manager")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(Command::new("list").about("List of scripts"))
        .subcommand(Command::new("add").about("Add a script").args([
            Arg::new("script_name").help("name of the script. run 'list' for available scripts. To add script from kjspkg, add `kjspkg:` before the name of the script."),
        ]))
        .subcommand(Command::new("info").about("Info about script").args([
            Arg::new("script_name").help("name of the script. run 'list' for available scripts. To get info from kjspkg package, add `kjspkg:` before the name of the script."),
        ]))
        .subcommand(Command::new("remove").about("Remove script").args([
            Arg::new("script_name").help("name of the script. run 'list' for available scripts. To remove kjspkg script, add `kjspkg:` before the name of the script."),
        ]))
        .subcommand(Command::new("publish").about("Publish script").args([
            Arg::new("script_name").help("name of the script."),
            Arg::new("github_profile_link").help("link to your github profile (eg. https://github.com/malezjaa)")
        ]))
    }

#[tokio::main]
async fn main() {
    std::env::set_var("API_URL", "https://carbon.beanstech.tech/api");

    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("list", sub_matches)) => {
            info!(
                "Full list of packages can be found here: {}",
                "https://carbon.beanstech.tech"
            )
        }

        Some(("remove", sub_matches)) => {
            let script_name: &str = sub_matches
                .get_one::<String>("script_name")
                .expect("Script name required.");

            let res = api::get_package(script_name).await;
            if res.is_err() {
                error!("Package not found!");
                return;
            }

            let package_info: PackageInfo = serde_json::from_value(res.unwrap()).unwrap();
            match manager::remove_package(package_info, script_name) {
                Ok(_) => {}
                Err(e) => error!("Error removing package: {}", e),
            }
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
                        error!("Version format is not correct. It should look someting like this: 1.0.0 or 3.12");
                        return;
                    }
                }
            }

            let res = api::get_package(parts.get(0).unwrap()).await;
            if res.is_err() {
                error!("Package not found!");
                return;
            }

            let package_info: PackageInfo = serde_json::from_value(res.unwrap()).unwrap();

            match manager::add_package(
                package_info,
                parts.get(0).unwrap_or(&""),
                version.unwrap_or(""),
            )
            .await
            {
                Ok(_) => {}
                Err(e) => error!("Error adding package: {}", e),
            }
        }

        Some(("info", sub_matches)) => {
            let script_name: &str = sub_matches
                .get_one::<String>("script_name")
                .expect("Script name required.");

            let package = api::get_package(script_name).await;
            if package.is_err() {
                error!("Package not found!");
                return;
            }

            let package_info: PackageInfo = serde_json::from_value(package.unwrap()).unwrap();

            println!("{}: {}", "Name:".bold().blue(), package_info.name);
            println!("{}: {}", "Version:".bold().blue(), package_info.version);
            println!(
                "{}: {}",
                "Description:".bold().blue(),
                package_info.description
            );
            println!(
                "{}: {}",
                "Repository".bold().blue(),
                package_info.repository
            );
        }

        _ => unreachable!(),
    }
}
