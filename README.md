# multisync

A small Rust CLI tool to pull multiple git repositories sequentially.

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
  - ~/projects/bar
  - ~/work/baz
```

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

- `0`: all repositories were pulled successfully
- `1`: at least one pull failed or no directories were configured
