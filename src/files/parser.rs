use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{BufRead, BufReader, ErrorKind, Read},
    path::PathBuf,
};

use colored::Colorize;
use serde::Deserialize;
use serde_json;

use super::check_if_folder_exists;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub author: String,
    pub minecraftVersion: String,
    pub description: String,
    pub modloaders: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Package {
    pub version: String,
    pub modloader: String,
    pub dependencies: HashMap<String, String>,
}

pub fn read_config_json(file_path: PathBuf) -> Result<Config, std::io::Error> {
    if !check_if_folder_exists(&file_path)? {
        println!(
            "[{}] {}",
            "error".red().bold(),
            format!(
                "Package's configuration file does not exist. Report it to the package's author."
            )
        );

        return panic!();
    }
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let person = serde_json::from_reader(reader)?;

    Ok(person)
}

pub fn read_package_json(file_path: PathBuf) -> Result<Package, std::io::Error> {
    if !check_if_folder_exists(&file_path)? {
        let mut file = File::create(&file_path)?;
    }
    let file = File::open(file_path)?;
    let mut reader: BufReader<File> = BufReader::new(file);
    let mut buffer = String::new();

    if reader.fill_buf()?.is_empty() {
        println!(
            "[{}] {}",
            "error".red().bold(),
            format!("Package's configuration file is empty.")
        );

        return panic!();
    }

    let person = serde_json::from_reader(reader)?;

    Ok(person)
}
