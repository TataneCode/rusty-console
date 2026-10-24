# Règles qualité — transverses (toutes couches)

Commandes de vérification, **dans cet ordre**, avant toute livraison :

```bash
cargo fmt                  # formate — aucun diff ne doit rester
cargo clippy -- -D warnings  # lint strict : la CI traite les warnings en erreurs
cargo test                 # tous les tests (297+)
cargo check                # compilation rapide (MSRV 1.88)
cargo build --release      # seulement si la livraison concerne le binaire
```

Toute livraison doit passer `fmt` + `clippy -D warnings` + `test` sans erreur.

## Style global

- Édition 2021, **MSRV 1.88** (`rust-version` dans `Cargo.toml`). Pas de feature nightly, pas de crate exigeant un toolchain plus récent.
- **Erreurs : toujours `thiserror`** pour les erreurs de couches ; jamais `String` ni `anyhow` dans les erreurs attendues des couches. `anyhow` réservé au bord de `main.rs`.
- Chaîne d'erreurs à 3 niveaux : `DomainError` → `AppError` (`#[from]`) → `InfraError` (conversion bollard) → `AppError` pour l'UI.
- **Async** : tout l'I/O via `tokio` ; traits async avec `#[async_trait]` ; features tokio limitées à `rt-multi-thread, macros, sync, time, process` — ne pas réactiver `full`.
- **Sécurité mémoire** : éviter les `clone()` superflus, préférer les références ; `Arc` uniquement au point d'injection DI.
- **Noms** : conventions Rust (snake_case fonctions, PascalCase types) ; renommer plutôt que d'ajouter `#[allow(clippy::...)]`.
- **Dépendances** : minimum nécessaire, features les plus étroites possibles (voir skill `rust-dependency`).
- **Tests** : couverture ≥ 70 % (courant : ~91 %) ; conventions UT/IT dans [testing.md](testing.md) (unit tests co-localisés dans `#[cfg(test)]`, IT dans `tests/`, `mockall` pour les traits, nom du test = comportement, ex. `rejects_zero_port_mapping`).
- **Diff minimal** : ne changer que ce qui est nécessaire ; matcher le style existant ; lire un fichier entièrement avant de le modifier.
