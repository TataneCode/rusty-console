---
name: rust-testing
description: Écrire les tests Rust de rusty-console — UT co-localisés (mockall, tokio) ou IT dans tests/ (TestBackend, build_app). Utiliser ce skill dès qu'une tâche demande des tests ou que du code testable est ajouté.
---

# Skill : Tests Rust

> **Source de vérité des conventions** : [rules/testing.md](../../../rules/testing.md) (seuil UT vs IT, anti-patterns). Ce skill en est le résumé opérationnel. La matrice des scénarios IT vivants : [docs/TESTING.md](../../../docs/TESTING.md).

## Choisir : UT ou IT

| Situation | Où |
|---|---|
| Logique pure (invariant, mapper, service mocké, presenter, vue pure) | **UT** — `#[cfg(test)]` dans le fichier |
| Comportement user-facing (écran × touche × état) | **IT** — scénario dans `tests/` |
| Bugfix de logique métier | **UT** de régression d'abord (rouge), puis correction |
| Nouveau comportement visible à l'écran | **IT** + entrée dans la matrice de `docs/TESTING.md` |

## UT — par couche

### Domain (`01_domain`)
- Tester les invariants : construction valide, rejet des entrées invalides (`new() → Err(DomainError::...)`).
- Aucun mock : le domaine est pur.

### Application (`02_application`)
- `#[cfg_attr(test, mockall::automock)]` sur le trait repository s'il n'y est pas déjà.
- Attentes explicites : `mock.expect_xxx().with(eq(...)).returning(|_| Ok(...))`.
- Cas : succès, erreur repository propagée, mappage domaine → DTO correct.
- Async : `#[tokio::test]`.

### Infrastructure (`03_infrastructure`)
- Pas de test contre un vrai démon Docker dans la suite standard.
- Tester les mappers bollard → domaine avec des fixtures de types bollard construits à la main.

### Presentation (`04_presentation`)
- Tester les fonctions de rendu pur de `view.rs` (références → widgets, testables sans terminal).
- Tester la logique de `presenter.rs` (sélection, scroll) sans mock lourd.

## IT — `tests/`

- Build du vrai `App` via `tests/common/mod.rs::build_app()` (vrais adapters, client bollard lazy — **pas de démon requis** pour le rendu et les actions).
- Pilotage : `app.handle_action(AppAction::…)`, `app.handle_filter_key(KeyCode::…)` (publics à cette fin).
- Rendu : `ratatui::backend::TestBackend`, assertions sur le buffer aplati.
- Toute fonctionnalité user-facing nouvelle → scénario ajouté à la matrice de `docs/TESTING.md`.

## Conventions communes

- Un test par comportement, nom descriptif : `rejects_empty_container_id`, pas `test_1`.
- Arrange / Act / Assert, une assertion logique par test.
- Bugfix : test de reproduction d'abord (rouge), puis correction.

## Vérification

```bash
cargo test                 # tout (UT + IT)
cargo test --lib           # UT seuls
cargo test --test <suite>  # une suite IT seule
cargo test <name>          # un test ciblé
cargo llvm-cov             # couverture (min 70 %)
```

## Anti-patterns interdits

- Mocker ce qu'on ne possède pas (types bollard entiers) — mapper à la main dans des fixtures.
- Tests dépendant d'un environnement externe (démon Docker, réseau) dans la suite standard.
- `#[ignore]` sans justification écrite.
- Modifier un test pour « faire passer » un refactor.
