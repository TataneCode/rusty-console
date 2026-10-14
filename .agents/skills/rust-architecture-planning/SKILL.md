---
name: rust-architecture-planning
description: Produire un plan d'architecture ou de conception pour rusty-console — analyse de l'existant, options, décision, plan d'implémentation étape par étape. Utiliser ce skill dès qu'on parle de planification, de conception, de nouvelle area fonctionnelle, ou avant toute implémentation non triviale.
---

# Skill : Conception et planification Rust

## Contexte projet

rusty-console : TUI Rust (ratatui, crossterm, bollard, tokio) pour Docker. Architecture DDD 4 couches à flux de dépendances strict :

```
04_presentation → 02_application → 01_domain ← 03_infrastructure
```

Répertoires réels : `src/01_domain/`, `src/02_application/`, `src/03_infrastructure/docker/`, `src/04_presentation/tui/`. Références de conception : `docs/refactoring/`, `.legacy/architecture_logs_26042026.md`, `.legacy/PLAN.md`, `.legacy/EVOLUTION.md`.

## Procédure systématique — 6 étapes

### 1. Cadrer la demande
- Reformuler le besoin en une phrase : « <verbe> <périmètre> pour <bénéfice> ».
- Lister les contraintes non négociables (architecture DDD, MSRV 1.88, pas de nouvelle dépendance sans justification, clippy `-D warnings`).

### 2. Auditer l'existant
- Localiser les areas concernées (`container`, `image`, `volume`, `stack`).
- Identifier les symboles impactés via recherche (`grep -rn`) dans les 4 couches.
- Vérifier les précédents : un problème déjà résolu ailleurs dans le code est un pattern à réutiliser, pas à réinventer.

### 3. Proposer des options (2–3 max)
- Chaque option : description courte, impacts par couche (domain/application/infrastructure/presentation), coût estimé, risques.
- Recommander une option avec justification. Ne jamais présenter d'options creuses.

### 4. Décider les points d'architecture
- Respecter la règle de placement : le code va dans la couche la plus interne qui le concerne.
- Toute nouvelle entité → value objects + `DomainError` dédiés.
- Toute nouvelle source de données → trait repository en `02_application`, adapter en `03_infrastructure`.
- Tout nouvel écran → triade presenter/view/actions + `Screen` + `AppAction` + câblage `main.rs`.
- Chaque décision notée avec sa rationale (pourquoi, pas seulement quoi).

### 5. Produire le plan d'implémentation
Format du livrable :

```markdown
# Plan : <titre>

## Objectif
<une phrase>

## Décisions d'architecture
- D1 : <décision> — <raison>
- D2 : ...

## Étapes
1. <étape atomique> — fichiers : <chemins> — vérification : <commande>
2. ...
   (chaque étape laisse le repo compilable et les tests verts)

## Risques et mitigations
- <risque> → <mitigation>

## Hors périmètre
- <ce qui est explicitement exclu>
```

- Une étape = un commit potentiel ; ordre des couches : domain → application → infrastructure → presentation → wiring.
- Chaque étape cite ses commandes de vérification (`cargo check`, `cargo test <name>`, etc.).

### 6. Validation finale du plan
- Le plan ne contient pas de code d'implémentation (snippets illustratifs courts admis pour les APIs clés).
- Vérifier la cohérence : chaque fichier mentionné existe ou est explicitement créé ; aucune étape ne viole le flux de dépendances.
- Anticiper les tests : chaque étape de logique métier a son test associé (voir skill `rust-testing`).

## Anti-patterns interdits

- Plan mélangé à de l'implémentation détaillée (le plan décrit le quoi/pourquoi, pas le comment ligne à ligne).
- Décision d'architecture sans rationale.
- Étape trop large (« refacto l'UI ») ou qui laisse le repo cassé en cours de route.
- Ignorer les patterns existants du projet au profit d'une approche générique.
