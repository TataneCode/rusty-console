# Plan : Surlignage de la ligne de log sélectionnée + copie presse-papiers (`y`)

> Statut : approuvé — produit par l'agent Architecte (skill `rust-architecture-planning`), après challenge (skill `challenge`) qui a validé le périmètre minimal : surligner la ligne sélectionnée sur l'écran `ContainerLogs`, copier la ligne affichée (timestamp + message) dans le presse-papiers cross-platform via `y`, **pas de sélection multiple**.

## Objectif

Rendre la ligne de log sélectionnée visible (surlignage, comme les autres listes du projet) et permettre de la copier dans le presse-papiers système en une touche, depuis l'écran des logs uniquement.

## Audit de l'existant

- Sélection ligne : déjà en place — `ContainerPresenter::logs_state` (`container/presenter.rs:17`), navigation `j/k`, `PgUp/PgDn`, recherche `n/N`. Rien à ré-implémenter.
- Surlignage : le `List` des logs a `highlight_symbol("> ")` mais **pas** `highlight_style` (`container/view.rs:124`) — écart unique vs les autres listes (`common/widgets.rs:270`).
- Matchs de recherche : déjà surlignés via `Theme::highlight_style()` (`view.rs:107`).
- Copie : aucune capacité presse-papiers dans le projet.
- Touche `y` : libre partout (vérifié). Convention vi déjà adoptée (`j/k`, `/`, `n/N`) → `y` (yank).
- Feedback : pattern popup existant — `PopupMessage::Info/Error` + ressources `logs_export_*_message`, handler `ExportLogs` (`router.rs:216`), fermeture du popup à l'action suivante (`router.rs:23`).
- Format ligne affichée : `LogLineDto::timestamp_display()` existe (`dto.rs:79`).

## Décisions d'architecture

- **D1 — Surlignage sélection** : ajouter `.highlight_style(Theme::highlight_style())` au `List` des logs. Cohérence avec toutes les autres listes du projet. Le conflit visuel avec le surlignage des matchs (même style) est accepté : ratatui applique le style de sélection sur la ligne entière, la sélection reste prioritaire.
- **D2 — Copie presse-papiers : pas de couche DDD.** Le presse-papiers n'est pas une source de données métier (pas d'invariant, pas d'état partagé, pas de mock pertinent). La copie est un port OS de la présentation, comme `exec.rs` spawne déjà un process externe directement. Implémentation dans un petit module `src/04_presentation/tui/common/clipboard.rs` (`copy_text(&str) -> Result<(), String>`), appelé depuis le handler.
- **D3 — Dépendance `arboard`** : `arboard = { version = "3", default-features = false, features = ["wayland-data-control"] }`. Cross-platform natif (X11 via `x11rb`, Wayland via `wl-clipboard-rs`, macOS, Windows) ; `rust_version` 1.71 ≤ MSRV 1.88 ; sans la feature `image` (économie ~30 crates) ; `wayland-data-control` couvre Wayland sans serveur de clipboard externe. Vérifié par `cargo check` + `cargo tree` dans un bac à sable.
- **D4 — Touche `y`** : nouvelle `AppAction::CopyLogLine`, mappée sur `KeyCode::Char('y')` dans `keys.rs`, restreinte au contexte `ContainerLogs` (même pattern que `ExportLogs`/`n`/`N` dans `KeyContext::Global => None`).
- **D5 — Format copié** : la ligne **telle qu'affichée** — `"{timestamp_display} {message}"` si timestamp, `"{message}"` sinon — via une méthode `display_line()` ajoutée sur `LogLineDto` (couche application, testable sans UI).
- **D6 — Feedback** : succès → `PopupMessage::Info("Log line copied…")`, échec (headless/SSH sans display) → `PopupMessage::Error(...)`. Ressources `logs_copy_success_message` / `logs_copy_error_message` (pattern `logs_export_*`). Le popup se ferme à l'action suivante (comportement existant).
- **D7 — Handler** : branche `AppAction::CopyLogLine` dans `handle_container_logs_action` (`router.rs`) — lit `selected_logs_line()`, construit le texte, appelle `clipboard::copy_text`, popup. Synchrone (arboard est synchrone, latence de quelques ms acceptable pour une action clavier).
- **D8 — Pas de sélection multiple** : une seule ligne (`logs_state.selected()`), explicitement hors périmètre.

## Étapes

1. **Dépendance** — `Cargo.toml` + `Cargo.lock` : ajouter `arboard` (features minimales, commentaire `# Clipboard` regroupé par rôle). Vérification : `cargo check`, `cargo tree -e features` (aucune feature existante élargie).
2. **DTO** — `src/02_application/container/dto.rs` : `LogLineDto::display_line()`. UT co-localisés : avec timestamp, sans timestamp. Vérification : `cargo test dto`.
3. **Module clipboard** — nouveau `src/04_presentation/tui/common/clipboard.rs` + export `common/mod.rs`. Encapsule `arboard::Clipboard::new()` + `set_text`, erreur mappée en `String`. Vérification : `cargo check`.
4. **Surlignage** — `src/04_presentation/tui/container/view.rs` : `.highlight_style(Theme::highlight_style())` sur le `List` des logs. UT `TestBackend` : ligne sélectionnée → cellules au style highlight. Vérification : `cargo test container::view`.
5. **Key mapping** — `src/04_presentation/tui/common/keys.rs` : `AppAction::CopyLogLine`, `Char('y')`, restriction `Global => None`. UT : `y` → `CopyLogLine` en contexte logs, `None` ailleurs.
6. **Handler + feedback** — `router.rs` (`handle_container_logs_action`) + `resources.rs` (`logs_copy_success_message` / `logs_copy_error_message`). Vérification : `cargo test router` + `cargo check`.
7. **Aide + docs** — `CONTAINER_LOGS_HELP` : ajouter `y: Copy`. `MANUAL.md` section « Log view » (à ce jour quasi vide) : documenter `y`, le surlignage, et les touches existantes manquantes (`j/k`, `PgUp/PgDn`, `Enter`, `/`, `n/N`, `E`). `docs/TESTING.md` : scénario ajouté à la matrice.
8. **IT** — `tests/` : build_app, injecter des logs dans le presenter, `Screen::ContainerLogs`, `handle_action(AppAction::CopyLogLine)`, assert qu'un popup est affiché (Info **ou** Error selon le display disponible — l'IT ne dépend pas de l'environnement). Vérification : `cargo test --test <suite>`.
9. **Quality gate** — `cargo fmt && cargo clippy -- -D warnings && cargo test`.

Chaque étape laisse le repo compilable et les tests verts (une étape = un commit potentiel).

## Risques et mitigations

- **MSRV** : arboard 3.7.0 (`rust_version` 1.71) et sa fermeture wayland (max 1.86) ≤ 1.88. Vérifié. Si régression future : épingler une version mineure inférieure.
- **CI headless** : l'IT accepte Info ou Error (`popup_message.is_some()`), jamais de dépendance stricte au succès clipboard.
- **Conflit visuel sélection/match** : une ligne sélectionnée et matchante → le style de sélection (ligne entière) prime. Accepté, cohérent avec les autres listes.
- **Blocage event loop** : `set_text` synchrone, quelques ms (connexion X11/Wayland) — acceptable pour une action clavier ponctuelle ; pas de `spawn_blocking`.
- **Wayland** : `wayland-data-control` permet l'écriture dans le presse-papiers sans serveur externe.

## Hors périmètre

- Sélection multiple / copie de plusieurs lignes (explicitement exclu par la demande).
- Copie depuis la vue étendue (overlay `Enter`).
- Raccourcis alternatifs (`c`, `Ctrl+C`), copie du timestamp seul ou du message seul.
