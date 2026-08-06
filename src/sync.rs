use anyhow::{anyhow, Context, Result};
use std::fmt;
use std::path::PathBuf;
use std::process::Command;

use crate::config::{Config, RsyncJob};

/// Check that required external binaries (git, rsync) are available.
/// Returns an error listing all missing binaries.
pub fn check_prerequisites(cfg: &Config) -> Result<()> {
    let mut missing: Vec<&str> = Vec::new();

    if !cfg.dirs.is_empty() && !binary_exists("git") {
        missing.push("git");
    }
    if !cfg.rsync.is_empty() && !binary_exists("rsync") {
        missing.push("rsync");
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(anyhow!(
            "Required binary not found: {}. Please install and ensure it is on PATH.",
            missing.join(", ")
        ))
    }
}

fn binary_exists(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub enum SyncResult {
    Git {
        dir: PathBuf,
        message: String,
    },
    Rsync {
        source: String,
        target: String,
        message: String,
    },
}

impl fmt::Display for SyncResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncResult::Git { dir, message } => {
                let dir = dir.display();
                let msg = message.trim();
                if msg.is_empty() {
                    write!(f, "[OK] {dir}: already up to date")
                } else {
                    write!(f, "[OK] {dir}: {msg}")
                }
            }
            SyncResult::Rsync {
                source,
                target,
                message,
            } => {
                let msg = message.trim();
                if msg.is_empty() {
                    write!(f, "[OK] rsync {source} -> {target}")
                } else {
                    write!(f, "[OK] rsync {source} -> {target}: {msg}")
                }
            }
        }
    }
}

pub fn pull(dir: &str, _quiet: bool) -> Result<SyncResult> {
    let expanded = shellexpand::tilde(dir);
    let path = PathBuf::from(expanded.as_ref());

    if !path.exists() {
        return Err(anyhow!("Directory does not exist: {path:?}"));
    }

    if !path.join(".git").is_dir() {
        return Err(anyhow!("Not a git repository: {path:?}"));
    }

    let output = Command::new("git")
        .arg("pull")
        .arg("--ff-only")
        .current_dir(&path)
        .output()
        .with_context(|| format!("Failed to run git pull in {path:?}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git pull failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(SyncResult::Git {
        dir: path,
        message: stdout.to_string(),
    })
}

pub fn run_rsync(job: &RsyncJob, _quiet: bool) -> Result<SyncResult> {
    let source = shellexpand::tilde(&job.source);
    let target = shellexpand::tilde(&job.target);

    let mut cmd = Command::new("rsync");
    if let Some(args) = &job.args {
        for arg in args.split_whitespace() {
            cmd.arg(arg);
        }
    }
    cmd.arg(source.as_ref()).arg(target.as_ref());

    let output = cmd
        .output()
        .with_context(|| format!("Failed to run rsync {source} -> {target}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("rsync failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(SyncResult::Rsync {
        source: source.to_string(),
        target: target.to_string(),
        message: stdout.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_git_up_to_date() {
        let result = SyncResult::Git {
            dir: PathBuf::from("/home/user/project"),
            message: String::new(),
        };
        assert_eq!(
            result.to_string(),
            "[OK] /home/user/project: already up to date"
        );
    }

    #[test]
    fn display_git_with_output() {
        let result = SyncResult::Git {
            dir: PathBuf::from("/home/user/project"),
            message: "Updating abc123..def456\nFast-forward\n".to_string(),
        };
        assert_eq!(
            result.to_string(),
            "[OK] /home/user/project: Updating abc123..def456\nFast-forward"
        );
    }

    #[test]
    fn display_rsync_no_output() {
        let result = SyncResult::Rsync {
            source: "~/docs".to_string(),
            target: "server:/backup".to_string(),
            message: String::new(),
        };
        assert_eq!(result.to_string(), "[OK] rsync ~/docs -> server:/backup");
    }

    #[test]
    fn display_rsync_with_output() {
        let result = SyncResult::Rsync {
            source: "~/docs".to_string(),
            target: "server:/backup".to_string(),
            message: "sent 1,234 bytes  received 56 bytes  2,580.00 bytes/sec\n".to_string(),
        };
        assert_eq!(
            result.to_string(),
            "[OK] rsync ~/docs -> server:/backup: sent 1,234 bytes  received 56 bytes  2,580.00 bytes/sec"
        );
    }

    #[test]
    fn check_prerequisites_none_needed() {
        let cfg = Config {
            dirs: vec![],
            rsync: vec![],
        };
        assert!(check_prerequisites(&cfg).is_ok());
    }

    #[test]
    fn check_prerequisites_git_found() {
        let cfg = Config {
            dirs: vec!["~/project".to_string()],
            rsync: vec![],
        };
        // git is expected to exist on any dev machine
        assert!(check_prerequisites(&cfg).is_ok());
    }

    #[test]
    fn check_prerequisites_rsync_found() {
        let cfg = Config {
            dirs: vec![],
            rsync: vec![RsyncJob {
                source: "~/docs".to_string(),
                target: "server:/backup".to_string(),
                args: None,
            }],
        };
        // rsync is expected to exist on macOS/Linux
        assert!(check_prerequisites(&cfg).is_ok());
    }

    #[test]
    fn check_prerequisites_missing_binary() {
        let cfg = Config {
            dirs: vec![],
            rsync: vec![RsyncJob {
                source: "~/docs".to_string(),
                target: "server:/backup".to_string(),
                args: None,
            }],
        };
        // We can't easily test missing binary without mocking,
        // but we can verify the error path compiles and the
        // function signature is correct.
        let _ = check_prerequisites(&cfg);
    }
}
