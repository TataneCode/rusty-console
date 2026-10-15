# Manual

This guide covers how to use rusty-console: navigation, key bindings for every screen, and the behaviours common to the whole UI.

## Common UI elements

- **Breadcrumb bar** — shows the current location (e.g. `Rusty - docker manager > Containers > web logs`), truncating leading segments first when space runs out.
- **Action bar** — the bottom bar shows available keys for the current screen; during long operations it is replaced by a loading spinner (`|/-\`, 200ms per frame).
- **Confirmation dialogs** — every destructive action (delete, prune, stop all, …) opens a confirmation dialog first.
- **Error popups** — errors are shown in a popup, dismissed on any keypress.
- **Filter mode** — pressing `/` activates a name filter; type to search, press `Esc` to clear.
- **Refresh** — press `r` to reload the current list.

## Global keys

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` | Select / confirm |
| `Esc` / `q` | Go back / quit |
| `r` | Refresh list |
| `/` | Activate filter (type to search, `Esc` to clear) |

## Containers

The container list shows all containers regardless of state, with live CPU and memory usage for active containers. Stats are streamed from Docker while the container screens are active, so values update without a full manual refresh.

| Key | Action |
|-----|--------|
| `s` | Start or stop the selected container |
| `p` | Pause or unpause the selected container |
| `R` | Restart the selected container |
| `e` | Open the exec shell picker, then launch `docker exec -it` with `sh` or `bash` |
| `l` | View container logs |
| `c` | View container details (including live network I/O) |
| `d` | Delete (opens confirmation dialog) |
| `X` | Prune all stopped containers |

### Log view

| Key | Action |
|-----|--------|
| `Ctrl+U` | Scroll up |
| `Ctrl+D` | Scroll down |

### Realtime stats

- The **container list** shows live **CPU** and **Memory** values for active containers.
- **Network I/O** is shown only in **container details**.
- Stats are streamed from Docker while the container screens are active, so values update without a full manual refresh.

## Stacks

Stacks are detected from Docker Compose container labels. The stack list shows running/total counts per stack.

| Key | Action |
|-----|--------|
| `Enter` | Drill down into the stack's containers |
| `s` | Start all containers in the selected stack |
| `S` | Stop all containers in the selected stack |

### Stack containers (drill-down)

| Key | Action |
|-----|--------|
| `s` | Start or stop the selected container |
| `e` | Open the exec shell picker for the selected container |
| `d` | Delete the selected container |
| `Ctrl+S` | Start all containers in the stack |
| `S` | Stop all containers in the stack |
| `D` | Remove all containers in the stack (force, with confirmation) |

## Volumes

The volume list shows all volumes and flags which ones are in use.

| Key | Action |
|-----|--------|
| `d` | Delete (opens confirmation dialog) |
| `X` | Prune unused volumes |

## Images

The image list shows all images with their usage status.

| Key | Action |
|-----|--------|
| `c` | View image details |
| `d` | Delete (opens confirmation dialog) |
| `X` | Prune dangling images |
| `P` | Pull images for the selected stack (shows a `docker pull`-style progress bar with downloaded/total bytes) |

## Networks

The network list shows name, driver, scope, subnet, and connected container count.

| Key | Action |
|-----|--------|
| `Enter` / `c` | View network details (driver, scope, subnet, gateway, connected containers) |
| `d` | Delete the selected network (opens confirmation dialog; predefined networks and networks with connected containers are refused) |
| `X` | Prune unused networks |
| `/` | Filter networks by name |
| `r` | Refresh |

Predefined networks (`bridge`, `host`, `none`) are protected and cannot be deleted.

### Network details

| Key | Action |
|-----|--------|
| `j` / `k` | Select a connected container |
| `d` | Disconnect the selected container from the network (opens confirmation dialog) |
| `r` | Refresh |
| `Esc` / `q` | Back to the network list |
