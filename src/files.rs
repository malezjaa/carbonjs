use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use serde_json;

use crate::dependencies::{self, check_if_dependency_exists};

use self::parser::Package;

pub(crate) mod parser;

struct Files {
    temp_dir: PathBuf,
    kjspkg_files: Vec<PathBuf>,
    startup_scripts: Vec<PathBuf>,
    client_scripts: Vec<PathBuf>,
    server_scripts: Vec<PathBuf>,
}

pub fn check_if_folder_exists(folder_name: &PathBuf) -> Result<bool, std::io::Error> {
    match fs::metadata(folder_name) {
        Ok(metadata) => Ok(true),
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => Ok(false),
            _ => Err(e),
        },
    }
}

fn get_files_in_folder(folder: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();

    for entry in fs::read_dir(folder)? {
        let path = entry?.path();
        if path.is_file() {
            files.push(path);
        }
    }

    Ok(files)
}

fn copy_dir_contents(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        if !dst.exists() {
            fs::create_dir(dst)?;
        }

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let dst_path = dst.join(file_name);

            if entry.file_type()?.is_dir() {
                copy_dir_contents(&path, &dst_path)?;
            } else {
                fs::copy(&path, &dst_path)?;
            }
        }
    }

    Ok(())
}

pub fn copy_files_to_dir_and_remove_temp_dir(
    files: (
        PathBuf,
        Vec<PathBuf>,
        Vec<PathBuf>,
        Vec<PathBuf>,
        Vec<PathBuf>,
    ),
    scripts_dir: &[&str; 3],
    current_dir: &PathBuf,
) -> Result<bool, std::io::Error> {
    let temp_dir = files.0;
    let carbon_files = files.1;
    let startup_scripts = files.2;
    let client_scripts = files.3;
    let server_scripts = files.4;

    if !temp_dir.is_dir() {
        fs::create_dir(&temp_dir)?;
    }

    let config: parser::Config = parser::read_config_json(temp_dir.join("carbon.config.json"))?;
    let carbon_file = &current_dir.join("carbon.package.json");

    if !check_if_folder_exists(carbon_file)? {
        let mut file = File::create(current_dir.join("carbon.package.json"))?;
    }

    let package: parser::Package =
        parser::read_package_json(current_dir.join("carbon.package.json"))?;

    if package.version != config.minecraftVersion {
        println!(
            "[{}] {}",
            "error".red().bold(),
            format!("The version of the script does not match the version of the minecraft version in your instance.")
        );

        return panic!();
    }

    if !config.modloaders.contains(&package.modloader) {
        println!(
            "[{}] {}",
            "error".red().bold(),
            format!("Your current modloader does not match the modloader of the script.")
        );

        return panic!();
    }

    match check_if_dependency_exists(&config.name, &current_dir) {
        Ok(true) => {
            if let Some(dependencies) = package.dependencies.get(&config.name) {
                if dependencies == &config.version {
                    for dir in scripts_dir {
                        let dir = PathBuf::from(dir);
                        let source_path = temp_dir.join(&dir);
                        let target_path = current_dir.join(&dir);

                        if check_if_folder_exists(&source_path)? {
                            copy_dir_contents(&source_path, &target_path)?;
                        }
                    }
                    println!(
                        "[{}] {}",
                        "info".blue().bold(),
                        format!(
                            "This script is up to date with the latest version of {} ({}).",
                            &config.name, dependencies
                        )
                    );

                    return Ok((true));
                } else {
                    for dir in scripts_dir {
                        let dir = PathBuf::from(dir);
                        let source_path = temp_dir.join(&dir);
                        let target_path = current_dir.join(&dir);

                        if check_if_folder_exists(&source_path)? {
                            copy_dir_contents(&source_path, &target_path)?;
                        }
                    }
                    dependencies::add_dependency(&config, current_dir);

                    println!(
                        "[{}] {}",
                        "success".green().bold(),
                        format!(
                            "Successfully installed {} with the version of ({}).",
                            config.name, config.version
                        )
                    );

                    return Ok((true));
                }
            }
        }
        Ok(false) => {
            for dir in scripts_dir {
                let dir = PathBuf::from(dir);
                let source_path = temp_dir.join(&dir);
                let target_path = current_dir.join(&dir);

                if check_if_folder_exists(&source_path)? {
                    copy_dir_contents(&source_path, &target_path)?;
                }
            }

            dependencies::add_dependency(&config, current_dir);

            println!(
                "[{}] {}",
                "success".green().bold(),
                format!(
                    "Successfully installed {} with the version of ({}).",
                    config.name, config.version
                )
            );

            return Ok((true));
        }
        Err(e) => {
            println!("[{}] {}", "error".red().bold(), e);
        }
    }

    //TODO:odkomentuj
    // fs::remove_dir_all(&temp_dir)?;

    Ok(true)
}
