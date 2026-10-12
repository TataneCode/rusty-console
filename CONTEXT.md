# Contexte général — rusty-console

Ce fichier est le **contexte projet de référence** pour tout agent IA travaillant sur ce dépôt.
Les conventions de code détaillées vivent dans [AGENTS.md](AGENTS.md) ; les règles par couche dans [rules/](rules/) ; les procédures opérationnelles dans [.agents/skills/](.agents/skills/).

## Qu'est-ce que rusty-console

TUI Rust pour gérer Docker : conteneurs, volumes, images et stacks.
Portage de **FancyContainerConsole** (C#/.NET) vers Rust.

- Stack technique : `ratatui` + `crossterm` (rendu TUI), `bollard` (API Docker, async), `tokio` (runtime async), `thiserror` (erreurs).
- Édition 2021, **MSRV 1.88** (déclarée via `rust-version` dans `Cargo.toml`).
- Le binaire nécessite un démon Docker accessible à l'exécution (les tests n'en dépendent pas : domain pur, application mockée).

## Architecture — DDD 4 couches, flux de dépendances strict

```
04_presentation (ui)  →  02_application  →  01_domain  ←  03_infrastructure
```

**Aucune couche externe ne peut être importée par une couche interne.**

| Couche | Dossier | Responsabilité |
|---|---|---|
| Domain | `src/01_domain/` | Entités (`Container`, `Volume`, `Image`, `Stack`), value objects, erreurs. Rust pur, **zéro crate externe**. |
| Application | `src/02_application/` | Traits repository (boundary d'inversion de contrôle), services stateless, DTOs, mappers domain→DTO. |
| Infrastructure | `src/03_infrastructure/` | Adapters Bollard implémentant les traits application ; mappers Docker API→domain. |
| Presentation | `src/04_presentation/` | Rendu Ratatui ; triade `presenter.rs` / `view.rs` / `actions.rs` par domaine. |

Wiring DI manuel dans `main.rs` : `DockerClient → Arc<*Adapter> → *Service → *Actions → App::new(...)`.

## État du projet (octobre 2026)

- Coverage ≈ 91 % (minimum requis : 70 %), 297+ tests.
- CI stricte : `cargo fmt`, `clippy -D warnings`, tests, MSRV.
- Refactors majeurs déjà réalisés : splitting de `app.rs`, déplacement des DTO vers leurs couches, réduction des features tokio.
- Dernière fonctionnalité : navigation dans les logs, vue étendue, recherche vi, export `.log` (PR #19).

## Cartographie des ressources

- [AGENTS.md](AGENTS.md) — conventions du code (point d'entrée des agents).
- [rules/](rules/) — règles par couche, mutualisées entre agents et skills.
- [.agents/skills/](.agents/skills/) — 8 procédures réutilisables (standard Agent Skills, découvertes par Vibe Code et Vibe CLI).
- [.vibe/agents/](.vibe/agents/) — agents Vibe CLI : `architect`, `developer`, `reviewer`.
- [agents/](agents/) — définitions de rôles agnostiques (Markdown), source des agents CLI.
- [AGENT_ARCHITECTURE.md](AGENT_ARCHITECTURE.md) — orchestration agents/skills.
- [PLAN.md](PLAN.md), [EVOLUTION.md](EVOLUTION.md) — roadmap et historique des fonctionnalités.
- [docs/](docs/) — plans et comptes-rendus de refactoring.
- [xx_learning/](xx_learning/) — parcours d'apprentissage Rust du projet.
