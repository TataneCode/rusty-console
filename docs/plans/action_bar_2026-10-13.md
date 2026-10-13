# Plan : Barre d'actions visible dans le langage visuel du fil d'Ariane

Date : 2026-10-13 — Branche : `breadcrumb` (suite du commit `79c5ed4`)

## Objectif

Redessiner la barre d'information des actions (aide clavier en bas de chaque écran) avec le même langage visuel que le fil d'Ariane et le menu principal (chrome cyan, touches en jaune gras), pour la rendre nettement plus lisible.

## Cadrage — précision importante

Les fichiers `actions.rs` (container, volume, image, stack, network) sont des **wrappers async purs autour des services application** : ils ne contiennent aucun affichage et ne sont donc pas modifiés. L'information des actions affichée à l'utilisateur est la **barre d'aide clavier** : chaînes `*_HELP` de `resources.rs`, rendues par `render_help` (`widgets.rs`, texte intégral en `Theme::help_style()` = DarkGray), appelée par les 5 vues de liste, les vues de détails/logs, le menu principal et le dialog exec shell (13 points d'appel).

## Décisions d'architecture

- **D1 : Cible = composant de rendu, pas `actions.rs`** — la demande porte sur l'affichage ; le code concerné est `render_help` + ses points d'appel, exclusivement en `04_presentation`. Raison : placement correct + les `actions.rs` n'ont rien à afficher.
- **D2 : Nouveau module `common/action_bar.rs` sur le pattern du fil d'Ariane** — module dédié + fonction pure testable (`action_bar_line`) + fonction de rendu. `render_help` est supprimé après migration complète des 13 points d'appel (pas de code mort). Raison : précédent `breadcrumb.rs` du repo, testabilité du parsing.
- **D3 : Même design que le fil d'Ariane / menu** — bordure cyan réutilisée (`Theme::breadcrumb_border_style()`), touches en jaune gras (`Theme::action_key_style()`, même famille que `title_style()`), libellés en blanc (`Theme::action_label_style()`), séparateurs ` | ` en DarkGray. Raison : unité visuelle demandée ; la distinction touche/libellé est le gain de lisibilité.
- **D4 : Deux points d'entrée explicites** — `render_action_bar(frame, area, help_text)` et `render_loading_bar(frame, area, spinner_char)`. Raison : le texte de loading contient le caractère spinner `|`, qui casserait un parsing naïf sur `|` ; une heuristique sur la chaîne serait fragile.
- **D5 : Parsing des chaînes existantes, `resources.rs` inchangé** — le format `" touche: Libellé | ..."` est homogène sur les 12 écrans ; le composant découpe sur `|`, `trim`, puis `split_once(':')`. Raison : diff minimal, aucune donnée déplacée.
- **D6 : `HELP_BAR_HEIGHT` reste 2** — bordure haute + une ligne de texte, aucun changement de layout des écrans. Raison : la visibilité vient du style, pas de la hauteur.
- **D7 : Helper DI partagé `tests/common/mod.rs`** — le wiring DI de `main.rs` était déjà dupliqué dans `tests/breadcrumb_ui.rs` (finding 🟢 #5 de la review) ; le nouveau test d'intégration l'utilise au lieu d'une 3e copie, et `breadcrumb_ui.rs` migre dessus. Raison : une seule source de wiring pour les tests.
- **D8 : Tests d'intégration vérifiant la visibilité** — `tests/action_bar_ui.rs` rend l'écran des containers sur `TestBackend` et asserte à la fois le texte d'aide et la **couleur des cellules** (fg jaune gras sur les touches). Raison : la demande est une exigence visuelle ; une assertion de texte seule ne la prouverait pas.

## Étapes

1. **Styles de thème** — fichiers : `common/theme.rs` — `action_key_style()` (jaune gras), `action_label_style()` (blanc) + tests — vérification : `cargo test theme`.
2. **Composant barre d'actions** — fichiers : `common/action_bar.rs` (nouveau), `common/mod.rs` — `action_bar_line(help_text) -> Line` (touches stylées, libellés blancs, séparateurs dim), `render_action_bar`, `render_loading_bar` + tests unitaires (parsing + rendu `TestBackend`) — vérification : `cargo test action_bar`.
3. **Migration des 13 points d'appel et suppression de `render_help`** — fichiers : `common/menu.rs`, `common/widgets.rs` (`render_selection_dialog`), `container/view.rs`, `volume/view.rs`, `image/view.rs`, `stack/view.rs`, `network/view.rs` — branches loading explicites `render_loading_bar`, aide via `render_action_bar` — vérification : `cargo test` (les tests existants assertent le texte `.trim()`, qui reste identique).
4. **Tests d'intégration + CI** — fichiers : `tests/common/mod.rs` (nouveau), `tests/action_bar_ui.rs` (nouveau), `tests/breadcrumb_ui.rs` (migration vers le helper), `.github/workflows/ci.yml` — vérification : `cargo test --test action_bar_ui && cargo test --test breadcrumb_ui`.

Chaque étape laisse le repo compilable et les tests verts. Quality gate complet en fin de tâche : `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build`.

## Risques et mitigations

- Segment sans `:` (chaîne d'aide future) → rendu tel quel en blanc (fallback du parsing) + test unitaire du fallback.
- Assertions existantes sur le texte d'aide → la jointure des segments trimmed par `" | "` reproduit exactement le `.trim()` des chaînes actuelles ; vérifié par `cargo test` complet à l'étape 3.
- Divergence de style entre le prompt de recherche des logs (hors périmètre) et la barre d'actions → assumée, le prompt est un champ de saisie, pas une barre d'actions.
- Couleurs réelles selon le terminal → les assertions portent sur le buffer `TestBackend`, pas sur le rendu matériel.

## Hors périmètre

- Les fichiers `actions.rs` (aucun affichage à changer).
- Les raccourcis clavier eux-mêmes (`keys.rs`, `AppAction`) et les chaînes de `resources.rs`.
- Le prompt de recherche des logs et le bloc titre du menu principal (findings 🟢 #4 de la review, à traiter séparément).
- La hauteur de la barre et le layout des écrans.
- Le dialog de confirmation et les popups d'erreur (pas de barre d'actions).
