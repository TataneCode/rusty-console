---
name: rust-quality-gate
description: Contrôle qualité Rust — formatage, clippy strict, MSRV 1.88, style du code rusty-console. Utiliser ce skill avant toute livraison de code, sur échec de CI, ou quand on demande un review qualité.
---

# Skill : Contrôle qualité Rust

## Commandes de vérification (dans cet ordre)

```bash
cargo fmt                  # formate — aucun diff ne doit rester
cargo clippy -- -D warnings  # lint strict : la CI traite les warnings en erreurs
cargo test                 # tous les tests
cargo check                # compilation rapide (MSRV 1.88)
cargo build --release      # seulement si la livraison concerne le binaire
```

Toute livraison doit passer `fmt` + `clippy -D warnings` + `test` sans erreur.

## Règles de style du projet

> Source de vérité mutualisée : `rules/quality.md` et les règles par couche (`rules/*.md`) à la racine du dépôt. Cette section en est le résumé opérationnel.


- Édition 2021, MSRV **1.88** (déclaré dans `Cargo.toml` via `rust-version`). Pas de feature nightly ni de crates exigeant un toolchain plus récent.
- Gestion d'erreurs : **toujours `thiserror`** pour les erreurs de couches (jamais de `String` ni `anyhow` dans les erreurs attendues des couches). `anyhow` réservé au bord de `main.rs`.
- Chaîne d'erreurs à 3 niveaux : `DomainError` → `AppError` (`#[from]`) → `InfraError` (conversion bollard) → `AppError` pour l'UI.
- Async : tout l'I/O via `tokio` ; traits async avec `#[async_trait]` ; features tokio limitées à `rt-multi-thread, macros, sync, time, process` — ne pas réactiver `full`.
- Sécurité mémoire : éviter `clone()` superflus, préférer les références ; `Arc` uniquement au point d'injection DI.
- Noms : suivre les conventions Rust (snake_case fonctions, PascalCase types) ; renommer plutôt que d'ajouter `#[allow(clippy::...)]`.
- Dépendances : minimum nécessaire, features les plus étroites possibles.

## Échec de CI — diagnostic

1. Lire le premier message d'erreur du haut (`cargo clippy` ou `cargo test`), pas les erreurs en cascade.
2. Reproduire localement avec la commande exacte de la CI.
3. Correr la cause démontrée, pas un symptôme deviné.
4. Re-vérifier la chaîne complète avant de pousser.
