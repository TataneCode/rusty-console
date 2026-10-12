# Plan : Correction des 3 findings importants de la revue networks management (PR #23)

> Statut : approuvé par l'utilisateur — produit par l'agent Architecte (skill `rust-architecture-planning`), exécution par l'agent Développeur.
> Baseline au moment du plan : 379 tests verts, `cargo clippy --all-targets -- -D warnings` propre, `cargo fmt --check` propre (branche `feature/networks_management`).

## Objectif

Corriger les trois findings 🟡 de la revue de la PR #23 (N+1 inspect séquentiel, champ `error` mort dans `NetworkPresenter`, message de prune trompeur « freed 0 B ») sans changer le comportement fonctionnel ni l'architecture.

## Décisions d'architecture

- **D1 : Paralléliser les inspect dans `NetworkAdapter::get_all()` avec `futures_util::future::join_all`** — `futures-util` est déjà une dépendance directe (`Cargo.toml`, utilisée dans `container/adapter.rs`), donc zéro nouvelle dépendance. Le fallback par réseau (inspect échoué → mapping list-only) est conservé tel quel : c'est un pattern du repo, pas un finding.
- **D2 : Retirer le champ `error` et les méthodes `set_error`/`clear_error` de `NetworkPresenter`** — les erreurs réseau transitent par `App::popup_message` (convention de toutes les autres areas) ; le champ n'est ni alimenté ni lu. Retrait plutôt que câblage, car le câblage dupliquerait le mécanisme popup existant.
- **D3 : Message de prune réseau dédié sans « freed » dans `resources.rs`** — `PruneResultDto.space_freed` reste `0` (l'API Docker ne renvoie pas de taille pour les réseaux ; le contrat `02_application` ne change pas), mais la présentation réseau n'affiche plus « freed 0 B ». Ajout de `prune_networks_result_message(deleted_count)` à côté de `prune_result_message` générique, plutôt que de rendre `freed` optionnel dans la signature existante (utilisée par 3 autres resources).
- **D4 : Les trois correctifs vivent dans `03_infrastructure` et `04_presentation` uniquement** — aucune couche domain/application n'est touchée ; le flux de dépendances DDD est inchangé.

## Étapes

1. **Paralléliser `get_all()`** — fichiers : `src/03_infrastructure/docker/network/adapter.rs` — vérification : `cargo check` puis `cargo test network` (les tests existants de l'adapter couvrent le fallback par mock, inchangés). Collecter en parallèle les `inspect(name)` par nom via `join_all`, en gardant le fallback « list-only mapping » quand l'inspect échoue. Ordre de sortie préservé (ordre de `list_networks`).
2. **Retirer le champ mort `error`** — fichiers : `src/04_presentation/tui/network/presenter.rs` — vérification : `cargo check` + `cargo test presenter::network` (supprimer avec la méthode le test associé s'il existe).
3. **Message de prune réseau sans « freed »** — fichiers : `src/04_presentation/tui/common/resources.rs` (nouvelle fonction + test), `src/04_presentation/tui/router.rs` (branche `ConfirmAction::NetworksPrune` utilise la nouvelle fonction) — vérification : `cargo test prune` (adapter le test existant de `app.rs` qui asserte le message prune réseau).
4. **Quality gate complet** — fichiers : aucun (contrôle) — vérification : `cargo test` (379+ tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, puis commit(s) sur `feature/networks_management` et push vers la PR #23.

## Risques et mitigations

- **Concurrence dans `get_all()`** : appels Docker parallèles via le même `DockerClient` — mitigé : bollard/tokio conçoivent le client pour l'usage concurrent (hyper sous le capot) ; passer à `futures_util::stream::iter(...).buffer_unordered(N)` si le nombre de réseaux devient très grand.
- **Régression UX message prune** : un test assertant l'ancien format — mitigé : l'étape 3 cite le test à mettre à jour (`test_execute_confirm_action_network_paths` dans `app.rs`).
- **Re-revue** : la PR est déjà relue — mitigé : correctifs poussés sur la même branche `feature/networks_management` ; le Reviewer ne re-vérifie que les points corrigés.

## Hors périmètre

- Les 2 findings 🟢 (allow ciblé sur `double_must_use`, commentaire sur le scope par défaut) — non demandés.
- Toute évolution du contrat `PruneResultDto` ou du trait `NetworkRepository`.
- La parallélisation d'autres adapters (container/volume/image), même si le même N+1 pourrait y exister.
