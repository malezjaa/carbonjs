use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use serde_json::{self, json, to_string_pretty, Value};

use crate::dependencies::{
    self, add_dependency, add_kjspkg_dependency, check_if_dependency_exists,
};

use self::parser::Package;

pub(crate) mod parser;

struct Files {
    temp_dir: PathBuf,
    kjspkg_files: Vec<PathBuf>,
    startup_scripts: Vec<PathBuf>,
    client_scripts: Vec<PathBuf>,
    server_scripts: Vec<PathBuf>,
    assets: Vec<PathBuf>,
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
        Vec<PathBuf>,
    ),
    scripts_dir: &[&str; 4],
    current_dir: &PathBuf,
) -> Result<bool, std::io::Error> {
    let temp_dir = files.0;
    let carbon_files = files.1;
    let startup_scripts = files.2;
    let client_scripts = files.3;
    let server_scripts = files.4;
    let assets = files.5;

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
            if let Some(dependencies) = package.dependencies.unwrap().get(&config.name) {
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

            if let Err(e) = add_dependency(&config, &current_dir) {
                if e.kind() == std::io::ErrorKind::InvalidData {
                    println!(
                        "[{}] {}",
                        "info".blue().bold(),
                        format!("'dependencies' section not found, adding it...",)
                    );
                    add_dependency(&config, &current_dir).unwrap();
                } else {
                    panic!("Error: {}", e);
                }
            }

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

const VERSIONS: [(&str, i32); 10] = [
    ("1.12.2", 2),
    ("1.12", 2),
    ("1.16.5", 6),
    ("1.16", 6),
    ("1.18.2", 8),
    ("1.18", 8),
    ("1.19.2", 9),
    ("1.19.3", 9),
    ("1.19.4", 9),
    ("1.19", 9),
];

pub fn copy_kjspkg_package(
    files: (
        PathBuf,
        Vec<PathBuf>,
        Vec<PathBuf>,
        Vec<PathBuf>,
        Vec<PathBuf>,
        Vec<PathBuf>,
    ),
    scripts_dir: &[&str; 4],
    current_dir: &PathBuf,
    name: &str,
    is_carbon: bool,
) -> Result<bool, std::io::Error> {
    let temp_dir = files.0;
    let carbon_files = files.1;
    let startup_scripts = files.2;
    let client_scripts = files.3;
    let server_scripts = files.4;
    let assets = files.5;

    if !temp_dir.is_dir() {
        fs::create_dir(&temp_dir)?;
    }

    let config: parser::KjspkgConfig = parser::read_kjpkg_conifg(temp_dir.join(".kjspkg"))?;
    let carbon_file = &current_dir.join("carbon.package.json");

    if !check_if_folder_exists(carbon_file)? {
        let mut file = File::create(current_dir.join("carbon.package.json"))?;
    }

    let package: parser::Package =
        parser::read_package_json(current_dir.join("carbon.package.json"))?;

    if let Some((_, version_value)) = VERSIONS
        .iter()
        .find(|(version_number, _)| *version_number == package.version)
    {
        if !config.versions.contains(&version_value) {
            println!(
                    "[{}] {}",
                    "error".red().bold(),
                    format!("The version of the script does not match any of the minecraft versions in your instance.")
                );
            panic!();
        }
    } else {
        println!(
            "[{}] {}",
            "error".red().bold(),
            format!("The version of the script is not recognized.")
        );
        panic!();
    }

    if !config.modloaders.contains(&package.modloader) {
        println!(
            "[{}] {}",
            "error".red().bold(),
            format!("Your current modloader does not match the modloader of the script.")
        );

        return panic!();
    }

    if let Err(e) = add_kjspkg_dependency(&config, &current_dir, std::string::String::from(name)) {
        if e.kind() == std::io::ErrorKind::InvalidData {
            println!(
                "[{}] {}",
                "info".blue().bold(),
                format!("'kjspkg_dependencies' section not found, adding it...",)
            );
            add_kjspkg_dependency(&config, &current_dir, std::string::String::from(name));
        } else {
            panic!("Error: {}", e);
        }
    }

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
        "success".green().bold(),
        format!("Successfully installed {}.", name)
    );

    return Ok((true));
}

pub fn create_hidden_folder(kubejs_dir: &PathBuf) -> Result<bool, std::io::Error> {
    let file_name = "carbon";
    let path = Path::new(kubejs_dir).join(file_name);

    if !path.exists() {
        fs::create_dir(path)?;
    }

    Ok((true))
}

pub fn remove_package(package_name: &str, current_dir: &PathBuf) -> std::io::Result<()> {
    let current_dir = std::env::current_dir().unwrap();
    let carbon_path = current_dir.join("kubejs").join("carbon.package.json");
    let kubejs_path = current_dir.join("kubejs");

    let mut carbon_data = fs::read_to_string(&carbon_path).unwrap();
    let mut carbon_json: Value = serde_json::from_str(&carbon_data).unwrap();
    let carbon_folder = current_dir.join("kubejs").join("carbon").join(package_name);

    if let Some(deps) = carbon_json.get_mut("dependencies") {
        if deps.get(package_name).is_some() {
            deps[package_name] = Value::Null;
            if let Some(deps) = carbon_json
                .get_mut("dependencies")
                .and_then(Value::as_object_mut)
            {
                deps.remove(package_name);
            }
            carbon_data = serde_json::to_string_pretty(&carbon_json).unwrap();
            fs::write(&carbon_path, carbon_data)?;

            let paths = fs::read_dir(&carbon_folder).unwrap();

            for path in paths {
                let path = path.unwrap();
                if path.file_type().unwrap().is_dir() {
                    let dir_name = path.file_name();
                    let dir_path = kubejs_path.join(dir_name);
                    remove_files_in_dir(&dir_path);
                } else {
                    let file_name = path.file_name();
                    let file_path = current_dir.join(&file_name);
                    if file_path.exists() && file_path.is_file() {
                        fs::remove_file(file_path).unwrap();
                    }
                }
            }

            fs::remove_dir_all(carbon_folder)?;

            println!(
                "[{}] {}",
                "success".bright_green().bold(),
                format!("Successfully removed {}", package_name)
            );
            return panic!();
        } else {
            println!(
                "[{}] {}",
                "error".red().bold(),
                format!("Package {} not found in dependencies", package_name)
            );
            return panic!();
        }
    }

    Ok(())
}

fn remove_files_in_dir(dir_path: &Path) {
    if dir_path.exists() && dir_path.is_dir() {
        let inner_paths = fs::read_dir(dir_path).unwrap();
        for inner_path in inner_paths {
            let inner_path = inner_path.unwrap();
            if inner_path.file_type().unwrap().is_dir() {
                let inner_dir_name = inner_path.file_name();
                let inner_dir_path = dir_path.join(inner_dir_name);
                remove_files_in_dir(&inner_dir_path);
            } else {
                let file_name = inner_path.file_name();
                let file_path = dir_path.join(file_name);
                if file_path.exists() && file_path.is_file() {
                    fs::remove_file(file_path).unwrap();
                }
            }
        }
    }
}
