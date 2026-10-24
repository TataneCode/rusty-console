---
name: audit
description: Analyser une zone du code produit de rusty-console — conformité aux règles par couche (rules/), dette technique, violations du flux DDD, qualité. Produit un rapport structuré. Utiliser pour auditer une area, préparer un refactor, ou évaluer l'état d'une partie du code. Lecture seule stricte.
user-invocable: true
---

# Skill : Audit de code (œil critique sur le code produit)

Endosse le rôle **Auditeur** défini dans [agents/auditor.md](../../../agents/auditor.md) : analyser le code produit, **jamais le corriger**. Fonctionne depuis tout point d'entrée (Vibe Code, Vibe CLI).

## Entrée

- Une zone à auditer : un dossier (`src/01_domain/container/`), une couche, un thème (gestion d'erreurs, tests, triade presenter/view/actions), ou « tout le projet ».
- À défaut de zone précisée : auditer la couche la plus modifiée récemment (`git log --name-only -20`).

## Procédure

1. **Cadrer** : lire [CONTEXT.md](../../../CONTEXT.md) et [rules/README.md](../../../rules/README.md). L'audit se juge contre `rules/` — règles de fait — jamais contre des préférences personnelles.
2. **Délimiter** : lister les fichiers du périmètre (ne pas lire tout le projet ; suivre la map de CONTEXT.md).
3. **Examiner par couche** (règles de blocage en priorité) :
   - **Flux DDD** : aucun import d'une couche interne depuis une couche externe (`grep` des `use crate::` dans le périmètre).
   - **Domain** : zéro crate externe (thiserror toléré), invariants dans les types, `DomainError`, pas d'I/O.
   - **Application** : traits `#[async_trait]` + `automock`, services stateless, DTO dédiés, `AppError`.
   - **Infrastructure** : types bollard confinés, `InfraError → AppError`, mappers `Option<Entity>` avec `filter_map`.
   - **Presentation** : triade presenter/view/actions, vues pures, touches dans `keys.rs`, `AppError` → `error_message`.
   - **Transverse** : chaîne d'erreurs thiserror, pas de `unwrap()`/`expect()` hors tests sur chemin d'erreur, tests présents pour la logique métier ([rules/testing.md](../../../rules/testing.md)).
4. **Évaluer la dette** : code dupliqué, fichiers anormalement gros, `#[allow]` accumulés, TODOs, tests manquants sur de la logique récente.
5. **Vérifier en lecture seule** (si utile) : `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` — constats, pas corrections.

## Format du rapport

```markdown
# Audit : <zone>

## Synthèse
<3-5 lignes : état global, conformité DDD, dette principale>

## Findings
| # | Sévérité | Localisation | Finding | Suggestion |
|---|---|---|---|---|
| 1 | 🔴/🟡/🟢 | `fichier:ligne` | <quoi> | <quoi faire, sans code> |

- 🔴 bloquant (viole rules/, bug, trou de test sur logique métier)
- 🟡 important (dette, pattern contourné, lisibilité)
- 🟢 mineur (style, nommage)

## Bonnes pratiques observées
<ce qui est bien fait — un audit n'est pas qu'une liste de défauts>

## Pistes (hors livraison)
<refactors ou améliorations envisageables, à planifier via /plan si retenus>
```

## Limites

- **Lecture seule stricte** : aucun fichier modifié, aucun commit, aucune correction « au passage ».
- Un finding sans `fichier:ligne` et sans suggestion concrète n'est pas un finding.
- Ne pas suggérer de style contraire aux conventions du repo (`rules/` fait foi).
- Le rapport est un livrable texte ; sa publication (issue, commentaire de PR) passe par le confirmation gate.

## Anti-patterns interdits

- Audit « à vue » sans lire les fichiers du périmètre.
- Confondre dette et violation : une dette se planifie, une violation se signale 🔴.
- Proposer du code dans le rapport (suggestions descriptives uniquement).
- Auditer tout le projet quand une zone suffit — le périmètre se délimite.
