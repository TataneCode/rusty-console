# rusty-console — Project Map (CONTEXT.md)

**Point d'entrée unique.** Lis ce fichier en premier ; il te dit quoi lire ensuite — et quoi ne PAS lire. Règle d'or : avant de parcourir le code, consulte la map ci-dessous et limite ta lecture au périmètre de la tâche.

## En 30 secondes

TUI Rust pour gérer Docker : conteneurs, volumes, images, stacks. Portage de **FancyContainerConsole** (C#/.NET) vers Rust. Stack : `ratatui` + `crossterm` (TUI), `bollard` (API Docker async), `tokio`, `thiserror`. Édition 2021, **MSRV 1.88**. Le binaire a besoin d'un démon Docker à l'exécution ; les tests n'en ont pas besoin.

## Architecture — DDD 4 couches, flux strictement inward

```
04_presentation (ui)  →  02_application  →  01_domain  ←  03_infrastructure
```

Aucune couche externe ne peut importer une couche interne. L'UI ne voit jamais le domain : DTO + traits de service uniquement.

## Map de navigation — quoi lire, quand

| Tu veux… | Lis | Ne lis PAS |
|---|---|---|
| Comprendre le projet | Ce fichier | AGENTS.md en entier, tout src/ |
| Challenger / cadrer une demande | Skill `/challenge` | — |
| Planifier une feature, un refactor | `/plan` + `rules/README.md` | tout src/ avant le cadrage |
| Coder — couche domain | `rules/domain.md` + les fichiers du périmètre | les autres couches |
| Coder — couche application | `rules/application.md` + `traits.rs` du domaine | `03_infrastructure` |
| Coder — couche infrastructure | `rules/infrastructure.md` (spécificités Bollard 0.18) | `04_presentation` |
| Coder — UI | `rules/presentation.md` + la triade presenter/view/actions | `01_domain` directement |
| Écrire des tests | `rules/testing.md` + skill `rust-testing` | — |
| Livraison / qualité | `rules/quality.md` (5 commandes, dans l'ordre) | — |
| Reviewer un diff / une PR | `/self-review` puis `/review` + `rules/` | — |
| Auditer une zone du code | `/audit` + `rules/` | — |
| Faire évoluer cette config agent | `/update-config` | — |
| Connaître les scénarios de test IT existants | `docs/TESTING.md` (matrice) | — |
| Comprendre une décision d'architecture passée | `ARCHITECTURE.md`, `docs/refactoring/`, `docs/plans/` | `.legacy/PLAN.md`, `.legacy/EVOLUTION.md` (archive) |

## Où vivent les responsabilités

| Chemin | Rôle |
|---|---|
| `src/01_domain/` | Entités, value objects, invariants. Rust pur, zéro crate externe (thiserror toléré). |
| `src/02_application/` | Traits repository (ports), services stateless, DTOs, mappers domain→DTO. |
| `src/03_infrastructure/` | Adapters Bollard, mappers Docker API→domain. Seule couche connaissant `bollard`. |
| `src/04_presentation/` | Ratatui. Triade `presenter.rs` / `view.rs` / `actions.rs` par area (container, image, volume, stack) + `common/` (spinner, keys, list, filter, sort, action_bar, breadcrumb…). |
| `src/main.rs` | Wiring DI manuel : `DockerClient → Arc<*Adapter> → *Service → *Actions → App::new(...)`. |
| `tests/` | IT : `tests/common/mod.rs::build_app()` (vrais adapters, client bollard lazy — pas de démon requis), `TestBackend`. |
| `rules/` | **Source unique de vérité** des règles par couche + qualité + tests. |
| `.agents/skills/` | Procédures réutilisables (standard Agent Skills, auto-découvertes Vibe Code + Vibe CLI). Les skills `/…` sont user-invocables. |
| `agents/` | Définitions de rôles agnostiques (Markdown). |
| `.vibe/` | Wiring Vibe CLI : `agents/*.toml`, `prompts/*.md`, `config.toml` (modes). |
| `docs/` | `TESTING.md` (matrice des scénarios), `plans/`, `refactoring/`. |
| `.legacy/` | Archive historique (PLAN, EVOLUTION, logs). Aucune référence depuis les skills ni les agents. |

## Commandes essentielles

```bash
cargo fmt                  # formate
cargo clippy -- -D warnings  # lint strict (la CI traite les warnings en erreurs)
cargo test                 # tous les tests
cargo test <name>          # un test ciblé
cargo llvm-cov             # couverture (min 70 %, courant ~91 %)
```

Gate complet avant toute livraison : voir `rules/quality.md`.

## État connu (octobre 2026)

- Coverage ≈ 91 % (min requis : 70 %), ~518 tests (500 lib + 18 IT UI).
- CI stricte : `fmt`, `clippy -D warnings`, tests, MSRV.
- Dernier lot : homogénéisation Stacks > Containers avec l'écran Containers (PR #41), `ListCore<TItem>` partagé (ADR-002).

## Ressources

- [AGENTS.md](AGENTS.md) — routeur de démarrage (confirmation gate + table de routage).
- [rules/README.md](rules/README.md) — index des règles par couche.
- [ARCHITECTURE.md](ARCHITECTURE.md) — vision haut niveau (schéma, DI, data flow).
- [docs/TESTING.md](docs/TESTING.md) — suites et matrice des scénarios user-facing.
- [AGENT_ARCHITECTURE.md](AGENT_ARCHITECTURE.md) — orchestration agents/skills (détail).
- [xx_learning/](xx_learning/) — parcours d'apprentissage Rust du projet.
