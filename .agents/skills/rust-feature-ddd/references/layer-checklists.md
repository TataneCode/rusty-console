# Référence — Checklists d'implémentation par couche

Détail de `rust-feature-ddd` (étapes 1 à 5). Chargé à la demande, couche par couche. Les règles normatives complètes vivent dans `rules/` (source de vérité) — ce fichier n'est qu'un aide-mémoire opérationnel.

## 1. Domain (`src/01_domain/<area>/`)

- Entité / value object en Rust pur : **zéro crate externe** (thiserror toléré pour les erreurs).
- Value objects : champs privés + `new() -> Result<Self, DomainError>` (jamais de constructeur infallible pour une donnée validée).
- Invariants dans le type, pas dans les couches supérieures.
- Erreurs : variantes dans `src/01_domain/error.rs` avec `#[thiserror::error]`.
- Tests unitaires dans `#[cfg(test)]` au sein même du fichier.

## 2. Application (`src/02_application/<area>/`)

- Trait repository si nouvelle capacité : `#[async_trait]`, retour `Result<T, AppError>`, `#[cfg_attr(test, mockall::automock)]`.
- Service stateless : `pub struct XxxService { repo: Arc<dyn XxxRepository> }`.
- DTO dédié + mapper `domain ↔ DTO` dans `mapper.rs` (unit struct, fonctions associées). Jamais d'entité domain exposée à l'UI.
- Erreurs : étendre `AppError` dans `src/02_application/error.rs`.

## 3. Infrastructure (`src/03_infrastructure/docker/<area>/`)

- Adapter implémentant le trait application, conversion des types bollard en entités domaine via un mapper dédié.
- `bollard::errors::Error` → `InfraError` → `AppError` via `From`.
- Spécificités Bollard 0.18 : `df()` sans argument, `Port.typ`, `Port.private_port: u16`.

## 4. Presentation (`src/04_presentation/tui/<area>/`)

- Triade **presenter / view / actions** :
  - `presenter.rs` — état mutable (sélection, offset de scroll, données chargées)
  - `view.rs` — fonctions de rendu pures (références → widgets)
  - `actions.rs` — wrappers async minces autour des services
- Étendre `Screen` dans `ui/app.rs` si nouvel écran ; overlays (`confirm_dialog`, `error_message`) traités avant les handlers spécifiques.
- Touches mappées dans `ui/common/keys.rs` → variantes `AppAction`.
- L'UI catch `AppError` et l'affiche via `App::error_message`.

## 5. Wiring (`src/main.rs` et `src/lib.rs`)

- Câblage manuel ascendant, sans framework DI :
  `DockerClient → Arc<XxxAdapter> → XxxService → XxxActions → App::new(...)`
- Exposer les modules dans les `mod.rs` / `lib.rs` correspondants.
