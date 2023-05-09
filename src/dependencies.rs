use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use serde_json::{json, Map, Value};

use crate::files::parser::{self, Config, KjspkgConfig};

pub fn check_if_dependency_exists(
    dependency: &str,
    current_dir: &PathBuf,
) -> Result<bool, std::io::Error> {
    let package: parser::Package =
        parser::read_package_json(current_dir.join("carbon.package.json"))?;

    if package.dependencies.is_none() {
        return Ok(false);
    }

    Ok(package.dependencies.unwrap().contains_key(dependency))
}

pub fn add_dependency(config: &Config, current_dir: &PathBuf) -> Result<(), std::io::Error> {
    let file = File::open(current_dir.join("carbon.package.json"))?;
    let reader = BufReader::new(file);
    let mut data: Map<String, Value> =
        serde_json::from_reader(reader).expect("Unable to read file.");

    let dependencies = data
        .entry("dependencies".to_owned())
        .or_insert_with(|| serde_json::Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "'Dependencies' section is not an object",
            )
        })?;

    dependencies.insert(config.name.clone(), json!(config.version));

    let file = File::create(current_dir.join("carbon.package.json"))?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &data)?;
    Ok(())
}

pub fn add_kjspkg_dependency(
    config: &KjspkgConfig,
    current_dir: &PathBuf,
    name: String,
) -> Result<(), std::io::Error> {
    let file = File::open(current_dir.join("carbon.package.json"))?;
    let reader = BufReader::new(file);
    let mut data: Map<String, Value> =
        serde_json::from_reader(reader).expect("Unable to read file.");

    let dependencies = data
        .entry("kjspkg_dependencies".to_owned())
        .or_insert_with(|| serde_json::Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "'kjspkg_dependencies' section is not an object",
            )
        })?;

    dependencies.insert(name, json!("1.0.0"));

    let file = File::create(current_dir.join("carbon.package.json"))?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &data)?;
    Ok(())
}
