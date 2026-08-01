use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub dirs: Vec<String>,
}

pub fn default_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".config").join("multisync.yml"))
}

pub fn load(path: &PathBuf) -> Result<Config> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Could not read config file {path:?}"))?;
    let cfg: Config = serde_yaml::from_str(&contents)
        .with_context(|| format!("Could not parse config file {path:?}"))?;
    Ok(cfg)
}
