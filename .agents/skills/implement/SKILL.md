---
name: implement
description: Implémenter une fonctionnalité ou un bugfix rusty-console via le rôle Développeur, à partir d'un plan approuvé de l'Architecte — tous les outils accessibles (équivalent auto-approve), jamais de commit ni de push sans accord explicite de l'utilisateur, validation par cargo build / test / fmt / clippy, surveillance des checks de la PR en Vibe Code.
user-invocable: true
---

# Skill : Implement (rôle Développeur)

Endosse le rôle **Développeur** défini dans [agents/developer.md](../../../agents/developer.md) : implémenter et vérifier, **jamais produire de plan**. Fonctionne depuis tout point d'entrée (Vibe Code, Vibe CLI).

## Entrée

- Un plan **approuvé** issu du skill `plan` (format `rust-architecture-planning`), ou une instruction directe suffisamment précise.
- Si aucun plan n'est fourni et que la tâche n'est pas triviale : demander à passer par le skill `plan` plutôt qu'improviser.

## Mode opératoire

1. Charger le contexte : [CONTEXT.md](../../../CONTEXT.md), le plan, et les règles des couches touchées ([rules/](../../../rules/)).
2. Appliquer le skill correspondant au type de travail : `rust-feature-ddd` (défaut, couche par couche), `rust-bugfix` (bug signalé ou test rouge), `rust-refactoring` (comportement identique) — plus `rust-testing` pour toute nouvelle logique et `rust-dependency` si `Cargo.toml` change.
3. Exécuter les étapes du plan dans l'ordre ; une étape = un commit potentiel, le repo reste vert à chaque étape. Tout écart nécessaire au plan est signalé, jamais fait silencieusement.
4. Passer le **quality gate complet** avant de déclarer terminé (skill `rust-quality-gate`) : `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy -- -D warnings`.
5. **Self-review obligatoire** avant toute livraison externe (skill `self-review` : checklist `rules/`, imports inter-couches, gate local relu sur le diff) — puis `/review` si une review externe est demandée.

## Règles de protection (non négociables)

- **Aucun `git commit` ni `git push` sans accord explicite préalable de l'utilisateur**, quel que soit le point d'entrée. Le dépôt reste en état de travail jusqu'à cet accord.
- Toute écriture reste couverte par le confirmation gate ([confirmation-gate](../confirmation-gate/SKILL.md)) : périmètre présenté, accord reçu, puis exécution à l'identique.

## En Vibe Code — surveillance de la PR

Après ouverture d'une PR (avec accord utilisateur) :
1. Surveiller les checks de la PR (GitHub Actions : build, test, clippy, fmt).
2. En cas d'échec, identifier la cause racine dans les logs, corriger localement, repasser le quality gate.
3. Un nouveau push pour livrer la correction demande un nouvel accord.

## Rapport final

Étapes réalisées, fichiers touchés, résultat du gate (commandes exécutées + verdict), écarts par rapport au plan avec raison.

## Anti-patterns interdits

- Commiter ou pousser sans accord explicite.
- Modifier le périmètre du plan silencieusement.
- Contourner une règle des skills ou de `rules/`, même pour aller plus vite.
- Déclarer terminé sans gate vert constaté (commandes exécutées et lues).
