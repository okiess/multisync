# multisync

A small Rust CLI tool to sync git repositories and run rsync jobs sequentially.

## Installation

```bash
cargo build --release
# Binary is then available at:
./target/release/multisync
```

## Configuration

Create a config file at `~/.config/multisync.yml`:

```yaml
dirs:
  - ~/projects/foo
  - ~/dotfiles

rsync:
  - source: ~/Documents/notes
    target: user@server:/backup/notes
    args: "-avz --delete"
```

- `dirs`: list of git repositories to pull
- `rsync`: list of rsync jobs to run

Tilde (`~`) is expanded to your home directory.

## Usage

```bash
multisync
```

Use a custom config file:

```bash
multisync --config /path/to/multisync.yml
```

Only print errors:

```bash
multisync --quiet
```

## Exit codes

- `0`: all sync jobs succeeded
- `1`: at least one sync job failed or nothing was configured

## Notes

- Git repositories are pulled with `git pull --ff-only`.
- Failed jobs are reported, but `multisync` continues with the remaining jobs.
- `rsync` must be installed on your system to use the `rsync` section.
