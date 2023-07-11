use crate::config::{get_config, get_instance_config};
use crate::files::{check_if_path_exists, copy_dir_contents, remove_files_in_dir};
use crate::PackageInfo;
extern crate simplelog;

use serde_json::{json, Map};
use simplelog::*;
use std::convert::Infallible;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;

const DIRS: [&str; 4] = [
    "server_scripts",
    "startup_scripts",
    "client_scripts",
    "assets",
];

use reqwest;

pub async fn add_package(
    pkg: PackageInfo,
    name: &str,
    version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let mut builder = git2::build::RepoBuilder::new();
    if !version.is_empty() {
        builder.branch(version);
    }

    let repo;
    let response = reqwest::get(pkg.repository.as_str()).await?;
    if response.status().is_success() {
        repo = builder
            .clone(pkg.repository.as_str(), temp_dir.path())
            .map_err(|e| format!("Failed to clone repo: {}", e))?;
    } else {
        error!("Repository does not exist");
        return Ok(());
    }

    let repo_path = repo.path().to_owned();

    let head = repo.head()?;
    let branch_name = head.shorthand().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "Failed to get branch name")
    })?;

    let mut config = get_config(temp_dir.path())?;

    let instance_config = get_instance_config()?;

    if !config.minecraftVersion.is_empty() {
        if !config.minecraftVersion.contains(&instance_config.version) {
            error!("Minecraft version mismatch");
            return Ok(());
        }
    }

    if !config.modloaders.is_empty() {
        if !config.modloaders.contains(&instance_config.modloader) {
            error!("Modloader mismatch");
            return Ok(());
        }
    }

    let mut pkg_dir = temp_dir.path().to_owned();
    let current_dir = std::env::current_dir()?;

    for dir in DIRS {
        let dir = PathBuf::from(dir);
        let source_path = temp_dir.path().join(&dir);
        let target_path = current_dir.join("kubejs").join(&dir);

        if check_if_path_exists(&source_path)? {
            copy_dir_contents(&source_path, &target_path)?;
        }
    }

    let carbon_folder = current_dir.join("kubejs").join("carbon").join(&config.name);
    if !check_if_path_exists(&carbon_folder)? {
        std::fs::create_dir_all(&carbon_folder)?;
    }

    let source_path = temp_dir.path();
    let target_path = carbon_folder;
    copy_dir_contents(&source_path, &target_path)?;

    info!(
        "{}",
        format!(
            "Successfully installed {:?} with the version of ({:?}).",
            config.name, config.version
        )
    );

    let config_path = current_dir.join("kubejs/carbon.json");

    let file = File::open(&config_path)?;
    let reader = std::io::BufReader::new(file);
    let mut instance_config: serde_json::Value = serde_json::from_reader(reader)?;

    if !instance_config["dependencies"].is_object() {
        instance_config["dependencies"] = json!(Map::new());
    }

    let dependencies = instance_config["dependencies"].as_object_mut().unwrap();

    dependencies.insert(config.name.clone(), json!(branch_name));

    let file = File::create(&config_path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &instance_config)?;

    Ok(())
}

pub fn remove_package(pkg: PackageInfo, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let instance_config = get_instance_config()?;
    let current_dir = std::env::current_dir()?;
    let carbon_folder = current_dir.join("kubejs").join("carbon");
    let config_path = current_dir.join("kubejs/carbon.json");
    let kubejs_path = current_dir.join("kubejs");

    let carbon_folder = carbon_folder.join(name);

    if carbon_folder.exists() && carbon_folder.is_dir() {
        for entry in fs::read_dir(&carbon_folder)? {
            let entry = entry?;
            let dir_name = entry.file_name();
            let dir_path = carbon_folder.join(&dir_name);

            if dir_path.is_dir() {
                for file_entry in fs::read_dir(&dir_path)? {
                    let file_entry = file_entry?;
                    let file_name = file_entry.file_name();
                    let kubejs_file_path = kubejs_path.join(&dir_name).join(&file_name);

                    if kubejs_file_path.exists() && kubejs_file_path.is_file() {
                        fs::remove_file(kubejs_file_path)?;
                    }
                }
            }
        }
    }

    fs::remove_dir_all(&carbon_folder)?;

    let file = File::open(&config_path)?;
    let reader = std::io::BufReader::new(file);
    let mut instance_config: serde_json::Value = serde_json::from_reader(reader)?;

    if !instance_config["dependencies"].is_object() {
        instance_config["dependencies"] = json!(Map::new());
    }

    let dependencies = instance_config["dependencies"].as_object_mut().unwrap();

    if !dependencies.contains_key(name) {
        error!("Package not found");
        return Ok(());
    }

    dependencies.remove(name);

    let file = File::create(&config_path)?;

    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &instance_config)?;

    info!("{}", format!("Successfully removed {:?}.", pkg.name));

    Ok(())
}
