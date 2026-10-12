# Règles — Couche Application (`src/02_application/`)

## Périmètre

Traits repository (boundary d'inversion de contrôle), services stateless, DTOs, mappers domain→DTO.

## Règles

1. **Traits repository dans `*/traits.rs`** — tous avec `#[async_trait]`, bounds `Send + Sync`, chaque méthode retourne `Result<T, AppError>`.
2. **Inversion de contrôle** — l'application définit les ports (traits) ; l'infrastructure les implémente. L'application n'importe jamais `03_infrastructure`.
3. **Erreurs : `AppError`** avec `thiserror` — wrappe `DomainError` via `#[from]`, ajoute les variantes `Repository`, `NotFound`, `Connection` avec méthodes builder.
4. **Services stateless** — pas d'état mutable ; les services reçoivent `Arc<dyn Trait>` en dépendance.
5. **DTOs distincts des entités** — mappers application (`unit structs`, fonctions associées, pas de `self`) convertissent domain→DTO. L'UI ne voit que des DTO.
6. **Mappers stateless** — unit structs avec fonctions associées ; jamais d'état.
7. **Tout l'I/O est async** via `tokio` ; jamais d'appel bloquant dans un service.
8. **Tests avec `mockall`** — `Mock*Repository` pour mocker les traits ; tests `#[tokio::test]` co-localisés dans les fichiers de service.

## Anti-patterns interdits

- Importer `03_infrastructure`, `bollard`, ou un type Docker quelconque.
- Retourner une entité du domain directement à l'UI (toujours un DTO).
- Logique de rendu ou de gestion de touches clavier (c'est le rôle de la presentation).
- État mutable dans un service (utiliser un presenter côté UI à la place).
