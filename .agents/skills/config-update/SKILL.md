---
name: config-update
description: Faire évoluer la configuration agent de rusty-console — ajouter/modifier un skill, une règle de couche, un agent Vibe CLI, une commande slash ; garder la cohérence (pas de duplication, sources de vérité uniques, références à jour). Utiliser quand on touche à AGENTS.md, CONTEXT.md, rules/, .agents/skills/, agents/ ou .vibe/.
user-invocable: true
---

# Skill : Mise à jour de la configuration agent

Faire évoluer la configuration vibe coding du dépôt (skills, règles, agents, commandes) **sans casser la cohérence** et en maîtrisant la taille du contexte. Fonctionne depuis tout point d'entrée (Vibe Code, Vibe CLI).

## Principes directeurs

1. **Une source de vérité par sujet** — ne jamais dupliquer une instruction dans deux fichiers. Si un contenu existe déjà, on référence, on ne copie pas.
2. **Progressive disclosure** — un skill verbeux se scinde : procédure courte dans `SKILL.md`, détails dans `references/<sujet>.md` chargé à la demande.
3. **Le contexte se paie** — chaque ligne de `AGENTS.md` est injectée à chaque session. Alléger > enrichir.
4. **Dualité Vibe Code / Vibe CLI** — un skill (`.agents/skills/`) couvre les deux ; un agent TOML (`.vibe/agents/`) ne couvre que Vibe CLI et doit avoir son pendant skill ou son rôle documenté dans `agents/`.

## Procédure

### 1. Cadrer le changement
- Quel sujet évolue (skill, règle, agent, commande) ? Quel problème concret résout-il (tokens, duplication, friction) ?
- Lire [CONTEXT.md](../../../CONTEXT.md) et [AGENT_ARCHITECTURE.md](../../../AGENT_ARCHITECTURE.md) pour comprendre l'articulation globale.

### 2. Choisir le bon endroit (anti-duplication)

| Le contenu est… | Il vit dans |
|---|---|
| Une règle de couche ou transverse (normatif, stable) | `rules/<couche>.md` ou `rules/testing.md` / `rules/quality.md` |
| Une procédure opérationnelle (comment faire une tâche) | `.agents/skills/<nom>/SKILL.md` |
| Un rôle d'agent (persona) | `agents/<role>.md` + `.vibe/agents/<role>.toml` + `.vibe/prompts/<role>.md` |
| De la navigation / du contexte projet | `CONTEXT.md` |
| Du routage de démarrage | `AGENTS.md` (table de routage uniquement) |
| Un détail de procédure trop verbeux | `references/` du skill concerné |

### 3. Appliquer le changement

- **Nouveau skill** : `.agents/skills/<nom>/SKILL.md` avec frontmatter (`name`, `description` avec déclencheurs, `user-invocable: true` si commande slash). S'il a un pendant agent CLI : `agents/<role>.md` + `.vibe/agents/<role>.toml` (safety adapté) + `.vibe/prompts/<role>.md`.
- **Nouveau rôle d'agent Vibe CLI** : `.vibe/agents/<role>.toml` — `safety = "safe"` si lecture seule, `"neutral"` si écriture possible (allowlist bash restrictive, git commit/push hors allowlist).
- **Nouvelle règle** : l'ajouter dans le fichier `rules/` concerné, référencer depuis `rules/README.md`, et faire pointer les skills/agents concernés dessus au lieu de résumer.
- **Modification de `AGENTS.md` ou `CONTEXT.md`** : mettre à jour la table de routage des deux côtés ; vérifier qu'aucun contenu normatif n'y a été réintroduit.

### 4. Propager les références

- `grep -rn "<ancien-chemin-ou-nom>"` sur le dépôt : mettre à jour **tous** les pointeurs (skills, agents, prompts, docs).
- Mettre à jour [AGENT_ARCHITECTURE.md](../../../AGENT_ARCHITECTURE.md) (arborescence + orchestration) si la structure a changé.
- Ne jamais laisser `.legacy/` référencé comme source par un skill actif (archive uniquement).

### 5. Vérifier la cohérence

- **Zéro duplication normative** : un même bloc d'instructions n'existe qu'une fois (grep les titres de sections entre fichiers).
- **Liens valides** : chaque lien Markdown pointe vers un fichier existant.
- **Pas de régression de contexte** : `wc -l AGENTS.md` n'a pas augmenté sans justification ; les skills allégés délèguent à `references/`.
- **Confirmation gate** : toute écriture passe par [.agents/skills/confirmation-gate/SKILL.md](../confirmation-gate/SKILL.md) — plan présenté, accord explicite, exécution à l'identique.

## Anti-patterns interdits

- Dupliquer une règle dans un skill au lieu de référencer `rules/`.
- Ajouter un agent Vibe CLI sans pendant skill (les deux points d'entrée doivent rester cohérents).
- Enrichir `AGENTS.md` au lieu de déléguer à `CONTEXT.md` ou `rules/`.
- Laisser un skill verbeux alors qu'un `references/` réduirait le contexte.
- Modifier la config sans mettre à jour la table de routage (`AGENTS.md` + `CONTEXT.md`).
