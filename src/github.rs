use colored::Colorize;
use git2::{Branch, BranchType, FetchOptions, RemoteCallbacks, Repository};
use serde_json::Value;
use std::error::Error;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use crate::files::parser::{Config, Package};

fn find_carbon_file(entry: DirEntry) -> Option<PathBuf> {
    let path = entry.path();
    if path.is_file()
        && path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .eq("carbon.config.json")
    {
        Some(path.to_owned())
    } else {
        None
    }
}

fn find_scripts_folder(entry: DirEntry) -> Option<PathBuf> {
    let path = entry.path();
    if path.is_dir() {
        match path.file_name() {
            Some(name)
                if name == "startup_scripts"
                    || name == "client_scripts"
                    || name == "server_scripts"
                    || name == "assets" =>
            {
                Some(path.to_owned())
            }
            _ => None,
        }
    } else {
        None
    }
}

pub fn get_files_from_repo(
    repo_url: &str,
    branch_name: &str,
    kubejs_dir: &Path,
) -> Result<
    (
        PathBuf,
        Vec<PathBuf>,
        Vec<PathBuf>,
        Vec<PathBuf>,
        Vec<PathBuf>,
        Vec<PathBuf>,
    ),
    Box<dyn std::error::Error>,
> {
    let temp_dir = tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let mut builder = git2::build::RepoBuilder::new();

    if !branch_name.is_empty() {
        builder.branch(branch_name);
    }

    let repo = builder.clone(repo_url, temp_dir.path())?;

    let mut carbon_files = Vec::new();
    for entry in fs::read_dir(repo.workdir().unwrap())
        .map_err(|e| format!("Failed to read directory: {}", e))?
    {
        if let Some(path) =
            find_carbon_file(entry.map_err(|e| format!("Failed to read entry: {}", e))?)
        {
            carbon_files.push(path);
        }
    }

    let mut startup_scripts = Vec::new();
    let mut client_scripts = Vec::new();
    let mut server_scripts = Vec::new();
    let mut assets = Vec::new();
    for entry in fs::read_dir(repo.workdir().unwrap())
        .map_err(|e| format!("Failed to read directory: {}", e))?
    {
        if let Some(path) =
            find_scripts_folder(entry.map_err(|e| format!("Failed to read entry: {}", e))?)
        {
            match path.file_name().unwrap().to_str().unwrap() {
                "startup_scripts" => startup_scripts.push(path),
                "client_scripts" => client_scripts.push(path),
                "server_scripts" => server_scripts.push(path),
                "assets" => assets.push(path),
                _ => {}
            }
        }
    }

    Ok((
        temp_dir.into_path(),
        carbon_files,
        startup_scripts,
        client_scripts,
        server_scripts,
        assets,
    ))
}

pub async fn get_json_from_github(repo: &str, branch: &str) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::builder()
        .user_agent("KubeJsPackageManager (carbonkjs@gmail.com)")
        .build()?;

    let repo_url = format!("https://api.github.com/repos/carbon-kjs/{}", repo);

    let default_file_url = format!(
        "https://raw.githubusercontent.com/carbon-kjs/{}/master/carbon.config.json",
        repo
    );

    let repo_response = client.get(&repo_url).send().await?;
    if repo_response.status().is_success() {
        let response = client.get(&default_file_url).send().await?.text().await?;
        let package: Config = serde_json::from_str(&response)?;

        println!("{}", "Package Info:".bold().underline());
        println!("Name: {}", package.name.bright_green());
        println!("Version: {}", package.version.bright_green());
        println!("Author: {}", package.author.bright_green());
        println!("Description: {}", package.description.bright_green());
        println!(
            "Minecraft Version: {}",
            package.minecraftVersion.bright_green()
        );
        print!("Modloaders: ");
        for (i, modloader) in package.modloaders.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}", modloader.bright_green());
        }
        println!();
    } else {
        println!(
            "[{}] {}",
            "error".red().bold(),
            format!("This is script does not exist. Make sure you typed the name correctly.",)
        );

        return panic!();
    }
    Ok(())
}
