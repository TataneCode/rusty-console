# Règles — Couche Presentation (`src/04_presentation/`)

## Périmètre

Rendu Ratatui, gestion des touches, state machine des écrans, `main.rs` (wiring DI).

## Règles

1. **Triade par domaine** : `presenter.rs` (état d'affichage mutable : sélection, scroll, données chargées), `view.rs` (fonctions de rendu pures, prennent des références, produisent des widgets), `actions.rs` (wrappers async fins autour des services application).
2. **L'UI ne voit que des DTO** — jamais d'entité domain directe ; les actions consomment les services application.
3. **State machine** : `ui/app.rs` possède un enum `Screen` qui pilote rendu et touches. Les états overlay (`confirm_dialog`, `error_message`) sont évalués **avant** les handlers d'écran.
4. **Mapping touches** : `ui/common/keys.rs` mappe les `KeyEvent` bruts vers des variantes `AppAction`. Pas de `match` sur KeyCode éparpillé.
5. **Erreurs : catch `AppError`** → `App::error_message` pour affichage popup. Jamais de panic côté UI.
6. **Loading state** — tous les presenters ont `loading: bool` + `set_loading/is_loading` ; les vues acceptent `is_loading: bool` et `spinner_char: &str` ; le texte d'aide est **remplacé** (pas appended) par « Loading... <spinner> » pendant le chargement ; l'exec shell nettoie l'état loading avant de spawner le process externe.
7. **Spinner** — `ui/common/spinner.rs`, animation ASCII `|/-\`, 200 ms par frame.
8. **Pas de logique métier** — un presenter gère de l'affichage ; les décisions métier vivent en application/domain.

## Anti-patterns interdits

- Importer `01_domain` directement ou `03_infrastructure` (toujours via l'application).
- I/O direct (appel Bollard ou std::fs) dans une vue ou un presenter.
- Vues avec état — `view.rs` reste des fonctions pures.
- Gestion des touches hors de `keys.rs`/handlers dédiés.
