use std::any;

use clap::Error;
use git2::Object;
use reqwest;
const API_URL: &str = "https://carbon.beanstech.tech/api";

pub async fn get_packages() -> Result<(), reqwest::Error> {
    let response = reqwest::get(API_URL.to_owned() + "/packages").await?;
    let body = response.text().await?;
    Ok(())
}

pub async fn get_package(name: &str) -> Result<serde_json::Value, std::io::Error> {
    let response = reqwest::get(&format!("{}/packages/{}", API_URL, name))
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let body = response
        .text()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let json = serde_json::from_str(&body)?;
    Ok(json)
}
