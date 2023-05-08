use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use serde_json::{json, Map, Value};

use crate::files::parser::{self, Config};

pub fn check_if_dependency_exists(
    dependency: &str,
    current_dir: &PathBuf,
) -> Result<bool, std::io::Error> {
    let package: parser::Package =
        parser::read_package_json(current_dir.join("carbon.package.json"))?;

    Ok(package.dependencies.contains_key(dependency))
}

pub fn add_dependency(config: &Config, current_dir: &PathBuf) -> Result<(), std::io::Error> {
    let file = File::open(current_dir.join("carbon.package.json"))?;
    let reader = BufReader::new(file);
    let mut data: Map<String, Value> =
        serde_json::from_reader(reader).expect("Unable to read file.");

    let dependencies = data
        .get_mut("dependencies")
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "'Dependencies' section is missing",
            )
        })?
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
