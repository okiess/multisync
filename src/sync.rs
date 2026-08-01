use anyhow::{anyhow, Context, Result};
use std::fmt;
use std::path::PathBuf;
use std::process::Command;

use crate::config::RsyncJob;

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
