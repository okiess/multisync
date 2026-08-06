use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod config;
mod sync;

#[derive(Parser, Debug)]
#[command(name = "multisync")]
#[command(version)]
struct Cli {
    /// Path to the configuration file.
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Only print errors.
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config_path = match cli.config {
        Some(path) => path,
        None => config::default_config_path()?,
    };

    let cfg = config::load(&config_path)?;

    if cfg.dirs.is_empty() && cfg.rsync.is_empty() {
        eprintln!("No directories or rsync jobs configured in {config_path:?}");
        std::process::exit(1);
    }

    let mut failed = 0;

    for dir in &cfg.dirs {
        match sync::pull(dir, cli.quiet) {
            Ok(result) => {
                if !cli.quiet {
                    println!("{result}");
                }
            }
            Err(err) => {
                failed += 1;
                eprintln!("[FAIL] git {dir:?}: {err:#}");
            }
        }
    }

    for job in &cfg.rsync {
        match sync::run_rsync(job, cli.quiet) {
            Ok(result) => {
                if !cli.quiet {
                    println!("{result}");
                }
            }
            Err(err) => {
                failed += 1;
                eprintln!("[FAIL] rsync {} -> {}: {err:#}", job.source, job.target);
            }
        }
    }

    if failed > 0 {
        eprintln!("{failed} sync job(s) failed.");
        std::process::exit(1);
    }

    Ok(())
}
