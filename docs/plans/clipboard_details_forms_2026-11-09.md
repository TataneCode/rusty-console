# Plan : Presse-papiers — sélection native + `y` sur les écrans de détail, `Ctrl+Shift+C/X/V` dans les formulaires

> Statut : approuvé (scope minimal validé après challenge) — produit par l'agent Architecte (skill `rust-architecture-planning`), après challenge (skill `challenge`) qui a tranché : `Ctrl+C` reste Quitter partout (#48) ; `y` sur les détails copie le bloc entier (réseau : la ligne conteneur sélectionnée) ; dans les formulaires, copier/couper/coller passe par `Ctrl+Shift+C/X/V` sur le champ focalisé (`y`/`x`/`p` restent des lettres saisissables) ; sélection souris native limitée aux 4 écrans de détail.

## Objectif

Rendre le texte copiable sur les 4 écrans de détail (container, image, volume, réseau) via la sélection native du terminal (souris) **et** `y` / `Ctrl+Shift+C`, et permettre copier/couper/coller dans les champs de saisie (filtre `/`, recherche de logs, dialogue d'ajout) via `Ctrl+Shift+C` / `Ctrl+Shift+X` / `Ctrl+Shift+V`.

## Audit de l'existant

- Presse-papier : `common/clipboard.rs` n'avait que `copy_text` (`arboard` déjà dépendance, `Cargo.toml:18`, feature `wayland-data-control`) ; `y` ne copiait que la ligne de log (`CopyLogLine`, contexte `ContainerLogs`).
- `Ctrl+C` = Quit avant toute saisie (`app.rs` run_inner, #48) ; `contains(CONTROL)` attrape aussi `Ctrl+Shift+C`.
- Capture souris : `EnableMouseCapture` au démarrage (`app.rs`), mais aucun événement souris n'est jamais lu (`event.rs`) → sélection native impossible avant ce plan.
- Écrans de détail : pas de curseur de ligne (sauf détail réseau) ; les vues construisaient leur texte en inline.
- Champs de saisie : `handle_filter_key` / `handle_add_dialog_key` consomment toute lettre ; `TextInputState` / `FilterState` n'avaient ni `push_str` ni `clear_value`.

## Décisions d'architecture

- **D1 — `Ctrl+C` = Quit inchangé** (#48) ; nouvelles actions `AppAction::Copy`/`Cut`/`Paste` sur `Ctrl+Shift+C/X/V` (match `c`/`C`, `x`/`X`, `v`/`V` avec `CONTROL` ; Quit exige `CONTROL` sans `SHIFT`). En mode raw, les raccourcis clavier arrivent à l'app : les intercepter rend la copie fiable sur tous les terminaux ; la copie 100 % OS reste la sélection souris.
- **D2 — Nouveau `KeyContext::Details`** (ContainerDetails/VolumeDetails/ImageDetails) ; `NetworkDetails` mappe `CopyLogLine` → `Copy` ; `Global` et `ContainerLogs` filtrent `Copy`/`Cut`/`Paste` → `None`. `y` mappe toujours sur `CopyLogLine`, traduit en `Copy` dans les contextes détails.
- **D3 — Cible de `y`** : builders purs extraits des vues (`container_details_text`, `volume_details_text`, `image_details_text`, `network_details_text`), réutilisés par le rendu et la copie ; réseau : `network_container_line` du conteneur sélectionné, sinon bloc entier.
- **D4 — Saisie** : `handle_input_clipboard_key(KeyEvent)` dans `run_inner` avant les handlers de saisie ; copie = champ entier, couper = copier + vider, coller = `read_text` + `push_str` en fin. En mode saisie, seul un échec lève un popup (un popup de succès avalerait la prochaine touche de l'input ouvert).
- **D5 — Presse-papier en couche présentation** (`clipboard.rs` += `read_text`), comme décidé pour `CopyLogLine` ; aucune nouvelle dépendance.
- **D6 — Sélection native** : `sync_mouse_capture` appelé à chaque itération de `run_inner` (état désiré = « pas un écran de détail » vs `mouse_capture_enabled` mémorisé) ; répare aussi l'état après `resume_tui` (exec).
- **D7 — Feedback** : popups `PopupMessage` existants ; messages génériques `clipboard_copy_success/error_message` + `clipboard_paste_error_message` (`resources.rs`) ; aide des 4 détails += `y: Copy`.

## Étapes

1. `common/clipboard.rs` : `read_text()` ; `common/mod.rs` : ré-export.
2. `common/input.rs` / `common/filter.rs` : `push_str`, `clear_value` (+ UT).
3. `common/keys.rs` : actions `Copy`/`Cut`/`Paste`, mapping `Ctrl+Shift+C/X/V`, Quit restreint, `KeyContext::Details`, filtres de contexte (+ UT).
4. Vues container/volume/image/network : extraire `*_details_text` (+ `network_container_line`), réutilisés par le rendu (+ UT) ; ré-exports `mod.rs`.
5. `app.rs` : branche `handle_input_clipboard_key` dans `run_inner`, contexte `Details`, champ `mouse_capture_enabled`, `sync_mouse_capture` dans la boucle, `resume_tui` met à jour le flag (+ UT).
6. `router.rs` : branche `Copy` dans les 3 handlers de détail ; `copy_to_clipboard`, `active_filter_mut`, `handle_input_clipboard_key` ; `handle_add_dialog_key` rendu `pub` (convention IT, comme `handle_filter_key`) (+ UT).
7. `resources.rs` : messages clipboard + aides des 4 détails (+ UT).
8. IT `tests/clipboard_details_ui.rs` (7 tests) + entrée matrice `docs/TESTING.md` + `MANUAL.md`.
9. Quality gate : `cargo fmt` → `cargo clippy -- -D warnings` → `cargo test`.

## Risques et mitigations

- Terminaux rapportant `Ctrl+Shift+C` comme `C` + `CONTROL` (Windows) : match aussi la variante majuscule.
- `arboard` indisponible (headless/SSH) : popup `Error`, jamais de panic (règle presentation).
- Oubli de resync de la capture souris après l'exec : `sync_mouse_capture` idempotent, rappelé à chaque itération.
- Popup d'erreur pendant un filtre actif : dismiss au prochain raccourci mappé (comportement popup existant).

## Hors périmètre

Sélection native sur tous les écrans ; curseur de ligne `j/k` sur les détails ; édition intra-champ avec curseur/sélection ; `Ctrl+C` = copier ; `y`/`x`/`p` réservés en saisie ; copie multi-lignes ; sélection in-app à la souris.
