# Règles de test — UT & IT (transverse, toutes couches)

**Source unique de vérité** pour les conventions de test. La matrice des scénarios user-facing (quoi est couvert où) vit dans [docs/TESTING.md](../docs/TESTING.md). Le skill opérationnel est `rust-testing` ([.agents/skills/rust-testing/](../.agents/skills/rust-testing/)).

## Seuil de décision : UT ou IT ?

| Situation | Où | Pourquoi |
|---|---|---|
| Logique pure (invariant, mapper, service avec mock, presenter, fonction de vue) | **UT** — `#[cfg(test)]` co-localisé dans le fichier | Rapide, isolé, pas de terminal ni de démon |
| Comportement user-facing traversant les couches (écran × touche × état) | **IT** — scénario dans `tests/` | Valide le wiring réel, pas une copie |
| Bugfix de logique métier | **UT** de régression d'abord (rouge), puis correction | La régression se fixe au niveau de la couche fautive |
| Nouveau comportement visible à l'écran | **IT** + entrée dans la matrice de `docs/TESTING.md` | Garantit la couverture du contrat utilisateur |

Règle : **1 comportement = 1 test**, nommé par le comportement (`rejects_zero_port_mapping`, pas `test_1`). Arrange / Act / Assert.

## UT — conventions

- Co-localisés dans `#[cfg(test)]` **dans chaque fichier** — pas de répertoire `tests/` parallèle pour les tests unitaires.
- **Domain** : pas de mock (couche pure) — tester les invariants, les rejets de `new()`.
- **Application** : `#[cfg_attr(test, mockall::automock)]` sur le trait repository ; attentes explicites (`mock.expect_xxx().with(eq(...)).returning(...)`) ; `#[tokio::test]` pour les services async (`tokio-test` disponible en dev-dependency pour les helpers).
- **Infrastructure** : tester les mappers bollard→domaine avec des fixtures construites à la main. **Jamais de test contre un vrai démon Docker** dans la suite standard.
- **Presentation** : tester les fonctions pures de `view.rs` (références → widgets) et la logique de `presenter.rs` (sélection, scroll) sans mock lourd.
- Si un bug est corrigé : écrire d'abord le test qui échoue (reproduction), puis corriger.

## IT — conventions

- Build du vrai `App` via `tests/common/mod.rs::build_app()` — vrais adapters, client bollard **lazy** : rendu et gestion d'actions ne nécessitent **pas** de démon Docker. Les suites liées au démon réel retournent tôt s'il est indisponible.
- Pilotage : `app.handle_action(AppAction::…)` et `app.handle_filter_key(KeyCode::…)` (rendues publiques à cette fin).
- Rendu : `ratatui::backend::TestBackend`, assertions sur le buffer aplati.
- **Toute nouvelle fonctionnalité user-facing** ajoute un scénario à la matrice de [docs/TESTING.md](../docs/TESTING.md).

## Vérification

```bash
cargo test                 # tout (UT + IT)
cargo test --lib           # UT seuls
cargo test --test <suite>  # une suite IT seule
cargo test <name>          # un test ciblé
cargo llvm-cov             # couverture (min 70 %, courant ~91 %)
```

## Anti-patterns interdits

- Mocker ce qu'on ne possède pas (types bollard entiers) — mapper à la main dans des fixtures.
- Tests dépendant d'un environnement externe (démon Docker, réseau) dans la suite standard.
- `#[ignore]` sans justification écrite.
- Modifier un test pour « faire passer » un refactor (un refactor ne change pas les tests).
