# Plan : `y` copie uniquement la sélection souris in-app sur les détails

> Statut : approuvé — demande utilisateur (2026-11-09) : « `y` ne doit copier que le texte sélectionné » (la sélection native du terminal n'étant pas lisible par l'app, la sélection devient in-app sur les 4 écrans de détail).

## Objectif

Sur les écrans de détail (container, volume, image, network) : sélectionner du texte à la souris (drag), la sélection est surlignée, et `y` / `Ctrl+Shift+C` ne copient **que cette sélection**. Sans sélection : no-op (sauf détail réseau : fallback sur la ligne conteneur sélectionnée avec `j`/`k`).

## Contrainte actée

L'app ne peut pas lire la sélection souris native du terminal → la sélection est gérée par l'app (capture souris maintenue partout, comme avant la PR). La sélection native du terminal ne marche donc plus sur les détails.

## Changements

- `event.rs` : `AppEvent::Mouse(MouseEvent)`.
- `app.rs` : plus de bascule de capture souris (`sync_mouse_capture` supprimé, retour à `EnableMouseCapture` permanent) ; état `details_selection: Option<((u16,u16),(u16,u16))>` + `details_selection_text`; `render()` surligne la sélection (REVERSED) dans la zone interne du bloc et extrait le texte depuis le buffer (lignes jointes par `\n`, fin de ligne tronquée) ; la sélection est effacée hors des détails.
- `router.rs` : `handle_details_mouse` (Down = ancrage, Drag = extension, ignoré ailleurs/avec overlay) ; `copy_details_selection` ; `Copy` sur les détails copie la sélection (fallback ligne conteneur sur le détail réseau).
- `resources.rs` : aides des 4 détails → `drag: Select | y: Copy selection`.
- Docs : `MANUAL.md`, matrice `docs/TESTING.md`.

## Tests

UT (`app.rs`) : état de sélection via événements souris (down/drag, ignoré hors détails et avec overlay), extraction + surlignage au rendu, no-op du `Copy` sans sélection. IT (`tests/clipboard_details_ui.rs`) : extraction via rendu réel, `Copy` avec sélection → popup, fallback réseau. Gate : `cargo fmt`, `clippy -D warnings`, `cargo test`.

## Hors périmètre

Sélection native du terminal sur les détails (remplacée), sélection multi-blocs, copier par clic droit in-app, sélection sur les autres écrans.
