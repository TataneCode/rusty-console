# Manual

This guide covers how to use rusty-console: navigation, key bindings for every screen, and the behaviours common to the whole UI.

## Common UI elements

- **Breadcrumb bar** — shows the current location (e.g. `Rusty - docker manager > Containers > web logs`), truncating leading segments first when space runs out.
- **Action bar** — the bottom bar shows available keys for the current screen; during long operations it is replaced by a loading spinner (`|/-\`, 200ms per frame).
- **Confirmation dialogs** — every destructive action (delete, prune, stop all, …) opens a confirmation dialog first.
- **Error popups** — errors are shown in a popup, dismissed on any keypress.
- **Filter mode** — pressing `/` activates a name filter; type to search, press `Esc` to clear. While the filter is active, letter-key actions (start/stop, delete, sort, refresh, …) are disabled — only typing, `Esc`, navigation (`j`/`k`/arrows) and `Enter` (open details) remain available. The action bar is contextual: while the filter is active it shrinks to `Type: Filter | j/k: Navigate | Enter | Esc: Clear`, and returns to the full list of keys when the filter is cleared.
- **Sort** — pressing `o` on a list screen cycles the name ordering: A→Z, then Z→A, then back to the default order. The chosen order survives refreshes.
- **Add dialogs** — pressing `A` on the Images or Containers screen opens a dialog to download an image or create a container (see below).
- **Refresh** — press `r` to reload the current list.

## Global keys

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` | Select / open details / confirm |
| `Esc` / `q` | Go back / quit |
| `r` | Refresh list |
| `/` | Activate filter (type to search, `Esc` to clear) |
| `o` | Cycle list sort: A→Z → Z→A → default |

## Containers

The container list shows all containers regardless of state, with live CPU and memory usage for active containers. Stats are streamed from Docker while the container screens are active, so values update without a full manual refresh.

| Key | Action |
|-----|--------|
| `s` | Start or stop the selected container |
| `o` | Cycle list sort by name (A→Z → Z→A → default) |
| `A` | Open the Create Container dialog (name, image, port mappings; downloads the image first if needed, then creates and starts the container) |
| `p` | Pause or unpause the selected container |
| `R` | Restart the selected container |
| `e` | Open the exec shell picker, then launch `docker exec -it` with `sh` or `bash` in an external terminal window when one is available, otherwise inline (TUI suspended) |
| `l` | View container logs |
| `Enter` | View container details (including live network I/O) |
| `d` | Delete (opens confirmation dialog) |
| `X` | Prune all stopped containers |

#
### Exec in an external terminal

When a terminal emulator is available, the exec shell opens in a new terminal window
and the TUI stays running in the background. Detection order:

1. `RUSTY_CONSOLE_TERMINAL` environment variable (binary name or absolute path)
2. Common emulators on `PATH`: `x-terminal-emulator`, `gnome-terminal`, `konsole`,
   `xfce4-terminal`, `kitty`, `alacritty`, `wezterm`, `xterm` (Linux); `wt.exe`,
   `cmd.exe`, `powershell.exe` (Windows)

If no emulator is found (e.g. headless server over SSH), the previous inline
behavior is used: the TUI is suspended and `docker exec -it` runs in the same
terminal, then the TUI resumes when the shell exits.

## Log view

The selected log line is highlighted. Copying copies the line as displayed
(timestamp included when present) to the system clipboard (cross-platform).

| Key | Action |
|-----|--------|
| `j` / `k` or arrows | Move line selection (highlighted) |
| `PgUp` / `PgDn` | Page up / down |
| `Ctrl+U` | Scroll up |
| `Ctrl+D` | Scroll down |
| `Enter` | Expand the selected line (full text, wrapped) |
| `y` | Copy the selected line to the clipboard |
| `/` | Search in logs (case-insensitive) |
| `n` / `N` | Next / previous search match |
| `E` | Export all logs to a `.log` file |
| `Esc` / `q` | Back |

### Realtime stats

- The **container list** and the **stack containers** screen show live **CPU** and **Memory** values for active containers.
- **Network I/O** is shown only in **container details**.
- Stats are streamed from Docker while the container screens are active, so values update without a full manual refresh. Logs and details opened from the stack containers screen return to that screen with `Esc`/`q`.

## Stacks

Stacks are detected from Docker Compose container labels. The stack list shows running/total counts per stack.

| Key | Action |
|-----|--------|
| `Enter` | Drill down into the stack's containers |
| `s` | Start all containers in the selected stack |
| `S` | Stop all containers in the selected stack |
| `D` | Remove all containers in the selected stack (force, with confirmation) |
| `o` | Cycle list sort by name (A→Z → Z→A → default) |

### Stack containers (drill-down)

This screen offers the same features as the Containers list (same columns, live CPU/memory, filter, sort, logs, details, exec, start/stop, pause, restart, delete, prune, add). Stack-level actions live on the stack list instead.

| Key | Action |
|-----|--------|
| `s` | Start or stop the selected container |
| `p` | Pause or unpause the selected container |
| `R` | Restart the selected container |
| `l` | View container logs |
| `Enter` | View container details (including live network I/O) |
| `e` | Open the exec shell picker for the selected container |
| `d` | Delete the selected container (opens confirmation dialog) |
| `/` | Filter containers by name or image |
| `o` | Cycle list sort by name (A→Z → Z→A → default) |
| `X` | Prune all stopped containers |
| `A` | Open the Create Container dialog (name, image, port mappings; downloads the image first if needed, then creates and starts the container) |
| `r` | Refresh the stack's containers |

## Volumes

The volume list shows all volumes and flags which ones are in use.

| Key | Action |
|-----|--------|
| `Enter` | View volume details |
| `d` | Delete (opens confirmation dialog) |
| `X` | Prune unused volumes |
| `o` | Cycle list sort by name (A→Z → Z→A → default) |

## Images

The image list shows all images with their usage status.

| Key | Action |
|-----|--------|
| `Enter` | View image details |
| `d` | Delete (opens confirmation dialog) |
| `X` | Prune dangling images |
| `o` | Cycle list sort by name (A→Z → Z→A → default) |
| `A` | Open the Download Image dialog (single field, e.g. `nginx:1.25`; a name without tag defaults to `:latest`) |

### Image details

| Key | Action |
|-----|--------|
| `P` | Pull the latest version of this image (shows a `docker pull`-style progress bar with downloaded/total bytes) |
| `r` | Refresh the image list and details |
| `Esc` / `q` | Back to the image list |

Notes on pulling:

- Only the image currently displayed is pulled; already-up-to-date images (matching registry digest) are skipped and reported as such.
- Registry credentials are read from `~/.docker/config.json`, like the Docker CLI — no in-app `docker login` is required.
- Dangling images (`<none>` repository) cannot be pulled and show an explicit error.
- When the pull completes, the details screen refreshes in place; if the image no longer exists locally, you are returned to the image list.

## Networks

The network list shows name, driver, scope, subnet, and connected container count.

| Key | Action |
|-----|--------|
| `Enter` | View network details (driver, scope, subnet, gateway, connected containers) |
| `d` | Delete the selected network (opens confirmation dialog; predefined networks and networks with connected containers are refused) |
| `X` | Prune unused networks |
| `/` | Filter networks by name |
| `o` | Cycle list sort by name (A→Z → Z→A → default) |
| `r` | Refresh |

Predefined networks (`bridge`, `host`, `none`) are protected and cannot be deleted.

### Network details

| Key | Action |
|-----|--------|
| `j` / `k` | Select a connected container |
| `d` | Disconnect the selected container from the network (opens confirmation dialog) |
| `r` | Refresh |
| `Esc` / `q` | Back to the network list |
