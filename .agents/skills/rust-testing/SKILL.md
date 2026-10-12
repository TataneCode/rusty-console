---
name: rust-testing
description: Écrire les tests Rust de rusty-console — tests unitaires co-localisés, mocks mockall pour les traits repository, tests async avec tokio. Utiliser ce skill dès qu'une tâche demande des tests ou que du code testable est ajouté.
---

# Skill : Tests Rust

## Contexte projet

- Tests unitaires dans des modules `#[cfg(test)]` **au sein de chaque fichier** (pas de répertoire `tests/` parallèle pour les tests unitaires).
- `mockall = "0.14"` en dev-dependency pour mocker les traits repository.
- `tokio-test = "0.4"` pour les helpers de tests async.
- `#[async_trait]` sur les traits application → utiliser `#[tokio::test]` pour les services async.

## Procédure systématique

### 1. Domain (`01_domain`)
- Tester les invariants : construction valide, rejet des entrées invalides (`new() → Err(DomainError::...)`).
- Aucun mock nécessaire : le domaine est pur.

### 2. Application (`02_application`)
- Ajouter `#[cfg_attr(test, mockall::automock)]` sur le trait repository s'il n'y est pas déjà.
- Mocker les attentes : `mock.expect_xxx().with(eq(...)).returning(|_| Ok(...))`.
- Tester les cas : succès, erreur repository propagée, mappage domaine → DTO correct.
- Tests async : `#[tokio::test]` autour des méthodes de service.

### 3. Infrastructure (`03_infrastructure`)
- Pas de tests contre un vrai démon Docker dans la suite standard. Tester les mappers bollard → domaine avec des fixtures de types bollard construits à la main.

### 4. Presentation (`04_presentation`)
- Tester les fonctions de rendu pur de `view.rs` (elles prennent des références et retournent des widgets — testables sans terminal).
- Tester la logique de `presenter.rs` (sélection, scroll) sans mock lourd.

### 5. Conventions
- Un test par comportement, nom descriptif : `rejects_empty_container_id`, pas `test_1`.
- Arrange / Act / Assert, une assertion logique par test (plusieurs `assert!` liés admis).
- Si un bug est corrigé : écrire d'abord le test qui échoue (reproduction), puis corriger.

## Vérification

```bash
cargo test            # tout
cargo test <name>     # un test ciblé
cargo llvm-cov        # couverture si cargo-llvm-cov installé
```

## Anti-patterns interdits

- Mocker ce qu'on ne possède pas (types bollard entiers) — mapper à la main dans des fixtures.
- Tests dépendant d'un environnement externe (démon Docker, réseau).
- Ignorer un test avec `#[ignore]` sans justification écrite.
