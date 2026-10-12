# Règles — Couche Infrastructure (`src/03_infrastructure/`)

## Périmètre

Adapters Bollard implémentant les traits repository de l'application ; mappers Docker API→domain.

## Règles

1. **Adapters implémentent les traits application** — `#[async_trait]`, contre l'API async Bollard. C'est la seule couche qui connaît `bollard`.
2. **Erreurs : `InfraError`** avec `thiserror` — convertit `bollard::errors::Error`, puis se convertit en `AppError` via `From` pour l'UI.
3. **Mappers infra : `Option<Entity>` avec `filter_map`** — les réponses Docker API malformées sont **silencieusement ignorées** (skip), pas d'erreur.
4. **Mappers stateless** — unit structs avec fonctions associées, conversion Docker type → entité domain.
5. **`Arc<*Adapter>`** au point d'injection DI (`main.rs`) pour la propriété partagée entre tâches async.
6. **Bollard 0.18 spécifiques** — `df()` sans argument (`DataUsageOptions` supprimé) ; `Port.typ` (pas `Port.r#type`) ; `Port.private_port` est `u16` (pas `Option<u16>`).
7. **Tests** — mocker l'API Bollard si nécessaire, sinon tester les mappers en isolation avec des fixtures JSON réalistes.

## Anti-patterns interdits

- Exposer un type `bollard` en sortie d'adapter (le retour est toujours une entité domain ou un DTO via l'application).
- Contenir de la logique métier — un adapter traduit, il ne décide pas.
- Importer `04_presentation` ou toucher au rendu.
- Dépendre des features tokio non déclarées (`rt-multi-thread, macros, sync, time, process` uniquement — jamais `full`).
