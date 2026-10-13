# Plan : Breadcrumb de navigation

Date : 2026-10-13 — Branche : `breadcrumb`

## Objectif

Afficher un fil d'Ariane (breadcrumb) dans la zone de titre de l'application pour indiquer à tout moment l'écran courant et son chemin de navigation, ex. `Rusty - docker manager > Containers > web logs`.

## Décisions d'architecture

- **D1 : Composant 100 % présentation** — le breadcrumb est un état d'affichage dérivé de `Screen` + présenters ; il n'apporte aucune donnée métier. Aucune couche domain/application/infrastructure n'est modifiée. Raison : règle de placement (couche la plus interne concernée = présentation).
- **D2 : Rendu au niveau `App::render`, pas dans chaque vue** — les segments sont calculés dans `app.rs` (état de navigation), le rendu est délégué à un widget réutilisable `common/breadcrumb.rs`. Chaque `render_*` reçoit une `area` réduite ; aucune signature de vue existante ne change. Raison : un seul point d'intégration, pas de diff sur 12 vues.
- **D3 : `MainMenu` conserve sa zone de titre existante** (`split_menu_area`) qui devient la racine du breadcrumb (segment unique = titre applicatif). Raison : le menu EST la racine ; une barre supplémentaire dupliquerait l'information.
- **D4 : Nouvelle couleur de bordure** — `Theme::breadcrumb_border_style()` (Cyan) distincte de `Theme::border_style()` (Blanc) utilisée par les sections principales. Le texte garde `Theme::title_style()`. Raison : demande explicite, cohérence avec `selected_border_style` existant.
- **D5 : Renommage du titre** — `MAIN_MENU_TITLE` devient `APP_TITLE = "Rusty - docker manager"` (valeur demandée) ; c'est la racine du breadcrumb. Tous les sites d'appel sont mis à jour (menu.rs, tests). Raison : le titre est applicatif, pas propre au menu.
- **D6 : Troncature du breadcrumb** — réutilisation de `truncate_text` sur la largeur de la zone. Raison : un nom de conteneur long ne doit pas casser le layout.

## Étapes

1. **Titre applicatif** — fichiers : `resources.rs`, `common/menu.rs`, `app.rs` (test) — remplacer `MAIN_MENU_TITLE` par `APP_TITLE = "Rusty - docker manager"` — vérification : `cargo test render_main_menu`.
2. **Widget breadcrumb** — fichiers : `common/breadcrumb.rs` (nouveau), `common/theme.rs`, `common/mod.rs` — `breadcrumb_text(&[&str]) -> String` (joint par `" > "`), `render_breadcrumb(frame, area, segments)`, `BREADCRUMB_BAR_HEIGHT = 3`, `Theme::breadcrumb_border_style()` — vérification : `cargo test breadcrumb`.
3. **Intégration dans `App::render`** — fichiers : `app.rs`, `common/widgets.rs` (`split_app_area`) — barre de breadcrumb en haut de chaque écran sauf `MainMenu` ; segments par écran (`Containers > <name> logs`, `Volumes > <name> details`, etc.) — vérification : `cargo test`.
4. **Tests d'intégration CI** — fichiers : `tests/breadcrumb_ui.rs` (nouveau), `app.rs` (rendre `render` public), `.github/workflows/ci.yml` — câblage complet de la pile DI (DockerClient lazy), `Screen::ContainerLogs`, rendu sur `TestBackend`, assertion du texte du breadcrumb — vérification : `cargo test --test breadcrumb_ui`.

Chaque étape laisse le repo compilable et les tests verts.

## Tests

- **Unitaires** : `breadcrumb.rs` (construction du texte, troncature, séparateur), `theme.rs` (nouveau style), `menu.rs`/`app.rs` (titre mis à jour, breadcrumb affiché par écran, absent sur MainMenu).
- **Intégration** : `tests/breadcrumb_ui.rs` — pile complète main.rs-like (client Docker lazy, aucun appel réseau), navigation jusqu'aux logs d'un conteneur, rendu buffer, assertions sur `Rusty - docker manager > Containers > <name> logs`. Ajouté au job `integration` de la CI.

## Risques et mitigations

- Nom de conteneur long → troncature via `truncate_text` (D6).
- `DockerClient::new()` indisponible dans un environnement sans Docker → skip gracieux du test d'intégration (même pattern que `tests/integration_docker.rs`).
- Régression visuelle sur les écrans existants → hauteur de barre fixe (3) identique à `MAIN_MENU_TITLE_HEIGHT` ; les vues reçoivent une area réduite, aucun calcul interne de layout modifié.

## Hors périmètre

- Navigation cliquable sur le breadcrumb (les segments ne sont pas interactifs).
- Breadcrumb dans les popups/dialogs (confirm, error, exec shell).
- Nouvelle dépendance Cargo.
