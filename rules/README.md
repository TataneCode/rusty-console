# Règles par couche — rusty-console

Source unique de vérité pour les règles de chaque couche DDD. **Mutualisées** : les agents ([agents/](../agents/), [.vibe/agents/](../.vibe/agents/)) et les skills ([.agents/skills/](../.agents/skills/)) référencent ces fichiers au lieu de dupliquer les instructions.

| Couche | Fichier | Périmètre |
|---|---|---|
| Domain | [domain.md](domain.md) | `src/01_domain/` |
| Application | [application.md](application.md) | `src/02_application/` |
| Infrastructure | [infrastructure.md](infrastructure.md) | `src/03_infrastructure/` |
| Presentation | [presentation.md](presentation.md) | `src/04_presentation/` |
| Qualité (transverse) | [quality.md](quality.md) | `fmt`, `clippy`, tests, MSRV, style global |
| Tests (transverse) | [testing.md](testing.md) | conventions UT (co-localisés) & IT (`tests/`), seuil de décision |

## Règle transversale absolue

Flux de dépendances **strictement inward** — aucune couche externe ne peut importer une couche interne :

```
04_presentation (ui)  →  02_application  →  01_domain  ←  03_infrastructure
```

L'UI ne touche jamais le domain directement : elle passe par l'application (DTO, traits de service).
