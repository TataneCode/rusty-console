# Plan : Homogénéité des écrans Containers et Stacks > Containers

> Statut : implémenté — branche `vibe/stack-containers-homogeneity-bddf38`, PR #41 (draft).
> Baseline au moment du plan : 498 tests verts, `cargo clippy -- -D warnings` propre, release v3.1 (`b2acf9d`).

## Objectif

Rendre les deux écrans affichant des containers homogènes :

- **Écran Containers** (référence) ;
- **Stacks > Containers de la stack** (drill-down).

L’écran drill-down devait offrir les mêmes fonctionnalités que l’écran Containers, et les fonctionnalités liées aux stacks devaient être sur la liste des stacks.

## Constat initial (écarts)

| Fonctionnalité | Containers | Stacks > Containers (avant) |
|---|---|---|
| Colonnes | Name, Image, State, **CPU, Memory**, Status, Ports | Name, Image, State, Status, Ports |
| `l` Logs | ✅ | ❌ |
| `Enter` Détails | ✅ | ❌ |
| `p` Pause/Unpause | ✅ | ❌ |
| `R` Restart | ✅ | ❌ |
| `/` Filtre | ✅ | ❌ |
| `o` Tri | ✅ | ❌ |
| `X` Prune | ✅ | ❌ |
| `A` Ajouter un container | ✅ | ❌ |
| `e` Exec, `s` Start/Stop, `d` Delete, `r` Refresh | ✅ | ✅ |
| `S` Stop All / `Ctrl+S` Start All / `D` Remove All | ❌ (actions stack) | ✅ ← déplacées |
| Rendu | `render_list` (titre filtré, aide contextuelle, spinner) | `render_table` + barre d’aide manuelle |
| État liste | `ListCore<ContainerDto>` | `Vec` + `TableSelection` bruts |

## Décisions

- **D1** — Déplacer les actions stack sur la liste des stacks : `D` Remove All ajouté sur **Stacks** (avec confirmation) ; `S`, `Ctrl+S`, `D` retirés de Stacks > Containers.
- **D2** — `StackPresenter` embarque un second `ListCore<StackContainerDto>` (filtre nom/image, tri par nom) : filtre, tri et sélection survivent aux refreshs.
- **D3** — Rendu via le composant partagé `render_list`/`ListViewSpec` : mêmes 7 colonnes, titre `Stack: <name>` avec indicateur de filtre `[/: …]`, aide contextuelle en mode filtre, spinner de chargement.
- **D4** — Stats live : `StackContainerDto` gagne `runtime_stats: Option<ContainerRuntimeStatsDto>` + `cpu_display()`/`memory_display()` ; la souscription de stats existante (`refresh_container_stats_subscription`) couvre les containers de la stack affichée.
- **D5** — Actions container sur Stacks > Containers : `l` (logs via nouveau `ContainerActions::load_logs_by`), `Enter` (détails), `p`, `R`, `X`, `A`, `/`, `o`.
- **D6** — Retours contextuels : `logs_return_screen` / `details_return_screen` sur `App` ; `Esc` depuis logs/détails revient à Stacks > Containers quand on en vient.
- **D7** — Domaine : `StackContainerState` gagne `is_active`, `can_be_paused`, `can_be_unpaused`, `can_be_restarted` (miroir de `ContainerState`) ; `StackContainerDto` gagne les capacités correspondantes.
- **D8** — Décisions utilisateur retenues : stats live **oui** sur Stacks > Containers ; touche `A` **oui** (le container créé est standalone, les stacks viennent des labels Compose).

## Étapes réalisées

1. Domaine : `01_domain/stack/state.rs` — nouvelles capacités + tests.
2. Application : `02_application/stack/dto.rs` (+capacités, `runtime_stats`, affichage CPU/mémoire) et `mapper.rs`.
3. `ContainerActions::load_logs_by` + `App::load_container_logs_by`.
4. `StackPresenter` : `ListCore<StackContainerDto>` + cache de stats + tests.
5. `stack/view.rs` : `render_stack_containers` via `render_list` (7 colonnes) + tests de rendu.
6. `router.rs` : déplacement des actions stack, nouvelles actions container, câblage filtre, retours contextuels, loading par écran (`set_container_list_loading`).
7. `app.rs` : rendu, champs de retour, souscription stats, dialog d’ajout depuis la stack.
8. `resources.rs` : `STACK_LIST_HELP` (+`D: Remove All`), `STACK_CONTAINERS_HELP` (aide container complète), `stack_containers_title` sans padding (le padding vient de `filter_prompt_title`).
9. `MANUAL.md` : tableaux de touches des deux écrans.
10. Tests d’intégration : `tests/stack_containers_ui.rs` (7 scénarios, voir ci-dessous).

## Scénarios de tests

### Tests unitaires (500 au total, `cargo test --lib`)

| Scénario | Où | Couverture |
|---|---|---|
| Capacités `StackContainerState` | `01_domain/stack/state.rs` | `is_active`, `can_be_paused`, `can_be_unpaused`, `can_be_restarted` pour chaque état |
| Mapping DTO stack | `02_application/stack/mapper.rs` | capacités dérivées de l’état (running/stopped/paused), `runtime_stats: None` |
| `StackPresenter` containers | `stack/presenter.rs` | filtre nom/image, sélection bornée par le filtre, tri A→Z/Z→A, stats (apply/retain/clear), loading |
| Rendu stack containers | `stack/view.rs` | 7 colonnes + N/A, aide complète, aide contextuelle en mode filtre, spinner |
| Workflow actions StackContainers | `app.rs` (`test_stack_containers_action_matches_container_list_features`) | logs → retour `Esc`, détails → retour `Esc`, pause/restart/start-stop, filtre + `handle_filter_key`, tri, `A`, sortie d’écran (stop souscription stats, liste vidée) |
| Actions stack déplacées | `app.rs` (`test_toggle_pause_restart_and_stack_actions`) | `D` Remove All sur StackList ouvre la confirmation ; `S`/`Ctrl+S`/`D` inertes sur StackContainers |

### Tests d’intégration UI (`tests/`, sans daemon Docker)

| Fichier | Scénarios |
|---|---|
| `stack_containers_ui.rs` (7 tests) | layout 7 colonnes + aide identique à Containers ; actions stack réservées à la liste des stacks ; filtre contextuel (`[/: db]`, aide réduite, `Esc` efface) ; tri A→Z/Z→A ; `D` ouvre la confirmation sur Stacks ; inertie de `S`/`Ctrl+S`/`D` sur StackContainers ; spinner remplace l’aide |
| `action_bar_ui.rs` (2 tests) | barre d’aide de l’écran Containers (touches surlignées) ; menu principal |
| `breadcrumb_ui.rs` (3 tests) | fil d’Ariane sur menu, logs, liste containers |
| `integration_docker.rs` (6 tests) | bout en bout contre un vrai daemon Docker (ignorés si indisponible) |

## Vérification

- `cargo test` : 500 tests lib + 18 tests d’intégration UI, tous verts
- `cargo clippy --all-targets -- -D warnings` : aucun warning
- `cargo fmt --check` : OK

## Suivi post-v3.1

Voir `docs/TESTING.md` pour la matrice de couverture des scénarios par écran.
