# Plan : Refonte de la gestion des logs (écran ContainerLogs)

> Statut : approuvé pour implémentation — produite par l'agent Architecte (skill `rust-architecture-planning`), consommable telle quelle par l'agent Développeur.
> Baseline au moment du plan : 297 tests verts, `cargo clippy -- -D warnings` propre.

## Objectif

Offrir sur l'écran des logs une navigation ligne par ligne horodatée, l'affichage complet d'une ligne tronquée (wrapping), l'export `.log` du conteneur via une touche, et une recherche dans les logs façon vi (`/` + `n`).

## Analyse de référence — oxker (mrjackwills/oxker)

| Mécanisme oxker | Implémentation observée | Transposition rusty-console |
|---|---|---|
| Navigation ligne + timestamp | `List` stateful + `ListState` (`highlight_symbol`, scroll padding) ; timestamps natifs via `LogsOptions { timestamps: true }` bollard | ✅ `List`/`ListState` ratatui, déjà utilisé ailleurs dans le projet |
| Ligne tronquée | scroll **horizontal** par découpage de chars (`format_log_line` + `offset`) — pas de wrapping | ✅ Remplacé par une vue « ligne étendue » en overlay avec `Paragraph::wrap` |
| Export `.log` | `save_logs` : `{container_name}_{epoch}.log`, ré-interroge Docker avec flux complet | ✅ Identique, avec timestamp lisible `chrono` `<name>_<YYYYMMDD_HHMMSS>.log` |
| Recherche (`LogSearch`) | `search_term: Option<String>` + `search_results: Vec<usize>` (indices des lignes matchantes), `search_scroll(Down/Up)` navigue entre résultats en tenant compte de la sélection courante | ✅ Adapté en mode vi : `/` ouvre une barre de saisie, `Enter` valide, `n` résultat suivant, `N` précédent |

Écart assumé avec oxker : oxker stocke les logs en `String` brute et découpe à l'affichage (déduplication par timestamp via `HashSet`). Pour rusty-console, la couche application parsera en `LogLineDto { timestamp, message }` — plus propre et aligné sur notre chaîne DTO, et testable sans Docker.

## Décisions d'architecture

- **D1** — Activer `timestamps: true` dans `LogsOptions` (`src/03_infrastructure/docker/container/adapter.rs:80`) ; `get_logs` retourne le texte brut, le parsing timestamp/message se fait dans un mapper dédié de la couche application (`02_application`). *Raison : respect du flux DTO, testabilité du parsing sans démon Docker.*
- **D2** — Le presenter possède un `ListState` (sélection courante) au lieu du simple `logs_scroll: u16` actuel : la sélection ligne survit au redimensionnement. *Raison : mécanisme idiomatique ratatui, cohérent avec les autres écrans.*
- **D3** — La vue « ligne étendue » est un **overlay** réutilisant le pattern `confirm_dialog` (état overlay dans `App`, traité avant les handlers d'écran). *Raison : moindre coût, pas de câblage `Screen` supplémentaire.*
- **D4** — L'export interroge Docker avec `tail: None` (flux complet) et écrit via `tokio::fs` dans le répertoire courant ; nom `<container_name>_<YYYYMMDD_HHMMSS>.log` (chrono, déjà en dépendance). *Raison : fidèle au comportement oxker (fichier complet), timestamp lisible plutôt qu'epoch.* ⚠️ Vérifier la feature `fs` de tokio (`Cargo.toml` : features actuelles `rt-multi-thread, macros, sync, time, process`) — l'ajouter relève du skill `rust-dependency` si absent.
- **D5** — Nouvelle `AppAction::ExportLogs` mappée sur `E` (écran logs uniquement) ; `E` est libre (`e` = Exec). Feedback succès/échec via la barre d'état / `App::error_message`.
- **D6** — Recherche en overlay façon vi : `/` ouvre une barre de saisie (réutilise `FilterState` de `common/filter.rs` étendu), `Enter` valide et calcule `Vec<usize>` d'indices matchants (recherche insensible à la casse), `n` va au résultat suivant, `N` au précédent (wrap-around aux extrémités), `Esc` ferme la saisie sans valider, `/` à nouveau réinitialise le terme. La recherche **saute** vers les lignes matchantes (elle ne filtre pas) ; le terme actif s'affiche dans le titre de l'écran avec le compteur `match idx/total`. Les lignes matchantes sont surlignées dans le `List`. *Raison : pattern vi standard (`less`), réutilise `search_scroll` d'oxker comme algorithme de navigation (position courante dans la liste des résultats), et le pattern d'état overlay existant plutôt qu'un écran dédié.*

## Étapes

1. **Application : DTO et parsing** — fichiers : `src/02_application/container/dto.rs`, `mapper.rs` (ou nouveau `log_parser.rs`), `traits.rs`, `service.rs`. `ContainerLogsDto` devient `{ container_name, lines: Vec<LogLineDto> }` avec `LogLineDto { timestamp: Option<DateTime<Utc>>, message: String }` ; parsing `"<ISO8601> <message>"` tolérant aux lignes sans timestamp. Vérification : `cargo test` (tests du parser : avec/sans timestamp, ligne vide, format bollard).
2. **Infrastructure : timestamps** — fichiers : `src/03_infrastructure/docker/container/adapter.rs`. `timestamps: true` dans `LogsOptions`. Vérification : `cargo check`.
3. **Presentation : navigation ligne** — fichiers : `src/04_presentation/tui/container/{presenter.rs, view.rs, actions.rs, app.rs}`, `common/keys.rs`. Presenter : `ListState` + `next/previous_logs_line` (`j/k`/flèches, PageUp/PageDown, bornes) ; view : `List` stateful, symbole de sélection, colonne timestamp grisée + message. Vérification : tests presenter + tests buffer `view.rs` (`TestBackend`).
4. **Presentation : ligne étendue (wrapping)** — fichiers : `app.rs` (état overlay `expanded_log_line: Option<usize>`), `view.rs` (overlay `Paragraph::wrap`, fermeture `Esc`/`Back`). Vérification : tests overlay `app.rs` + buffer test.
5. **Recherche façon vi** — fichiers : `common/filter.rs` (réutilisation/extension), `container/presenter.rs` (terme actif, `search_results: Vec<usize>`, `next_match`/`previous_match` avec wrap-around, `active_search_value()`), `container/view.rs` (barre de saisie `/` en bas, surlignage des matchs, compteur `idx/total` dans le titre), `common/keys.rs` (`/` sur l'écran logs → `ActivateLogSearch`, `n`/`N` → `NextLogMatch`/`PreviousLogMatch`, saisie filtrée quand l'overlay actif), `app.rs` (routage : overlay recherche traité avant les actions d'écran, `Enter` valide, `Esc` ferme). Vérification : tests presenter (navigation résultats, wrap-around, sélection hors résultats), tests buffer (barre de saisie rendue, surlignage).
6. **Export .log** — fichiers : `traits.rs`/`service.rs` (`export_logs(container_name, full_log) -> Result<PathBuf, AppError>`), `adapter.rs` (réutilisation de `get_logs` avec `tail: None`), `actions.rs`, `keys.rs` (`E`), `app.rs` (feedback). Vérification : tests service avec mock repository + test formatage du nom de fichier.
7. **Quality gate complet** — `cargo fmt && cargo clippy -- -D warnings && cargo test`. Le repo reste vert à chaque étape intermédiaire (une étape = un commit).

## Risques et mitigations

- **Lignes de logs très volumineuses** (stacktraces JSON mono-ligne) → la vue étendue (étape 4) répond à ce cas ; le `List` reste léger (une ligne par item).
- **Changement de format DTO** (`String` → `Vec<LogLineDto>`) impacte `service.rs`, `app.rs`, `view.rs` et leurs tests → étape 1 isolée en premier commit, tests adaptés mécaniquement dans la même étape (changement d'API annoncé, pas de contournement).
- **Performance sur conteneurs bavards** → conserver `tail: 500` pour l'affichage ; seul l'export prend le flux complet.
- **Conflit de touches** : `/` est déjà `ActivateFilter` sur les écrans de listes — le mapping doit être **par écran** (sur l'écran logs, `/` = recherche logs). `n`/`N` sont libres sur tous les écrans ; restreindre leur effet à l'écran logs.
- **Répertoire courant non inscriptible** → erreur mappée en `AppError`, affichée via `error_message`.

## Hors périmètre

- Streaming temps réel des logs (follow) — oxker le fait par polling ; plan séparé si souhaité.
- Scroll horizontal à la oxker — remplacé par la vue étendue avec wrapping.
- Filtrage (masquage des lignes non matchantes) — la recherche saute, elle ne filtre pas.
- Configuration d'un répertoire d'export (oxker : `dir_save`).
- Regex dans la recherche — recherche textuelle insensible à la casse dans ce plan.
