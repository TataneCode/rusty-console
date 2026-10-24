---
name: plan
description: Produire un plan de conception rusty-console via le rôle Architecte — lecture seule sur les sources, tous les outils accessibles (équivalent smart-approve), approbation explicite du plan obligatoire avant toute implémentation, puis invocation du skill implement. Utiliser pour cadrer une fonctionnalité, un bugfix ou un refactoring avant d'écrire du code.
user-invocable: true
---

# Skill : Plan (rôle Architecte)

Endosse le rôle **Architecte** défini dans [agents/architect.md](../../../agents/architect.md) : produire des plans de conception, **jamais de code d'implémentation**. Fonctionne depuis tout point d'entrée (Vibe Code, Vibe CLI).

## Mode opératoire

> Sur une demande non triviale, le skill `challenge` (`/challenge`) doit avoir cadré la demande **avant** ce skill — ne pas planifier une demande non cadrée.

1. Charger le contexte : [CONTEXT.md](../../../CONTEXT.md), puis [rules/README.md](../../../rules/README.md).
2. Invoquer le skill `rust-architecture-planning` et suivre ses 6 étapes : cadrage, audit de l'existant, options (2–3 max), décisions d'architecture, plan, validation finale.
3. Invoquer `rust-dependency` dès que le plan envisage un nouveau crate (MSRV 1.88, features minimales).
4. Auditer l'existant en **lecture seule** : `read_file`, `grep`, commandes shell non destructrices.
5. Consulter les règles de couche ([rules/](../../../rules/)) pour garantir que chaque étape respecte le flux DDD.

## Limites d'outils

- **Lecture seule sur les sources** : aucune modification de fichier du dépôt pendant la phase de plan.
- Tous les outils restent accessibles (équivalent smart-approve) ; une écriture éventuelle passe par l'approbation explicite de l'utilisateur.

## Approbation du plan (obligatoire)

1. Présenter le plan au format `rust-architecture-planning` : Objectif, Décisions (avec rationale), Étapes avec commandes de vérification, Risques, Hors périmètre.
2. Demander l'approbation explicite de l'utilisateur. **Le silence ou une réponse ambiguë signifie « ne pas continuer ».**
3. Plan approuvé → invoquer le skill `implement` pour confier l'exécution au rôle Développeur.
4. Plan rejeté ou amendé → itérer sur le plan, toujours sans toucher aux sources.

## Anti-patterns interdits

- Commencer l'implémentation sans approbation explicite du plan.
- Plan mélangé à du code d'implémentation détaillé (snippets d'API illustratifs courts admis).
- Décision d'architecture sans rationale, ou étape qui viole le flux de dépendances DDD.
- Ignorer les patterns existants du projet au profit d'une approche générique.
