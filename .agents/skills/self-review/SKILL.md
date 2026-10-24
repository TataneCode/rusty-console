---
name: self-review
description: Relire son propre diff avant de demander une review externe — checklist rules/ par couche, vérification des imports inter-couches, quality gate local. Utiliser systématiquement après /implement et avant /review ou une demande de review à un humain. Le filtre anti-bruit qui économise le plus de contexte.
user-invocable: true
---

# Skill : Self-review (relire son propre travail)

Rôle du **Développeur** ([agents/developer.md](../../../agents/developer.md)) : vérifier son propre diff **avant** de le soumettre. Fonctionne depuis tout point d'entrée (Vibe Code, Vibe CLI).

## Objectif

Filtrer le bruit avant une review externe : la plupart des findings d'une review sont détectables seul en 2 minutes. Un self-review systématique réduit le contexte consommé par `/review` et le nombre d'allers-retours.

## Procédure

1. **Lister le diff** : `git status` + `git diff` (unstaged) ou `git diff main...HEAD` (branche complète). Lire chaque fichier modifié **en entier**.
2. **Vérifier le périmètre** : le diff fait-il exactement ce que la tâche demandait ? Tout changement non lié est retiré ou signalé — jamais « au passage ».
3. **Checklist par couche** (règles de blocage de [rules/](../../../rules/)) :
   - Flux DDD : `grep -rn "use crate::"` sur les fichiers modifiés — aucun import de couche interne depuis l'externe.
   - Domain : zéro crate externe, invariants dans les types, pas d'I/O.
   - Application : services stateless, DTO dédiés, `AppError`, pas d'entité domain exposée.
   - Infrastructure : types bollard confinés, `InfraError → AppError`.
   - Presentation : triade presenter/view/actions, vues pures, touches dans `keys.rs`, `AppError` → `error_message`.
   - Transverse : pas de `unwrap()`/`expect()` ajouté hors tests sur chemin d'erreur ; pas de `#[allow]` ajouté pour contourner clippy (renommer/refactoriser à la place).
4. **Vérifier les tests** ([rules/testing.md](../../../rules/testing.md)) : logique nouvelle = UT co-localisé ; comportement user-facing nouveau = scénario IT + entrée dans la matrice de `docs/TESTING.md` ; bugfix = test de régression qui échouait avant la correction.
5. **Passer le quality gate local** ([rules/quality.md](../../../rules/quality.md), dans l'ordre) :
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   ```
   Tout rouge → corriger avant de continuer. **Ne jamais déclarer « ça marche » sans preuve exécutée.**
6. **Rédiger le bilan** : ce qui a été vérifié, ce qui reste ouvert (ex. test IT à ajouter, point à discuter en review).

## Quand s'arrêter

- Le gate est vert et la checklist est cochée → le diff est prêt pour `/review` ou une review humaine.
- Un point de la checklist révèle un doute structurel → corriger, ou le noter explicitement pour la review (ne pas le cacher).

## Anti-patterns interdits

- Self-review « de mémoire » sans relire le diff.
- `cargo test` vert mais `clippy` non passé (le gate est un tout).
- Corriger un finding de review en « commit rapide » sans repasser le self-review complet.
- Demander une review externe pour un diff qui n'a pas passé le gate local.
