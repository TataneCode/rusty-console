# Agent Architecture — rusty-console

Arborescence exploitable par tout agent IA (Mistral Vibe, ou autre), sans dépendance à un écosystème propriétaire.

```
CONTEXT.md            contexte général du projet (pitch, stack, état, cartographie)
rules/                règles par couche DDD, mutualisées entre agents et skills
  README.md                   index des règles + flux de dépendances
  domain.md                   src/01_domain/
  application.md              src/02_application/
  infrastructure.md           src/03_infrastructure/
  presentation.md             src/04_presentation/
  quality.md                  transverse : fmt, clippy, tests, MSRV, style
.agents/skills/       procédures réutilisables (standard Agent Skills, découvertes par Vibe Code et Vibe CLI)
  rust-architecture-planning/   conception et plans
  rust-feature-ddd/             implémentation couche par couche
  rust-refactoring/             refactoring comportement-preserving
  rust-testing/                 conventions de tests
  rust-quality-gate/            contrôle qualité avant livraison
  rust-bugfix/                  correction de bugs
  rust-dependency/              gestion des dépendances Cargo
  rust-code-review/             revue de pull requests
  layer-rules/                  index des règles par couche (rules/)
  plan/                         skill utilisateur /plan (rôle Architecte, lecture seule, approbation du plan obligatoire)
  implement/                    skill utilisateur /implement (rôle Développeur, protection commit/push)
  review/                       skill utilisateur /review (rôle Reviewer, PR GitHub ou diff de branche)
agents/               définitions de rôles orchestrant les skills (agnostiques)
  architect.md                produit les plans, jamais de code
  developer.md                exploite les plans, produit et vérifie le code
  reviewer.md                 relit les PR GitHub
.vibe/agents/         agents Vibe CLI (TOML) : architect.toml, developer.toml, reviewer.toml
.vibe/prompts/        prompts système des agents Vibe CLI
.vibe/config.toml     modes intégrés désactivés (ne reste que ask + agents projet)
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

Les points d'entrée utilisateur sont les skills `plan`, `implement` et `review` (slash-commands `/plan`, `/implement`, `/review`), qui endossent chacun le rôle de l'agent correspondant et orchestrent les skills métier :
- `plan` — Architecte, lecture seule sur les sources, tous les outils accessibles ; **approbation explicite du plan obligatoire** avant implémentation ; si approuvé, invoque `implement`.
- `implement` — Développeur, tous les outils accessibles ; **jamais de commit ni de push sans accord utilisateur** ; gate `cargo build/test/fmt/clippy` ; en Vibe Code, surveillance des checks de la PR.
- `review` — Reviewer, lecture seule stricte ; en Vibe Code, revue de la PR GitHub ; en CLI, diff de branche (courante vs `main` par défaut).

## Contrat entre agents

- Le plan de l'Architecte est le seul format d'entrée du Développeur pour un travail planifié ; chaque étape doit être actionnable sans retour vers l'Architecte.
- Le Développeur signale tout écart au plan au lieu de l'exécuter silencieusement.
- Le Reviewer juge sur les skills et les règles de couche (`rules/` — règles de fait), pas sur des préférences personnelles ; son verdict est toujours argumenté (`fichier:ligne` + suggestion).

## Format des skills

Chaque skill vit dans `.agents/skills/<nom>/SKILL.md` avec un frontmatter YAML (`name`, `description` — qui sert de description d'invocation), une procédure systématique étape par étape, et une section d'anti-patterns interdits.

Les règles de couche sont **mutualisées** dans `rules/` : agents et skills les référencent au lieu de dupliquer les instructions. Le skill `layer-rules` sert d'index.

### Invocation depuis les runtimes

- **Vibe Code (sandbox)** et **Vibe CLI** découvrent nativement les skills dans `.agents/skills/` (standard Agent Skills, dossier trusted). Le champ `user-invocable: true` du frontmatter expose un skill en slash-commande (`/<name>`).
- **Vibe CLI** charge les agents depuis `.vibe/agents/*.toml` (sélection : `vibe --agent architect|developer|reviewer`, ou Shift+Tab en interactif). Les prompts système correspondants vivent dans `.vibe/prompts/`. `.vibe/config.toml` désactive les modes intégrés (`plan`, `accept-edits`, `auto-approve`, `smart-approve`, `lean`) : ne restent que `ask` et les agents projet, avec `ask` par défaut.
- Tout autre runtime peut consommer les fichiers Markdown agnostiques (`agents/`, `rules/`, `.agents/skills/`) sans dépendre d'un écosystème propriétaire.
