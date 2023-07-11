use std::{collections::HashMap, path::Path};

use crate::files;
use serde::Deserialize;

extern crate simplelog;

use simplelog::*;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub minecraftVersion: Vec<String>,
    pub modloaders: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct InstanceConfig {
    pub version: String,
    pub modloader: String,
    pub dependencies: Option<HashMap<String, String>>,
}

pub fn get_config(temp_dir: &Path) -> Result<Config, std::io::Error> {
    let config_path = get_config_path(temp_dir)?;

    if !files::check_if_path_exists(&config_path)? {
        error!("Could not find config file. Please contact script author.");
    }

    let config_file = std::fs::File::open(config_path)?;

    let config_reader = std::io::BufReader::new(config_file);

    let config: Config = serde_json::from_reader(config_reader).unwrap();
    Ok(config)
}

pub fn get_config_path(tmp_dir: &Path) -> Result<std::path::PathBuf, std::io::Error> {
    Ok(tmp_dir.join("carbon.config.json"))
}

pub fn get_instance_config() -> Result<InstanceConfig, std::io::Error> {
    let current_dir = std::env::current_dir()?;
    let config_path = current_dir.join("kubejs/carbon.json");

    if !files::check_if_path_exists(&config_path)? {
        error!("Could not find instance configuration file. Please create carbon.json file in your KubeJS folder. \n\nIt should contain: version (minecraft version) and modloader.");
    }

    let config_file = std::fs::File::open(config_path)?;

    let config_reader = std::io::BufReader::new(config_file);

    let config: InstanceConfig = serde_json::from_reader(config_reader)?;
    Ok(config)
}
