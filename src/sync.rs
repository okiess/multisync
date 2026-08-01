use anyhow::{anyhow, Context, Result};
use std::fmt;
use std::path::PathBuf;
use std::process::Command;

pub enum PullResult {
    Success { dir: PathBuf, message: String },
}

impl fmt::Display for PullResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PullResult::Success { dir, message } => {
                let dir = dir.display();
                let msg = message.trim();
                if msg.is_empty() {
                    write!(f, "[OK] {dir}: already up to date")
                } else {
                    write!(f, "[OK] {dir}: {msg}")
                }
            }
        }
    }
}

pub fn pull(dir: &str, _quiet: bool) -> Result<PullResult> {
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
    Ok(PullResult::Success {
        dir: path,
        message: stdout.to_string(),
    })
}
