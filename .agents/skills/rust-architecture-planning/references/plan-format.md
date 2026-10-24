# Référence — Format du plan et checklist de validation

Détail du livrable et de la validation finale de `rust-architecture-planning` (étape 5 et 6). Chargé à la demande.

## Format du livrable

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

## Checklist de validation finale

- Le plan ne contient pas de code d'implémentation (snippets illustratifs courts admis pour les APIs clés).
- Cohérence : chaque fichier mentionné existe ou est explicitement créé ; aucune étape ne viole le flux de dépendances.
- Tests anticipés : chaque étape de logique métier a son test associé (voir skill `rust-testing` et `rules/testing.md`).
