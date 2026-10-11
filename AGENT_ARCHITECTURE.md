# Agent Architecture — rusty-console

Arborescence exploitable par tout agent IA (Mistral Vibe, ou autre), sans dépendance à un écosystème propriétaire.

```
skills/               Procédures réutilisables (une compétence = un dossier)
  rust-architecture-planning/   conception et plans
  rust-feature-ddd/             implémentation couche par couche
  rust-refactoring/             refactoring comportement-preserving
  rust-testing/                 conventions de tests
  rust-quality-gate/            contrôle qualité avant livraison
  rust-bugfix/                  correction de bugs
  rust-dependency/              gestion des dépendances Cargo
  rust-code-review/             revue de pull requests
agents/               Définitions de rôles orchestrant les skills
  architect.md                produit les plans, jamais de code
  developer.md                exploite les plans, produit et vérifie le code
  reviewer.md                  relit les PR GitHub
AGENT_ARCHITECTURE.md  (ce fichier) — comment tout s'articule
AGENTS.md              conventions du code, point d'entrée commun
```

## Orchestration

| Phase | Agent | Skills invoqués | Livrable |
|---|---|---|---|
| Planification | **Architecte** | `rust-architecture-planning`, `rust-dependency` | Plan (objectif, décisions, étapes vérifiables, risques, hors périmètre) |
| Implémentation | **Développeur** | `rust-feature-ddd` / `rust-refactoring` / `rust-bugfix`, `rust-testing`, `rust-dependency`, `rust-quality-gate` | Code + tests + gate vert |
| Revue | **Reviewer** | `rust-code-review`, `rust-quality-gate` | Revue structurée sur la PR |

Flux nominal : l'utilisateur parle conception → **Architecte** produit un plan → le plan est confié au **Développeur** qui implémente et passe le quality gate → la PR ouverte est relue par le **Reviewer**. Chaque agent est indépendant et peut aussi être invoqué seul (ex. revue d'une PR externe sans plan).

## Contrat entre agents

- Le plan de l'Architecte est le seul format d'entrée du Développeur pour un travail planifié ; chaque étape doit être actionnable sans retour vers l'Architecte.
- Le Développeur signale tout écart au plan au lieu de l'exécuter silencieusement.
- Le Reviewer juge sur les skills (règles de fait), pas sur des préférences personnelles ; son verdict est toujours argumenté (`fichier:ligne` + suggestion).

## Format des skills

Chaque skill vit dans `skills/<nom>/SKILL.md` avec un frontmatter YAML (`name`, `description` — qui sert de description d'invocation), une procédure systématique étape par étape, et une section d'anti-patterns interdits. Les agents peuvent être adaptés à tout runtime en lisant ces fichiers : rien ne dépend de `.claude`, `.github/copilot-instructions.md` ou d'un outil spécifique.
