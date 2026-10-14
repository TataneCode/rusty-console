# Lesson 9 — Packaging: Generating Installers (MSI, RPM)

## Goal

Understand how a Rust binary like `rusty_console` is packaged into system installers — a `.msi` for Windows and an `.rpm` for Linux — and published automatically as GitHub Release assets when a version tag is pushed.

## Why package a CLI/TUI binary?

A raw binary (`rusty_console` / `rusty_console.exe`) works, but users must:

- find a place to put it,
- make it discoverable from the shell (`PATH`),
- set executable permissions manually on Linux (`chmod +x`).

An installer automates all of this: it puts the binary in the right directory and registers that directory on the `PATH`.

## The three requirements

| Requirement | MSI (Windows) | RPM (Linux) |
|---|---|---|
| Install into the user's directory | `%LOCALAPPDATA%\rusty_console` (per-user, no admin rights) | `/usr/bin/rusty_console` (standard system location) |
| Add to `PATH` | `HKCU\Environment` → user `PATH` variable | `/usr/bin` is already on `PATH` |
| Executable bit | Not applicable on Windows | `mode = "755"` (`rwxr-xr-x`) in the RPM metadata |

> **Note on APK (Android):** an Android installer was considered and deliberately excluded. `rusty_console` is a terminal UI (crossterm/ratatui) that talks to the Docker daemon over a Unix socket. Android provides neither a standard terminal nor an accessible Docker daemon, so an APK would ship a binary that cannot run. Packaging only formats where the application is actually usable is part of good release engineering.

## Where the packaging configuration lives

### RPM — `Cargo.toml`

`cargo-generate-rpm` reads a metadata table inside `Cargo.toml`:

```toml
[package.metadata.generate-rpm]
assets = [
    { source = "target/x86_64-unknown-linux-gnu/release/rusty_console", dest = "/usr/bin/rusty_console", mode = "755" },
    { source = "LICENSE", dest = "/usr/share/licenses/rusty_console/LICENSE", mode = "644", doc = true },
]
```

- `source`: the built binary path (relative to the project root).
- `dest`: install destination — `/usr/bin` is already on every user's `PATH`.
- `mode = "755"`: **this is the `chmod +x` requirement** — the executable bit is set for everyone, and read/execute for group/others. The install step does it for you.
- `doc = true`: marks the file as documentation, so `rpm -qc rusty_console` lists it.

Build locally:

```sh
cargo build --release --target x86_64-unknown-linux-gnu
cargo install cargo-generate-rpm --locked
cargo generate-rpm --target x86_64-unknown-linux-gnu
# result: target/generate-rpm/rusty_console-<version>-1.x86_64.rpm
```

Install and verify:

```sh
sudo rpm -i rusty_console-*.rpm
which rusty_console        # → /usr/bin/rusty_console
rusty_console --version    # PATH resolution works
```

### MSI — `wix/main.wxs`

The MSI is built from a WiX v4 source file, `wix/main.wxs`. Key parts:

```xml
<Package Name="rusty_console" Scope="perUser" Version="$(var.Version)"
         UpgradeCode="7e63fcfd-e8d0-479a-bf9e-fbaf5f9bcc1c" ...>
```

- `Scope="perUser"`: installs for the current user only, under `%LOCALAPPDATA%\rusty_console`, **without requiring admin rights**.
- `Version="$(var.Version)"`: `cargo wix` injects the package version at build time, so the MSI version always matches the Git tag.
- `UpgradeCode`: a fixed GUID that identifies the product across versions. It must never change once published — it is what lets a new MSI upgrade (or uninstall) an old installation.

The `PATH` registration is a WiX `Environment` element:

```xml
<Component Id="Path" Guid="...">
    <Environment Id="UserPath" Name="PATH" Value="[INSTALLFOLDER]"
                 Permanent="no" Part="last" Action="set" System="no" />
</Component>
```

- `System="no"`: modifies the **user** PATH (`HKCU\Environment`), not the machine-wide one — consistent with a per-user install.
- `Part="last"`: appends the install directory after existing entries.
- `Permanent="no"`: removing the application restores the PATH to its previous state.

> A WiX quirk worth knowing: a `Component` that only writes an environment variable needs a `RegistryValue` (here under `HKCU\Software\TataneCode\rusty_console`) to serve as its key path — WiX refuses components without a key path, and registry keys are the standard trick for per-user components.

Build locally (Windows):

```powershell
cargo build --release --target x86_64-pc-windows-msvc
dotnet tool install --global wix
cargo install cargo-wix --locked
cargo wix --toolset modern --no-build -o rusty_console.msi
```

`--no-build` reuses the binary already compiled — in CI the workflow builds once and packages once.

Install and verify:

```powershell
msiexec /i rusty_console.msi
# new terminal (PATH changes are not visible in already-open shells)
rusty_console
```

## The release pipeline — `.github/workflows/release.yml`

The workflow runs only on `v*` tags. Each matrix entry builds its target, then packages the native installer before upload:

- **Linux job**: `cargo install cargo-generate-rpm` → `cargo generate-rpm` → uploads both the plain binary and the `.rpm`.
- **Windows job**: `dotnet tool install --global wix` (WiX v4 toolchain) → `cargo install cargo-wix` → `cargo wix --toolset modern --no-build` → uploads the binary and the `.msi`.

The `release` job downloads every artifact and attaches them all to the GitHub Release (`softprops/action-gh-release`), so a tag like `v0.1.0` publishes:

```
rusty_console-linux-x86_64        (binary, chmod +x applied)
rusty_console-linux-x86_64.rpm    (RPM installer)
rusty_console-windows-x86_64.exe  (binary)
rusty_console-windows-x86_64.msi  (MSI installer, per-user + PATH)
```

## Releasing a new version

```sh
# 1. bump `version` in Cargo.toml
# 2. commit
git tag v0.2.0
git push origin main --tags
# 3. the Release workflow builds, packages and publishes the installers
```

## Key takeaways

- A packaging tool reads metadata (`Cargo.toml` table or `.wxs` template) and turns a raw binary into a system-managed installation.
- "Inscription au PATH" means different things per OS: the MSI writes the user's `PATH` environment value in the registry; the RPM simply relies on `/usr/bin` being on `PATH` by default.
- "chmod +x" is expressed declaratively in RPM metadata as `mode = "755"` — no shell step needed at install time.
- Per-user installs (`Scope="perUser"`, `%LOCALAPPDATA%`) avoid UAC elevation and let the tool install on locked-down machines.
- Excluding an unviable format (APK) is a packaging decision too: installers exist to make software *usable*, and a terminal Docker client cannot run on Android.

## Files to study

- [`Cargo.toml`](../Cargo.toml) — `[package.metadata.generate-rpm]` section
- [`wix/main.wxs`](../wix/main.wxs) — the full WiX source for the per-user MSI
- [`.github/workflows/release.yml`](../.github/workflows/release.yml) — the tag-driven build/package/publish pipeline
