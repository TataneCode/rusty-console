---
name: rust-feature-ddd
description: Implémenter une nouvelle fonctionnalité dans rusty-console en respectant l'architecture Domain-Driven Design à 4 couches (domain → application → infrastructure → presentation). Utiliser ce skill dès qu'une tâche demande d'ajouter un comportement, un écran, une entité ou une opération Docker.
---

# Skill : Implémentation DDD d'une fonctionnalité Rust

## Contexte projet

rusty-console est un TUI Rust (ratatui + crossterm + bollard + tokio) pour gérer conteneurs, volumes, images et stacks Docker.

Flux de dépendances **strictement inward** — aucune couche externe ne peut être importée par une couche interne :

```
ui (04_presentation) → application (02_application) → domain (01_domain) ← infrastructure (03_infrastructure)
```

Chemins réels du projet : `src/01_domain/`, `src/02_application/`, `src/03_infrastructure/docker/`, `src/04_presentation/tui/`.

> Règles détaillées par couche : charger `rules/domain.md`, `rules/application.md`, `rules/infrastructure.md`, `rules/presentation.md` à la racine du dépôt (source de vérité mutualisée).

## Procédure systématique — 5 étapes

### 1. Domain (`src/01_domain/<area>/`)
- Créer l'entité / value object en Rust pur : **zéro crate externe**.
- Value objects : structs avec champs privés + `new() -> Result<Self, DomainError>` (jamais de constructeur infallible pour une donnée validée).
- Mettre les invariants de validation dans le type, pas dans les couches supérieures.
- Erreurs : variantes dans `src/01_domain/error.rs` avec `#[thiserror::error]` (thiserror est la seule dépendance autorisée du domaine).
- Tests unitaires dans `#[cfg(test)]` au sein même du fichier.

### 2. Application (`src/02_application/<area>/`)
- Définir le trait repository si nouvelle capacité : `#[async_trait]`, retour `Result<T, AppError>`, `#[cfg_attr(test, mockall::automock)]` sur le trait.
- Service stateless : `pub struct XxxService { repo: Arc<dyn XxxRepository> }` avec `Arc<dyn Trait>` injecté.
- DTO dédiés + mapper `domain ↔ DTO` dans un module `mapper.rs`. Ne jamais exposer une entité du domaine à l'UI.
- Erreurs : étendre `AppError` dans `src/02_application/error.rs` (variantes `Repository`, `NotFound`, `Connection`, `#[from] DomainError`, méthodes builder).

### 3. Infrastructure (`src/03_infrastructure/docker/<area>/`)
- Adapter implémentant le trait application, convertissant les types bollard en entités domaine via un mapper dédié.
- Convertir `bollard::errors::Error` en `InfraError`, puis `InfraError → AppError` via `From`.
- Considérer les particularités de l'API bollard (ex. `df()` sans argument, `Port.typ`, `Port.private_port: u16`).

### 4. Presentation (`src/04_presentation/tui/<area>/`)
- Respecter le triade **presenter / view / actions** :
  - `presenter.rs` — état mutable (sélection, offset de scroll, données chargées)
  - `view.rs` — fonctions de rendu pures (références → widgets)
  - `actions.rs` — wrappers async minces autour des services
- Étendre `Screen` dans `ui/app.rs` si nouvel écran ; overlays (`confirm_dialog`, `error_message`) traités avant les handlers spécifiques.
- Mapper les touches dans `ui/common/keys.rs` → variantes `AppAction`.
- L'UI catch `AppError` et l'affiche via `App::error_message`.

### 5. Wiring (`src/main.rs` et `src/lib.rs`)
- Câblage manuel ascendant, sans framework DI :
  `DockerClient → Arc<XxxAdapter> → XxxService → XxxActions → App::new(...)`
- Exposer les modules dans `mod.rs` / `lib.rs` correspondants.

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
- Exposer une entité domain à l'UI (touiser un DTO).
- Logique métier dans les adapters ou les actions UI.
- Nouveau constructeur qui bypass la validation (`pub fn new` infallible sur un value object validé).
