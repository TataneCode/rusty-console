---
name: rust-architecture-planning
description: Produire un plan d'architecture ou de conception pour rusty-console — cadrage, audit de l'existant, options, décision, plan d'implémentation étape par étape. Utiliser dès qu'on parle de planification, de conception, de nouvelle area fonctionnelle, ou avant toute implémentation non triviale.
---

# Skill : Conception et planification Rust

## Contexte projet

rusty-console : TUI Rust (ratatui, crossterm, bollard, tokio) pour Docker. DDD 4 couches, flux strictement inward :

```
04_presentation → 02_application → 01_domain ← 03_infrastructure
```

Répertoires : `src/01_domain/`, `src/02_application/`, `src/03_infrastructure/docker/`, `src/04_presentation/tui/`. Contexte et navigation : [CONTEXT.md](../../../CONTEXT.md). Règles de couche (source de vérité) : [rules/](../../../rules/).

## Procédure — 6 étapes

### 1. Cadrer la demande
Reformuler en « <verbe> <périmètre> pour <bénéfice> » ; lister les contraintes non négociables (DDD, MSRV 1.88, pas de nouvelle dépendance sans justification, clippy `-D warnings`).

### 2. Auditer l'existant
Localiser les areas concernées (`container`, `image`, `volume`, `stack`) ; identifier les symboles impactés (`grep -rn`) dans les 4 couches ; vérifier les précédents — un problème déjà résolu ailleurs est un pattern à réutiliser, pas à réinventer.

### 3. Proposer des options (2-3 max)
Description courte, impacts par couche, coût estimé, risques. Recommander une option avec justification — jamais d'options creuses.

### 4. Décider les points d'architecture
- Placement : le code va dans la couche la plus interne qui le concerne.
- Nouvelle entité → value objects + `DomainError` dédiés.
- Nouvelle source de données → trait repository en `02_application`, adapter en `03_infrastructure`.
- Nouvel écran → triade presenter/view/actions + `Screen` + `AppAction` + câblage `main.rs`.
- Chaque décision notée avec sa rationale.

### 5. Produire le plan d'implémentation
Format, ordre des étapes et commandes de vérification : voir [references/plan-format.md](references/plan-format.md).

### 6. Validation finale du plan
Checklist (pas de code, cohérence, tests anticipés) : voir [references/plan-format.md](references/plan-format.md).

## Anti-patterns interdits

- Plan mélangé à de l'implémentation détaillée (le plan décrit le quoi/pourquoi, pas le comment ligne à ligne).
- Décision d'architecture sans rationale.
- Étape trop large (« refacto l'UI ») ou qui laisse le repo cassé en cours de route.
- Ignorer les patterns existants du projet au profit d'une approche générique.
