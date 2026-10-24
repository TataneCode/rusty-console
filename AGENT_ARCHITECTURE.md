# Agent Architecture — rusty-console

Arborescence exploitable par tout agent IA (Mistral Vibe, ou autre), sans dépendance à un écosystème propriétaire.

```
CONTEXT.md            project map : contexte général, navigation (quoi lire, quand), état du projet
AGENTS.md             routeur de démarrage : confirmation gate + table de routage + sources de vérité
rules/                règles par couche DDD, mutualisées entre agents et skills (source unique)
  README.md                   index des règles + flux de dépendances
  domain.md                   src/01_domain/
  application.md              src/02_application/
  infrastructure.md           src/03_infrastructure/
  presentation.md             src/04_presentation/
  quality.md                  transverse : fmt, clippy, tests, MSRV, style
  testing.md                  transverse : conventions UT & IT, seuil de décision
.agents/skills/       procédures réutilisables (standard Agent Skills, découvertes par Vibe Code et Vibe CLI)
  challenge/                  skill utilisateur /challenge (rôle Challenger, cadrage de la demande)
  plan/                       skill utilisateur /plan (rôle Architecte, approbation du plan obligatoire)
  implement/                  skill utilisateur /implement (rôle Développeur, protection commit/push)
  self-review/                skill utilisateur /self-review (relire son diff avant review externe)
  review/                     skill utilisateur /review (rôle Reviewer, PR GitHub ou diff de branche)
  audit/                      skill utilisateur /audit (rôle Auditeur, analyse du code produit)
  config-update/              skill utilisateur /update-config (faire évoluer cette configuration)
  rust-architecture-planning/   conception et plans (+ references/plan-format.md)
  rust-feature-ddd/             implémentation couche par couche (+ references/layer-checklists.md)
  rust-refactoring/             refactoring comportement-preserving
  rust-testing/                 conventions de tests (résumé de rules/testing.md)
  rust-quality-gate/            contrôle qualité avant livraison
  rust-bugfix/                  correction de bugs
  rust-dependency/              gestion des dépendances Cargo
  rust-code-review/             revue de pull requests
  layer-rules/                  index des règles par couche (rules/)
  confirmation-gate/            gate de confirmation obligatoire avant toute écriture
agents/               définitions de rôles orchestrant les skills (agnostiques)
  challenger.md               challenge la demande avant planification, jamais de plan ni de code
  architect.md                produit les plans, jamais de code
  developer.md                exploite les plans, produit et vérifie le code
  reviewer.md                 relit les PR GitHub
  auditor.md                  analyse le code produit, lecture seule, jamais de correction
.vibe/agents/         agents Vibe CLI (TOML) : challenger.toml, architect.toml, developer.toml, reviewer.toml, auditor.toml
.vibe/prompts/        prompts système des agents Vibe CLI
.vibe/config.toml     modes intégrés désactivés (ne reste que ask + agents projet)
AGENT_ARCHITECTURE.md (ce fichier) — comment tout s'articule
```

## Orchestration

| Phase | Agent | Skills invoqués | Livrable |
|---|---|---|---|
| Cadrage | **Challenger** | `challenge` | Demande reformulée, ambiguïtés bloquantes, périmètre minimal |
| Planification | **Architecte** | `rust-architecture-planning`, `rust-dependency` | Plan (objectif, décisions, étapes vérifiables, risques, hors périmètre) |
| Implémentation | **Développeur** | `rust-feature-ddd` / `rust-refactoring` / `rust-bugfix`, `rust-testing`, `rust-dependency`, `rust-quality-gate`, `self-review` | Code + tests + gate vert + self-review |
| Revue | **Reviewer** | `rust-code-review`, `rust-quality-gate` | Revue structurée sur la PR |
| Analyse | **Auditeur** | `audit`, `layer-rules` | Rapport d'audit (findings 🔴/🟡/🟢, dette, pistes) |

Flux nominal : l'utilisateur exprime un besoin → **Challenger** cadre la demande → **Architecte** produit un plan → le plan est confié au **Développeur** qui implémente, passe le quality gate et le self-review → la PR ouverte est relue par le **Reviewer**. L'**Auditeur** intervient en lecture seule pour analyser une zone du code. Chaque agent est indépendant et peut aussi être invoqué seul.

Les points d'entrée utilisateur sont les skills user-invocables (slash-commands) :
- `/challenge` — Challenger, lecture seule ; reformule, détecte ambiguïtés et scope creep, propose le périmètre minimal ; à utiliser **avant** `/plan` sur une demande non triviale.
- `/plan` — Architecte, lecture seule sur les sources, tous les outils accessibles ; **approbation explicite du plan obligatoire** avant implémentation ; si approuvé, invoque `implement`.
- `/implement` — Développeur, tous les outils accessibles ; **jamais de commit ni de push sans accord utilisateur** ; gate `cargo build/test/fmt/clippy` + self-review obligatoire avant livraison externe.
- `/self-review` — Développeur, relit son propre diff (checklist `rules/`, imports inter-couches, gate local) avant `/review`.
- `/review` — Reviewer, lecture seule stricte ; en Vibe Code, revue de la PR GitHub ; en CLI, diff de branche (courante vs `main` par défaut).
- `/audit` — Auditeur, lecture seule stricte ; analyse une zone du code produit, rapport structuré.
- `/update-config` — faire évoluer cette configuration (skills, règles, agents, commandes) en gardant la cohérence.

## Contrat entre agents

- Le Challenger cadre la demande avant que l'Architecte ne planifie ; le plan de l'Architecte est le seul format d'entrée du Développeur pour un travail planifié — chaque étape doit être actionnable sans retour vers l'Architecte.
- Le Développeur signale tout écart au plan au lieu de l'exécuter silencieusement, et self-review son diff avant toute review externe.
- Le Reviewer et l'Auditeur jugent sur les skills et les règles de couche (`rules/` — règles de fait), pas sur des préférences personnelles ; leurs verdicts/findings sont toujours argumentés (`fichier:ligne` + suggestion).

## Format des skills

Chaque skill vit dans `.agents/skills/<nom>/SKILL.md` avec un frontmatter YAML (`name`, `description` — qui sert de description d'invocation), une procédure systématique étape par étape, et une section d'anti-patterns interdits. Les détails verbeux vivent dans `references/` du skill, chargés à la demande (progressive disclosure).

Les règles de couche sont **mutualisées** dans `rules/` : agents et skills les référencent au lieu de dupliquer les instructions. Le skill `layer-rules` sert d'index.

### Invocation depuis les runtimes

- **Vibe Code (sandbox)** et **Vibe CLI** découvrent nativement les skills dans `.agents/skills/` (standard Agent Skills, dossier trusted). Le champ `user-invocable: true` du frontmatter expose un skill en slash-commande (`/<name>`).
- **Vibe CLI** charge les agents depuis `.vibe/agents/*.toml` (sélection : `vibe --agent challenger|architect|developer|reviewer|auditor`, ou Shift+Tab en interactif). Les prompts système correspondants vivent dans `.vibe/prompts/`. `.vibe/config.toml` désactive les modes intégrés (`plan`, `accept-edits`, `auto-approve`, `smart-approve`, `lean`) : ne restent que `ask` et les agents projet, avec `ask` par défaut.
- Tout autre runtime peut consommer les fichiers Markdown agnostiques (`agents/`, `rules/`, `.agents/skills/`) sans dépendre d'un écosystème propriétaire.
