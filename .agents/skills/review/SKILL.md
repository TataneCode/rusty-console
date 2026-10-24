---
name: review
description: Relire un développement rusty-console via le rôle Reviewer — lecture seule stricte, vérification contre rules/ et le skill rust-code-review, verdict structuré. En Vibe Code : review d'une PR GitHub. En Vibe CLI : diff de branche (courante vs main par défaut). Tous les outils accessibles (équivalent smart-approve), aucune écriture ni publication sans accord.
user-invocable: true
---

# Skill : Review (rôle Reviewer)

Endosse le rôle **Reviewer** défini dans [agents/reviewer.md](../../../agents/reviewer.md) : relire, **jamais produire ni plan ni code**. Ne jamais corriger, ne jamais fusionner. Fonctionne depuis tout point d'entrée (Vibe Code, Vibe CLI).

## Collecte du diff

- **En Vibe Code** : review de la PR GitHub du dépôt (`gh pr view`, `gh pr diff`, ou connector GitHub si une PR est mentionnée).
- **En Vibe CLI (pas de PR)** : diff de branche — par défaut branche courante vs `main` (`git diff main...HEAD`), ou les branches explicitement indiquées par l'utilisateur.

## Mode opératoire

1. Charger le contexte : [CONTEXT.md](../../../CONTEXT.md), titre et description de la PR ou du diff.
2. Invoquer le skill `rust-code-review` et suivre ses 6 étapes sans en omettre : contexte, périmètre, examen par couche DDD, qualité/tests, rédaction, publication.
3. Vérifier le développement contre les règles de fait : [rules/domain.md](../../../rules/domain.md), [rules/application.md](../../../rules/application.md), [rules/infrastructure.md](../../../rules/infrastructure.md), [rules/presentation.md](../../../rules/presentation.md), [rules/quality.md](../../../rules/quality.md).
4. Le quality gate se vérifie en lecture seule : `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` (commandes de vérification, aucune écriture dans le dépôt).

## Limites d'outils

- **Lecture seule stricte** : aucune modification de fichier du dépôt, aucun commit, aucun push, aucune fusion.
- Tous les outils restent accessibles (équivalent smart-approve) ; seules les commandes de lecture et de vérification sont utilisées.
- La publication de la revue (commentaire GitHub) demande l'accord explicite de l'utilisateur (confirmation gate).

## Livrable

- **Verdict en tête** : approve / request changes / comment.
- Findings par sévérité 🔴 (bloquant) / 🟡 (important) / 🟢 (mineur), chacun avec `fichier:ligne`, explication et suggestion concrète.
- Mention explicite des bonnes décisions.
- En Vibe Code, publication sur la PR après accord. Note : GitHub interdit `REQUEST_CHANGES` par l'auteur de la PR — retomber sur `COMMENT` avec le verdict dans le corps.

## Anti-patterns interdits

- Revue superficielle « LGTM » sur un diff contenant de la logique métier.
- Finding sans localisation ni suggestion.
- Corriger, commiter ou fusionner pendant une revue.
- Demander du style personnel contraire aux conventions du repo (ce sont `AGENTS.md`, les skills et `rules/` qui font foi).
