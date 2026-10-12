---
name: rust-bugfix
description: Corriger un bug dans rusty-console — reproduction, localisation de la cause racine dans la couche fautive, test de régression. Utiliser ce skill dès qu'un comportement incorrect est signalé ou qu'un test échoue.
---

# Skill : Correction de bug Rust

## Procédure systématique — 5 étapes

### 1. Reproduire
- Identifier la commande minimale qui déclenche le bug (`cargo test <name>`, scénario TUI précis).
- Si le bug dépend du démon Docker, isoler d'abord la couche fautive via les tests mappés (domain pur, application mockée) avant de conclure « infrastructure ».
- Si impossible à reproduire, demander les étapes exactes plutôt que deviner.

### 2. Localiser la couche fautive (grâce à l'architecture DDD)
| Symptôme | Couche probable |
|---|---|
| Invariant violé, donnée invalide acceptée | `01_domain` |
| Mauvais DTO, logique de service incorrecte | `02_application` |
| Erreur bollard mal convertie, mapping Docker faux | `03_infrastructure` |
| Rendu, sélection, touches, état d'écran | `04_presentation` |

La correction se fait **dans la couche fautive**, jamais en compensant dans une couche voisine.

### 3. Écrire le test de régression d'abord
- Voir le skill `rust-testing` : test co-localisé dans le fichier de la couche, nom décrivant le bug (`rejects_zero_port_mapping`).
- Vérifier qu'il échoue avant la correction (rouge), sinon il ne teste rien.

### 4. Corriger au plus petit périmètre
- Changement minimal dans la couche identifiée.
- Ne pas « améliorer » du code adjacent pendant un bugfix.
- Si la correction exige un refactor, le découpler : bugfix d'abord, refactor ensuite.

### 5. Vérifier la chaîne complète

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test
```

## Anti-patterns interdits

- Corriger le symptôme dans l'UI alors que la cause est en dessous (ex. catcher une erreur plus tôt pour la masquer).
- Ajouter un `unwrap()`/`expect()` de convenance pour « débloquer ».
- Pousser sans test de régression pour un bug de logique métier.
