---
name: rust-dependency
description: Ajouter ou mettre à jour une dépendance Cargo dans rusty-console — choix de version compatible MSRV, features minimales, vérification Cargo.lock. Utiliser ce skill dès qu'une tâche touche Cargo.toml ou introduit un crate.
---

# Skill : Gestion des dépendances Cargo

## Procédure systématique — 5 étapes

### 1. Justifier la dépendance
- Répondre avant d'ajouter : quel problème ne peut pas être résolu avec la std ou les crates existantes ?
- Vérifier si un crate déjà présent couvre le besoin (ex. `chrono` pour le temps, `serde` pour la sérialisation).

### 2. Contraintes du projet
- MSRV **1.88** : vérifier que le crate (et sa version) supporte ce toolchain. `rust-version` du crate ≤ 1.88.
- Édition 2021.
- Le domaine (`01_domain`) reste à **zéro crate externe** (thiserror seul toléré pour les erreurs). Jamais de nouvelle dépendance utilisée par le domaine.

### 3. Déclarer avec les features minimales
- Lister explicitement les features (ex. tokio limité à `rt-multi-thread, macros, sync, time, process` — pas de `full`).
- Version contrainte mineure (`"0.30"`, pas `"*"` ni `^` large inutile).
- Dev-dependencies séparées (`mockall`, `tokio-test`).

### 4. Verrouiller et vérifier
```bash
cargo update -p <crate>       # ou ajout manuel + cargo build
cargo check                   # résolution + compilation MSRV
cargo clippy -- -D warnings
cargo test
```
- `Cargo.lock` est commité : l'inclure dans le commit.
- Vérifier qu'aucune feature d'un crate existant n'a été élargie au passage (`cargo tree -e features` si doute).

### 5. Documenter dans Cargo.toml
- Regrouper par rôle avec commentaires, comme l'existant (`# TUI Framework`, `# Docker API`, `# Async Runtime`, `# Error Handling`...).

## Anti-patterns interdits

- Ajouter un crate « au cas où » ou pour une seule fonction triviale implémentable en quelques lignes.
- Élargir des features (`default-features = true` implicite + features larges) sans nécessité.
- Mettre à jour des dépendances non liées à la tâche dans le même commit.
- Introduire une dépendance qui force le domaine ou relève le MSRV.
