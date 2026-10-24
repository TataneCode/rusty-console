---
name: rust-feature-ddd
description: Implémenter une nouvelle fonctionnalité dans rusty-console en respectant l'architecture DDD à 4 couches (domain → application → infrastructure → presentation → wiring). Utiliser dès qu'une tâche ajoute un comportement, un écran, une entité ou une opération Docker.
---

# Skill : Implémentation DDD d'une fonctionnalité Rust

## Contexte projet

rusty-console est un TUI Rust (ratatui + crossterm + bollard + tokio) pour gérer conteneurs, volumes, images et stacks Docker.

Flux de dépendances **strictement inward** :

```
ui (04_presentation) → application (02_application) → domain (01_domain) ← infrastructure (03_infrastructure)
```

> Règles détaillées par couche : charger `rules/domain.md`, `rules/application.md`, `rules/infrastructure.md`, `rules/presentation.md` à la racine du dépôt (source de vérité mutualisée — voir aussi le skill `layer-rules`).

## Procédure — 5 étapes

Ordre des couches : **domain → application → infrastructure → presentation → wiring**. Une étape = un commit potentiel ; le repo reste vert à chaque étape.

1. **Domain** (`src/01_domain/<area>/`)
2. **Application** (`src/02_application/<area>/`)
3. **Infrastructure** (`src/03_infrastructure/docker/<area>/`)
4. **Presentation** (`src/04_presentation/tui/<area>/`)
5. **Wiring** (`src/main.rs`, `src/lib.rs`)

Checklists détaillées de chaque étape : [references/layer-checklists.md](references/layer-checklists.md).

## Tests (obligatoire)

- Logique nouvelle → UT co-localisé ; comportement user-facing nouveau → scénario IT + matrice `docs/TESTING.md`. Voir [rules/testing.md](../../../rules/testing.md) et le skill `rust-testing`.

## Vérification (obligatoire avant livraison)

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo check
```

## Anti-patterns interdits

- Importer un crate dans `01_domain`.
- Utiliser un type bollard en dehors de `03_infrastructure`.
- Exposer une entité domain à l'UI (toujours un DTO).
- Logique métier dans les adapters ou les actions UI.
- Nouveau constructeur qui bypass la validation (`pub fn new` infallible sur un value object validé).
