use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub dirs: Vec<String>,

    #[serde(default)]
    pub rsync: Vec<RsyncJob>,
}

#[derive(Debug, Deserialize)]
pub struct RsyncJob {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub args: Option<String>,
}

pub fn default_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".config").join("multisync.yml"))
}

pub fn load(path: &PathBuf) -> Result<Config> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Could not read config file {path:?}"))?;
    let cfg: Config = yaml_serde::from_str(&contents)
        .with_context(|| format!("Could not parse config file {path:?}"))?;
    Ok(cfg)
}

#[cfg(test)]
pub fn parse_from_str(yaml: &str) -> Result<Config> {
    yaml_serde::from_str(yaml).context("Could not parse config")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_config() {
        let cfg = parse_from_str("").unwrap();
        assert!(cfg.dirs.is_empty());
        assert!(cfg.rsync.is_empty());
    }

    #[test]
    fn parse_dirs_only() {
        let yaml = "dirs:\n  - ~/projects/foo\n  - ~/dotfiles\n";
        let cfg = parse_from_str(yaml).unwrap();
        assert_eq!(cfg.dirs.len(), 2);
        assert_eq!(cfg.dirs[0], "~/projects/foo");
        assert_eq!(cfg.dirs[1], "~/dotfiles");
        assert!(cfg.rsync.is_empty());
    }

    #[test]
    fn parse_rsync_with_args() {
        let yaml =
            "rsync:\n  - source: ~/docs\n    target: server:/backup\n    args: \"-avz --delete\"\n";
        let cfg = parse_from_str(yaml).unwrap();
        assert_eq!(cfg.rsync.len(), 1);
        assert_eq!(cfg.rsync[0].source, "~/docs");
        assert_eq!(cfg.rsync[0].target, "server:/backup");
        assert_eq!(cfg.rsync[0].args.as_deref(), Some("-avz --delete"));
    }

    #[test]
    fn parse_rsync_without_args() {
        let yaml = "rsync:\n  - source: ~/docs\n    target: server:/backup\n";
        let cfg = parse_from_str(yaml).unwrap();
        assert_eq!(cfg.rsync[0].args, None);
    }

    #[test]
    fn parse_full_config() {
        let yaml =
            "dirs:\n  - ~/projects/foo\nrsync:\n  - source: ~/docs\n    target: server:/backup\n";
        let cfg = parse_from_str(yaml).unwrap();
        assert_eq!(cfg.dirs.len(), 1);
        assert_eq!(cfg.rsync.len(), 1);
    }

    #[test]
    fn parse_invalid_yaml_is_error() {
        let result = parse_from_str("dirs: [unclosed");
        assert!(result.is_err());
    }
}
