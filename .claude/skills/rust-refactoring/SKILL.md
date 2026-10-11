---
name: rust-refactoring
description: Refactoriser du code rusty-console (découpage de gros fichiers, déplacement de types, renommages) en préservant le comportement et l'architecture DDD. Utiliser ce skill pour toute tâche de refactorisation, extraction de module ou nettoyage de code.
---

# Skill : Refactorisation Rust

## Contexte projet

Architecture DDD 4 couches (`01_domain` → `02_application` → `03_infrastructure` → `04_presentation`), MSRV 1.88, CI stricte (`clippy -D warnings`). Historique du repo : refactors déjà réalisés = splitting de `app.rs` (exec/confirm/router), déplacement de `PruneResultDto`/`ByteSize` vers leurs couches, réduction des features tokio. S'inspirer de `docs/refactoring/code_quality_refactoring_2026-10-11.md` pour le style de documentation d'un refactor.

## Procédure systématique — 6 étapes

1. **Établir la baseline** : `cargo test && cargo clippy -- -D warnings` avant tout changement. Si ce n'est pas vert, arrêter et le signaler.
2. **Cartographier les usages** : `grep -rn "<symbole>" src/` pour tout symbole déplacé. Lister tous les sites d'appel avant de toucher au code.
3. **Petits pas atomiques** : un refactor = un commit. Ordre typique :
   - Extraction : créer le nouveau module → déplacer le code → réexporter temporairement si utile (`pub use`) → mettre à jour les `mod.rs` → supprimer les réexports temporaires.
   - Déplacement inter-couches : le type va dans la couche la plus interne qui le concerne (un DTO applicatif ne vit jamais dans `shared.rs` global).
   - Renommage : `cargo clippy` guide vers les `#[allow]` devenues inutiles à supprimer (ex. suppression de `allow(clippy::enum_variant_names)` après renommage des variants).
4. **Préserver les APIs publiques** : vérifier `src/lib.rs` — c'est la frontière publique ; tout ce qui n'est pas réexporté peut être renommé librement.
5. **Vérifier à chaque étape** : `cargo check` rapide après chaque déplacement, suite complète (`fmt, clippy -D warnings, test`) à la fin.
6. **Documenter** : si le refactor est significatif, ajouter une note dans `docs/refactoring/` datée et listant les fichiers avant/après.

## Vérification finale

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

Aucun test ne doit être modifié pour « faire passer » : si un test change, c'est un changement de comportement, pas un refactor.

## Anti-patterns interdits

- Mélanger refactor et changement fonctionnel dans le même commit.
- Supprimer ou affaiblir un test pour le rendre vert.
- Créer des fichiers « utils » fourre-tout ; placer les types dans leur couche DDD.
- Casser la règle de flux de dépendances lors d'un déplacement.
