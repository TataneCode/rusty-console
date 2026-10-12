---
name: layer-rules
description: Règles de rusty-console par couche DDD (domain, application, infrastructure, presentation) + règles qualité transverses. Utiliser ce skill dès qu'une tâche touche le code d'une couche, pour charger les règles de placement et de style applicables.
---

# Skill : Règles par couche (mutualisées)

Toute procédure qui touche au code d'une couche doit respecter les fichiers de règles correspondants :

Ces fichiers vivent à la racine du dépôt, dans `rules/` (les charger avec `read_file`) :

- `rules/domain.md` — `src/01_domain/`
- `rules/application.md` — `src/02_application/`
- `rules/infrastructure.md` — `src/03_infrastructure/`
- `rules/presentation.md` — `src/04_presentation/`
- `rules/quality.md` — commandes de vérification et style global

## Rappel rapide — flux de dépendances

```
04_presentation (ui)  →  02_application  →  01_domain  ←  03_infrastructure
```

Aucune couche externe ne peut importer une couche interne. Avant de livrer, vérifier les imports de chaque fichier modifié.
