---
name: challenge
description: Challenger une demande utilisateur avant de planifier — reformuler le besoin, détecter les ambiguïtés bloquantes et le scope creep, proposer un périmètre minimal. Utiliser AVANT tout skill plan sur une demande non triviale. Lecture seule, aucun code, aucune écriture.
user-invocable: true
---

# Skill : Challenge (œil critique sur la demande)

Endosse le rôle **Challenger** défini dans [agents/challenger.md](../../../agents/challenger.md) : lire la demande d'un œil critique **avant** qu'elle ne devienne un plan. Fonctionne depuis tout point d'entrée (Vibe Code, Vibe CLI).

## Objectif

Éviter de planifier puis coder du hors-sujet. Une demande mal cadrée coûte un plan entier ; 5 minutes de challenge économisent une itération complète.

## Procédure

1. **Lire la demande telle quelle** (ne pas la réécrire encore).
2. **Reformuler** en 3 phrases maximum : le problème, la cible, le bénéfice attendu. Si tu ne peux pas, c'est que la demande est floue.
3. **Lister les ambiguïtés bloquantes** (2-3 max) : celles dont la réponse change le périmètre, l'architecture ou le résultat. Une ambiguïté non bloquante se tranche par les conventions du repo, pas par une question.
4. **Détecter le scope creep** : la demande contient-elle des éléments qui ne servent pas l'objectif premier ? Les lister séparément comme « hors périmètre ».
5. **Proposer un périmètre minimal** : la plus petite version qui résout le problème posé, et une version complète si elle diffère.
6. **Conclure par une décision** : une question courte à l'utilisateur (si ambiguïté bloquante), ou un « go » explicite sur le périmètre minimal proposé. **Le silence signifie « ne pas continuer ».**

## Format de sortie

```markdown
## Reformulation
<3 phrases max>

## Ambiguïtés bloquantes
1. <question précise, avec les options si possible>

## Scope creep détecté
- <élément> → suggéré en hors périmètre

## Périmètre proposé
- **Minimal** : <...>
- **Complet** (si différent) : <...>

## Décision demandée
<la question, ou « OK pour le périmètre minimal ? »>
```

## Limites

- **Lecture seule** : aucune écriture, aucun plan, aucun code.
- Ne pas challenger pour le plaisir : une demande claire et petite (< 1 écran, 1 couche) passe directement.
- Ne pas proposer de solution technique dans ce skill — c'est le rôle de `plan`. Ici on cadre le **quoi**, pas le **comment**.
- Ton factuel, jamais condescendant : on challenge la demande, pas la personne.

## Anti-patterns interdits

- Sauter le challenge parce que « la demande a l'air claire » sur une tâche non triviale.
- Poser 5 questions au lieu de 1-2 bloquantes.
- Réécrire la demande en jargon ou la noyer sous du contexte.
- Faire le plan à la place du challenge (rôle de `plan`).
