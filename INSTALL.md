# Installation

## Prerequisites

- Rust toolchain (`rustup` recommended) — MSRV 1.88
- A running Docker daemon (accessible via the default socket)
- Docker CLI installed and available on `PATH` (`docker`) for interactive exec/shell access

## Prebuilt installers

Prebuilt installers are attached to every [GitHub Release](https://github.com/TataneCode/rusty-console/releases):

- **Windows** — `rusty_console-windows-x86_64.zip`: portable archive containing `rusty_console.exe`; extract it anywhere and add its folder to your `PATH` (or run it directly). The MSI installer was dropped in favor of this simpler, more reliable cross-compiled zip
- **Linux (Debian/Ubuntu)** — `rusty_console-linux-x86_64-amd64.deb`: installs to `/usr/bin/rusty_console` with executable permissions
- **Linux (RHEL/Fedora)** — `rusty_console-linux-x86_64.rpm`: installs to `/usr/bin/rusty_console` with executable permissions (`755`)
- Plain binaries are also published for both platforms

## Build from source

```bash
# Run in development mode
cargo run

# Build an optimized binary
cargo build --release
./target/release/rusty_console
```

## Testing

Unit tests live alongside production code in `#[cfg(test)]` modules. Repository traits are mocked with `mockall` so services can be tested in isolation without a Docker daemon.

```bash
cargo test              # run all tests
cargo test <name>       # run a specific test by name
```
