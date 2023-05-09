use crate::utils::convert_int_to_version;
use colored::Colorize;
use git2::{Branch, BranchType, FetchOptions, RemoteCallbacks, Repository};
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::error::Error;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use crate::files::parser::{Config, KjspkgConfig, Package};

fn find_carbon_file(entry: DirEntry, name: &str, is_carbon: bool) -> Option<PathBuf> {
    let path = entry.path();
    let file = if is_carbon {
        "carbon.config.json"
    } else {
        ".kjspkg"
    };
    if path.is_file() && path.file_name().unwrap().to_str().unwrap().eq(file) {
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
    name: &str,
    is_carbon: bool,
    repo_name: &str,
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

    if !branch_name.is_empty() && is_carbon {
        builder.branch(branch_name);
    }

    let carbon_folder = kubejs_dir.join("carbon").join(repo_name);

    let repo = builder.clone(repo_url, temp_dir.path())?;
    if !carbon_folder.exists() {
        builder.clone(repo_url, carbon_folder.as_path())?;
    }

    let mut carbon_files = Vec::new();
    for entry in fs::read_dir(repo.workdir().unwrap())
        .map_err(|e| format!("Failed to read directory: {}", e))?
    {
        if let Some(path) = find_carbon_file(
            entry.map_err(|e| format!("Failed to read entry: {}", e))?,
            name,
            is_carbon,
        ) {
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

pub async fn get_json_from_github(
    repo: &str,
    branch: &str,
    name: &str,
    is_carbon: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent("KubeJsPackageManager (carbonkjs@gmail.com)")
        .build()?;

    let repo_url = format!("https://api.github.com/repos/{}/{}", name, repo);

    let file_name = if is_carbon {
        "carbon.config.json"
    } else {
        ".kjspkg"
    };

    let default_file_url = format!(
        "https://raw.githubusercontent.com/{}/{}/master/{}",
        name, repo, file_name
    );

    let repo_response = client.get(&repo_url).send().await?;

    match repo_response.status() {
        StatusCode::OK => {
            if is_carbon {
                let carbon_config = fetch_carbon_config(default_file_url, client).await?;
                print_carbon_config(carbon_config);
            } else {
                let kjspkg_config = fetch_kjspkg_config(default_file_url, client, name).await?;
                print_kjspkg_config(repo, kjspkg_config);
            }
        }
        StatusCode::NOT_FOUND => {
            println!(
                "[{}] {}",
                "error".red().bold(),
                "This script does not exist. Make sure you typed the name correctly."
            );
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Script not found",
            )));
        }
        _ => {
            println!(
                "[{}] {}",
                "error".red().bold(),
                "Failed to retrieve script info"
            );
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to retrieve script info",
            )));
        }
    }
    Ok(())
}

async fn fetch_carbon_config(
    default_file_url: String,
    client: reqwest::Client,
) -> Result<Config, Box<dyn std::error::Error>> {
    let response = client.get(&default_file_url).send().await?;
    let response_text = response.text().await?;
    let package: Config = serde_json::from_str(&response_text)?;

    Ok(package)
}

async fn fetch_kjspkg_config(
    default_file_url: String,
    client: reqwest::Client,
    name: &str,
) -> Result<KjspkgConfig, Box<dyn std::error::Error>> {
    let response = client.get(&default_file_url).send().await?;
    let response_text = response.text().await?;
    let package: KjspkgConfig = serde_json::from_str(&response_text)?;

    Ok(package)
}

fn print_kjspkg_config(name: &str, package: KjspkgConfig) {
    println!("{}", "Package Info:".bold().underline());
    println!("Name: {}", name.bright_green());
    println!("Author: {}", package.author.bright_green());
    println!("Description: {}", package.description.bright_green());
    let result = convert_int_to_version(&package.versions);
    println!("Minecraft Version:");
    for (i, version) in result.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
        print!("{}", version.bright_green());
    }

    println!();
    println!("Modloaders: ");
    for (i, modloader) in package.modloaders.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
        print!("{}", modloader.bright_green());
    }
    println!();
}

fn print_carbon_config(package: Config) {
    println!("{}", "Package Info:".bold().underline());
    println!("Name: {}", package.name.bright_green());
    println!("Version: {}", package.version.bright_green());
    println!("Author: {}", package.author.bright_green());
    println!("Description: {}", package.description.bright_green());
    println!(
        "Minecraft Version: {}",
        package.minecraftVersion.bright_green()
    );
    println!("Modloaders: ");
    for (i, modloader) in package.modloaders.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
        print!("{}", modloader.bright_green());
    }
    println!();
}
