# Testing

How this project is tested: what runs where, and which user-facing scenario each suite covers.

## Commands

```bash
cargo test                              # everything (lib + integration)
cargo test --lib                         # unit tests only
cargo test --test stack_containers_ui   # one integration suite
cargo test <name>                       # one test by name
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Minimum coverage: 70% (enforced by CI via `cargo llvm-cov`).

## Suites

### Unit tests (`src/…`, `#[cfg(test)]` modules)

One test module per source file, next to the code. `mockall` generates repository mocks (`MockContainerRepository`, …), `tokio::test` drives async code. UI logic is tested through presenters (state) and `TestBackend` renders (view), without a Docker daemon.

### Integration tests (`tests/`)

End-to-end-ish tests that build the real `App` with the real adapters wired exactly like `main.rs` (`tests/common/mod.rs::build_app`). The bollard client connects lazily, so rendering and pure action handling need **no Docker daemon**; the suites return early if the Docker client cannot be constructed.

| Suite | What it covers |
|---|---|
| `tests/stack_containers_ui.rs` | Homogeneity of the two container screens: Stacks > Containers has the same 7-column layout and the same container actions as Containers (logs, details, exec, start/stop, delete, pause, restart, prune, add, filter, sort, loading spinner); stack-level actions (Start All, Stop All, Remove All) live only on the stack list; filter shows the contextual help bar and narrows rows; `Esc` clears the filter; sort cycles A→Z → Z→A → default |
| `tests/action_bar_ui.rs` | Action bar content and key highlighting on list screens and the main menu |
| `tests/breadcrumb_ui.rs` | Breadcrumb path on the main menu, container logs and container list screens |
| `tests/integration_docker.rs` | Real Docker daemon: pull images, create/start/stop/delete containers, inspect — skipped when no daemon is available |

## Scenario matrix (user-facing behavior)

| Scenario | Unit tests | Integration UI |
|---|---|---|
| Container list: filter / sort / selection bounds | `container/presenter.rs`, `common/list.rs` | — |
| Container list: start/stop, pause, restart, delete, prune, add | `app.rs` (confirm-action paths) | — |
| Logs: navigation, search, export | `container/presenter.rs`, `container/view.rs` | `breadcrumb_ui.rs` (breadcrumb) |
| Logs: selected line highlighted, `y` copies it to clipboard (popup feedback) | `container/view.rs` (highlight style), `common/keys.rs` (`y` context), `app.rs` (copy handler), `container/dto.rs` (`display_line`) | `logs_copy_ui.rs` |
| Live CPU/memory stats streaming | `app.rs` (`drain_container_stats`), `container/presenter.rs` | — |
| Stack list: drill-down, start/stop/remove all, filter, sort | `stack/presenter.rs`, `app.rs` | `stack_containers_ui.rs` |
| Stack containers == container list features | `stack/presenter.rs`, `stack/view.rs`, `app.rs` | `stack_containers_ui.rs` (7 tests) |
| Logs/details return to their origin screen | `app.rs` | `stack_containers_ui.rs` |
| Exec shell (dialog, external terminal, inline fallback) | `exec.rs`, `app.rs` | — |
| Image pull with progress bar, up-to-date skip | `app.rs` (pull workflow) | `integration_docker.rs` |
| Real daemon end-to-end (create, run, stop, remove) | — | `integration_docker.rs` |

## Conventions

Les conventions UT/IT (où placer un test, seuil de décision UT vs IT, anti-patterns) sont la source de vérité dans [rules/testing.md](../rules/testing.md) ; le skill opérationnel est `rust-testing`.

- New behavior gets a unit test in the module that owns it; new cross-layer behavior (screen × keys × state) gets a scenario in `tests/`.
- View tests render with `ratatui::backend::TestBackend` and assert on the flattened buffer text.
- Repository boundaries are mocked with `mockall`; expectations state exact call counts where the scenario depends on them (e.g. stats subscription refreshes).
- Doc references: user-facing keys live in [MANUAL.md](../MANUAL.md); the plan and scenario history for a feature lives in [docs/plans/](plans/).
