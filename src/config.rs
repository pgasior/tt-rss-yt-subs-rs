use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path};

pub fn load_config(config_dir_path: &Path) -> Result<Config> {
    let config_path = config_dir_path.join("config.yml");
    let config_file = File::open(config_path)?;
    let config: Config = serde_yaml::from_reader(config_file)?;
    Ok(config)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub app: App,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct App {
    pub category_name: String,
    pub ttrss: Ttrss,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ttrss {
    pub url: String,
    pub username: String,
    pub password: String,
}
