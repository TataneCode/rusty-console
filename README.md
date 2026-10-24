# Rusty Console

A terminal user interface (TUI) for managing Docker containers, volumes, images, networks, and stacks — written in Rust. This is a port of FancyContainerConsole (originally in C#/.NET) to Rust.

## Features

- **Containers** — list all containers regardless of state, with live CPU and memory usage; start/stop, pause/unpause, restart, interactive exec shell, real-time logs, inspect details, delete, prune, filter by name, sort by name
- **Create containers** — press `A` on the Containers screen to create and start a container from a 3-field dialog (image pulled automatically if missing)
- **Volumes** — list, detect in-use volumes, delete, prune, filter by name
- **Networks** — list with subnet and connected container count, inspect details, disconnect containers, delete, prune, filter by name
- **Images** — list with usage status, inspect details, delete, prune dangling, filter by name
- **Stacks** — detect Docker Compose stacks, start/stop/remove all containers at once, and drill down into a stack's containers with the full container feature set (logs, details, exec, live stats, filter, sort)
- Confirmation dialogs for all destructive actions, error popups, loading spinner for async operations
- Breadcrumb navigation bar and homogeneous section design across the whole UI

## Quick start

```bash
cargo run
```

Requires a Rust toolchain and a running Docker daemon. See the [installation guide](./INSTALL.md) for details and prebuilt installers.

## Documentation

| Document | Content |
|----------|---------|
| [TECHNOLOGIES.md](./TECHNOLOGIES.md) | Crates used in the project and why they were chosen |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | DDD layered architecture, graphs, data flow, and the architecture decision log |
| [INSTALL.md](./INSTALL.md) | Installation guide (prebuilt installers, building from source) |
| [MANUAL.md](./MANUAL.md) | User manual and key bindings for every screen |
| [docs/TESTING.md](./docs/TESTING.md) | Test suites, commands, and the scenario coverage matrix |

Additional resources:

- [xx_learning/](./xx_learning/) — a project-based Rust learning path built around this codebase, from ownership basics to the full application architecture
- [CONTEXT.md](./CONTEXT.md) — project map for AI agents (navigation, state, key commands); [AGENTS.md](./AGENTS.md) — agent entry point (confirmation gate, routing table)
- [AGENT_ARCHITECTURE.md](./AGENT_ARCHITECTURE.md) — how the agents, skills and rules articulate
- [.legacy/](./.legacy/) — historical documents (roadmap, evolution history, architecture logs)

## License

This project is licensed under the [MIT License](./LICENSE).
