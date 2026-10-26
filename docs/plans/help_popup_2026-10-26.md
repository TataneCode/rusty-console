# Plan : Aide contextuelle — footer réduit + pop-up Espace (style LazyVim)

Date : 2026-10-26 — Branche : `vibe/help-popup-space-15e8ab`

## Objectif

Remplacer l'aide contextuelle « tout en bas de l'écran » par deux niveaux :

1. **Footer réduit** — en bas de chaque écran, seules les commandes fondamentales :
   `j/k: Navigate | Enter: Details | /: Filter | r: Refresh | q: Quit | Space: All commands`
   (`Enter: Drill-down` sur la liste des stacks).
2. **Pop-up ` Keybindings `** — la touche **Espace** ouvre une pop-up centrée listant
   **toutes** les commandes de l'écran courant, une par ligne, même style que la barre
   (touches jaune gras, libellés blancs). Fermeture par `Esc` / `q` / `Espace`.

Transverse : listes Containers, Volumes, Images, Networks, Stacks (+ Stack Containers),
échens de détails, logs, menu principal.

## Décisions d'architecture

- **D1 : Pop-up générique alimentée par les constantes existantes** — nouveau module
  `common/help_popup.rs` avec `render_help_popup(frame, help_text)` : prend la chaîne
  `*_HELP` complète de l'écran en entrée et la découpe en une ligne stylée par commande.
  Raison : la liste complète est déjà définie par écran dans `resources.rs` ; aucune
  donnée déplacée ni dupliquée.
- **D2 : Style partagé, pas de duplication** — `action_bar.rs` expose `help_segment_spans`
  (un segment `touche: Libellé` → spans stylés), utilisé à la fois par la barre et la
  pop-up. Raison : cohérence visuelle garantie, un seul endroit pour le style.
- **D3 : Footer réduit factorisé dans `resources.rs`** — `list_footer_help(enter_label)`
  produit la chaîne du footer ; `render_list` (`common/list_view.rs`) l'utilise à la place
  de `spec.help_text`, qui disparaît du `ListViewSpec`. Raison : les 6 vues de liste
  (container, volume, image, network, stack ×2) partagent déjà `render_list` ; un seul
  point de changement.
- **D4 : État `help_popup_open` + dispatch transverse dans `router.rs`** —
  `AppAction::ToggleHelp` (touche Espace, `common/keys.rs`) ; `handle_action` gère la
  pop-up avant le dispatch écran : ouverte → seules `ToggleHelp`/`Back`/`Quit` ferment,
  tout le reste est ignoré (modale) ; fermée → `ToggleHelp` ouvre. Placé **après** les
  autres modaux (confirm, exec, add dialog) pour ne jamais s'ouvrir par-dessus.
  `current_help_text()` mappe l'écran courant vers sa constante `*_HELP`.
  Raison : un seul point de branchement, transverse à tous les écrans.
- **D5 : Espace est libre** — vérifié : aucun `KeyCode::Char(' ')` dans le mapping
  clavier. Le filtre `/`, la recherche de logs et les dialogues de saisie capturent les
  caractères avant le mapping d'actions, donc aucun conflit (la pop-up ne s'ouvre pas
  pendant la saisie).
- **D6 : Hint de discoverability** — `Space: All commands` ajouté aux footers déjà
  courts (`*_DETAILS_HELP`, `CONTAINER_LOGS_HELP`, `MAIN_MENU_HELP`) ; ces écrans
  ouvrent aussi la pop-up sur Espace.
- **D7 : `has_modal_overlay` rendu `pub` et couvrant la pop-up** — la boucle
  d'événements bloque le filtre pendant que la pop-up est ouverte, et le rendu assombrit
  l'arrière-plan comme les autres overlays. La pop-up est rendue **sous** les autres
  overlays (confirm, exec, add) : elle ne peut pas être ouverte en même temps qu'eux.
- **D8 : Tests** — UT co-localisés (`keys.rs`, `resources.rs`, `help_popup.rs`,
  `list_view.rs`, `app.rs`) + nouvelle suite IT `tests/help_popup_ui.rs` (footer réduit,
  ouverture/fermeture, modale, non-ouverture au-dessus d'un autre modal, écran de
  détails, scroll sur terminal court). `tests/action_bar_ui.rs` et
  `tests/stack_containers_ui.rs` mis à jour sur le footer réduit. Entrées ajoutées à la
  matrice de `docs/TESTING.md` ; `MANUAL.md` documente la touche Espace.
- **D9 : Popup scrollable (review PR #44)** — quand la liste de commandes dépasse la
  hauteur utile (80 % du terminal − bordures − barre de fermeture), la popup scrolle
  avec `j`/`k`/↑/↓ (1 ligne) et PgUp/PgDn (10 lignes), avec une scrollbar verticale à
  droite (`ScrollbarOrientation::VerticalRight`, n'apparaît que si scrollable). L'état
  `HelpPopupState` (scroll, content_length, viewport_length) est mis à jour à chaque
  rendu et clampé ; il est remis à zéro à l'ouverture. Raison : sur un terminal < ~22
  lignes, les 14 commandes de `CONTAINER_LIST_HELP` dépassaient la hauteur utile et
  ratatui tronquait silencieusement sans indication.

## Étapes réalisées

1. **resources.rs** — `HELP_POPUP_TITLE`, `HELP_POPUP_CLOSE_HELP`, `list_footer_help`,
   hint `Space: All commands` sur les footers détails/logs/menu + UT.
2. **action_bar.rs** — extraction de `help_segment_spans` (partagé barre/pop-up).
3. **help_popup.rs** (nouveau) — `help_popup_lines` + `render_help_popup` + UT.
4. **keys.rs** — `AppAction::ToggleHelp` + mapping `KeyCode::Char(' ')` + UT.
5. **list_view.rs + 6 vues de liste** — footer réduit via `list_footer_help`,
   suppression de `ListViewSpec.help_text` + UT.
6. **app.rs / router.rs** — état `help_popup_open`, `current_help_text`, dispatch
   transverse, rendu pop-up + overlay, `has_modal_overlay` pub + UT.
7. **Tests IT** — `tests/help_popup_ui.rs` (nouveau, 6 tests), mise à jour
   `action_bar_ui.rs` / `stack_containers_ui.rs`.
8. **Docs** — `MANUAL.md` (touche Espace + pop-up + scroll), `docs/TESTING.md` (matrice).
9. **Scroll (review)** — `HelpPopupState` + `Paragraph::scroll` + `Scrollbar` vertical,
   touches `j`/`k`/↑/↓/PgUp/PgDn dans la popup, UT (bornes de scroll) + IT (scroll sur
   terminal 60x12).

## Vérification

```bash
cargo fmt --check          # OK
cargo clippy --all-targets -- -D warnings   # OK
cargo test                 # 523 UT + 28 IT, 0 failed
```
