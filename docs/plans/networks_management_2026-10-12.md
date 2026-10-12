# Plan : Gestion des Networks (écran NetworkList + NetworkDetails)

> Statut : proposé à l'utilisateur — produit par l'agent Architecte (skill `rust-architecture-planning`), consommable tel quel par l'agent Développeur.
> Baseline au moment du plan : 327 tests verts, `cargo clippy -- -D warnings` propre (MSRV 1.88).

## Objectif

Ajouter la gestion des réseaux Docker à rusty-console en reproduisant le pattern DDD complet de l'area `volume` (domain → application → infrastructure → presentation), conformément à la Phase 4 d'`EVOLUTION.md` : liste filtrable, détails (driver, scope, subnet, gateway, containers connectés), déconnexion d'un container (`d`), suppression avec confirmation, prune des réseaux inutilisés.

## Analyse de l'existant

L'area `volume` est le modèle de référence le plus proche : liste + détails légers + delete + prune, sans stats temps réel ni exec.

| Couche | Référence volume | Transposition network |
|---|---|---|
| `01_domain/volume/{entity,value_objects,mod}.rs` | `Volume` + `VolumeId` (`DomainError::InvalidVolume`) | `Network` + `NetworkId` (`DomainError::InvalidNetwork`) |
| `02_application/volume/{traits,dto,mapper,service}.rs` | `VolumeRepository` (mockall), `VolumeDto`, service stateless | `NetworkRepository`, `NetworkDto`, `NetworkService` |
| `03_infrastructure/docker/volume/{adapter,mapper,mod}.rs` | `VolumeAdapter` + `VolumeInfraMapper` | `NetworkAdapter` + `NetworkInfraMapper` |
| `04_presentation/tui/volume/{presenter,view,actions}.rs` | triade + `FilterState` + `TableSelection` | triade `network/` à l'identique |

API Bollard 0.21 confirmée dans le crate source : `list_networks`, `inspect_network` (verbosité par défaut suffisante : `Network.containers` est rempli), `remove_network`, `disconnect_network` (`NetworkDisconnectRequest { container, force }`), `prune_networks` (`NetworkPruneResponse { networks_deleted, space_reclaimed }`). Aucune nouvelle dépendance : bollard, chrono, mockall, ratatui sont déjà utilisés.

Particularité par rapport aux volumes : les réseaux **prédéfinis** (`bridge`, `host`, `none`) ne doivent jamais être supprimables — c'est une règle métier, donc elle vit dans le domaine (`can_be_deleted()`).

## Options considérées

- **Option A — area `network/` complète, pattern volume** (recommandée) : 4 couches + 2 écrans + 2 actions (delete, disconnect) + prune. Impact homogène sur toutes les couches, coût ~équivalent à l'area image, risque faible car 100 % calqué sur un pattern éprouvé. Recommandée : alignée avec `EVOLUTION.md` Phase 4 et la règle de placement du skill `rust-feature-ddd`.
- **Option B — liste seule, sans détails ni disconnect** : coût réduit (~-40 %), mais `list_networks` ne fournit pas les containers connectés de façon fiable (`Network` listé vs inspecté) ; les détails seraient incomplets et il faudrait y revenir. Rejetée : livrer un écran Details pauvre contredit l'objectif de reproduire le pattern « de A à Z ».
- **Option C — réutiliser l'écran StackContainers pour la liste des containers connectés** : évite un écran, mais couple deux areas distinctes et rend la navigation ambigüe. Rejetée : viole l'isolation des areas du projet.

## Décisions d'architecture

- **D1** — Entité `Network` dans `01_domain/network/` avec value objects `NetworkId` (rejeter vide → `DomainError::InvalidNetwork`, variante à ajouter dans `01_domain/error.rs`), `NetworkScope` (enum `Local`/`Swarm`, `Global`) et réutilisation de types primitifs pour le reste. Champs : `id`, `name`, `driver`, `scope`, `internal`, `attachable`, `subnet` (`Option<IpNet>` minimal : `Option<String>` en v1), `gateway: Option<String>`, `created`, `builtin: bool`, `containers: Vec<NetworkContainer>` (nom + ip). *Raison : miroir exact de `VolumeId`/`Volume` ; `builtin` porte la règle métier.*
- **D2** — Règle métier `can_be_deleted()` = `!builtin && containers.is_empty()` ; `can_disconnect(container)` = `containers` non vide. Logique dans l'entité, testée sans Docker. *Raison : le domaine est pur et sans dépendance externe, c'est là que vit l'invariant.*
- **D3** — `NetworkRepository` en `02_application/network/traits.rs` : `get_all()`, `get_by_name()`, `inspect(name) -> Network` (enrichit avec containers), `delete(name)`, `prune()`, `disconnect(network, container, force)`. Le `NetworkService` reste stateless et mappe vers `NetworkDto`. `#[cfg_attr(test, mockall::automock)]` comme `VolumeRepository`. *Raison : inversion de dépendance identique aux autres areas.*
- **D4** — `NetworkAdapter` en `03_infrastructure/docker/network/` : `list_networks` + `inspect_network` par nom (l'inspect fournit `IPAM.config[0]` subnet/gateway et `containers` — le list ne le garantit pas), `remove_network`, `prune_networks` (mappe vers `PruneResultDto` : `networks_deleted.len()` / `space_reclaimed`), `disconnect_network` via `NetworkDisconnectRequest`. Détection `builtin` : `Options` (labels) contient la clé `com.docker.network.bridge.name` OU name ∈ {`bridge`, `host`, `none`} — calqué sur le comportement `docker network ls`. *Raison : fidélité à l'API Docker ; inspect par item est le pattern déjà utilisé par `get_by_name` côté volume (list + find) mais ici l'inspect est nécessaire pour la complétude des données.*
- **D5** — Triade `04_presentation/tui/network/{presenter,view,actions}.rs` + `Screen::NetworkList` et `Screen::NetworkDetails` dans `ui/app.rs`, `MainMenuItems` étendu (`resources.rs` : tableau 5 → 6 entrées + `Quit`), `AppAction` réutilise les variants existants (`Delete`, `ViewDetails`, `Prune`) ; seule `DisconnectContainer` est nouvelle, mappée sur `d` (uniquement actif sur `Screen::NetworkDetails`). *Raison : réutiliser les actions existantes minimise les touches et le câblage ; `d` est libre et mnémonique.*
- **D6** — Écran `NetworkList` : colonnes NAME / DRIVER / SCOPE / SUBNET / CONTAINERS (count) ; filtre `/` sur le nom (réutilise `FilterState`), `r` refresh, `Enter` → Details, `X` prune avec confirmation (pattern `Prune` existant), `d` delete avec confirmation. Écran `NetworkDetails` : sections Driver/Scope/Internal/Attachable/Subnet/Gateway/Created + liste des containers connectés (nom + IP), `d` sur un container → confirmation → `disconnect(force = false)`. *Raison : cohérence UX avec les écrans volume/image.*
- **D7** — Câblage `main.rs` : `NetworkAdapter::new(docker.clone())` → `Arc` → `NetworkService::new(...)` → `NetworkActions::new(...)` → `App::new(...)`. *Raison : ordre bottom-up déjà documenté dans `AGENTS.md`.*
- **D8** — Aucune nouvelle dépendance Cargo. `bollard = "0.21"` expose déjà tout le nécessaire ; `ipnetwork` n'est pas requis (subnet/gateway en `String` en v1, le formatage réseau n'apporte rien à un TUI). *Raison : skill `rust-dependency` — pas de crate sans justification.*

## Étapes

1. **Domaine** — créer `src/01_domain/network/{mod.rs, entity.rs, value_objects.rs}` + variante `InvalidNetwork(String)` dans `src/01_domain/error.rs` ; déclarer `pub mod network;` dans `src/01_domain/mod.rs`. Tests : constructeur builder (`with_*`), `can_be_deleted` (builtin / containers attachés / cas normal), `can_disconnect`. — fichiers : `src/01_domain/network/*`, `src/01_domain/error.rs`, `src/01_domain/mod.rs` — vérification : `cargo test network && cargo clippy -- -D warnings`
2. **Application** — `src/02_application/network/{mod.rs, traits.rs, dto.rs, mapper.rs, service.rs}` + export dans `src/02_application/mod.rs`. `NetworkDto` : `name, id, driver, scope, internal, attachable, subnet: Option<String>, gateway: Option<String>, created, builtin, containers: Vec<NetworkContainerDto { name, ip }>`, plus `can_delete`/`container_count` dérivés. Tests : service avec `MockNetworkRepository` (get_all, get_by_name, delete, prune, disconnect — incl. erreur propagée). — vérification : `cargo test network && cargo clippy -- -D warnings`
3. **Infrastructure** — `src/03_infrastructure/docker/network/{mod.rs, adapter.rs, mapper.rs}` + déclaration dans `src/03_infrastructure/docker/mod.rs`. Mapper `NetworkInfraMapper::from_docker` (list) et `from_inspect` (enrichi) ; détection builtin ; extraction subnet/gateway depuis `ipam.config`. Tests unitaires du mapper avec structs bollard construits à la main (pattern `make_container` du `VolumeAdapter`). — vérification : `cargo test network && cargo clippy -- -D warnings`
4. **Presenter & actions UI** — `src/04_presentation/tui/network/{mod.rs, presenter.rs, view.rs, actions.rs}` + `pub mod network;` dans `src/04_presentation/tui/mod.rs`. `NetworkPresenter` (volumes, selection `TableSelection`, filter `FilterState`, loading, error) ; `filter_networks` ; actions async fines sur `NetworkService`. Tests : filtrage, sélection, reset au refresh. — vérification : `cargo test network && cargo clippy -- -D warnings`
5. **Écrans & navigation** — `Screen::NetworkList` / `Screen::NetworkDetails` dans `src/04_presentation/tui/app.rs` ; entrée « Networks » dans `resources.rs` (`MAIN_MENU_ITEMS` 6 items + mise à jour des limites de navigation menu→Screen, reprendre le mapping indexé existant) ; `render` + `handle_*` pour les deux écrans ; `AppAction::DisconnectContainer` + mapping `d` dans `keys.rs` (Scopes : `d` reste libre sur ces écrans ; vérifier qu'aucun conflit `d` = delete sur NetworkList → utiliser `Delete` existant mappé ailleurs et `DisconnectContainer` seulement sur Details). Confirmation dialogs réutilisés (`confirm_dialog`). — vérification : `cargo test && cargo clippy -- -D warnings`
6. **Câblage main.rs** — instanciation adapter→service→actions, passage à `App::new`, refresh après delete/prune/disconnect. Tests d'intégration `App` (navigation MainMenu → NetworkList → Details → retour) sur le modèle des tests existants dans `app.rs`. — vérification : `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
7. **Docs & quality gate** — mise à jour `README.md` (features + key bindings), `EVOLUTION.md` (Phase 4 → livrée), `PLAN.md` si backlog. — vérification : skill `rust-quality-gate` complet : `cargo fmt --check && cargo clippy -- -D warnings && cargo test`

## Risques et mitigations

- **`list_networks` ne remplit pas toujours `containers`/IPAM de façon détaillée** → l'adapter fait un `inspect_network` par réseau lors du chargement de la liste (N+1 acceptable : quelques réseaux ; parallélisable plus tard). Si le N+1 s'avère visible, basculer sur list seul pour l'écran liste et inspect uniquement à l'ouverture des détails (option déjà isolée derrière le repository).
- **Suppression des réseaux prédéfinis (`bridge`/`host`/`none`)** → règle métier `can_be_deleted()` côté domaine + garde-fou UI (désactivation de l'action + confirmation). Double protection.
- **`d` déjà utilisé ailleurs (delete)** → garder `Delete` existant pour la suppression réseau (touche déjà standard du projet sur les listes) et mapper `DisconnectContainer` uniquement sur `Screen::NetworkDetails` ; le dispatch par écran évite toute collision.
- **Prune renvoie `space_reclaimed` souvent à 0** → `PruneResultDto` affiche le compte de réseaux supprimés, message inchangé.
- **Menu principal indexé par position** (`MAIN_MENU_ITEMS` passé de 5 à 6) → toute navigation mappée sur l'index doit être revérifiée (grep `MainMenu` dans `app.rs`/`router.rs`).

## Hors périmètre

- Création de réseaux (`create_network` bollard) — pas demandée, UI de saisie complexe.
- Connexion d'un container à un réseau (`connect_network`) — dual de disconnect, ajoutable plus tard par simple étape.
- Affichage temps réel / stats réseau — Phase 5 d'`EVOLUTION.md`.
- Gestion des networks overlay Swarm (drivers non locaux en écriture, scope global en lecture seule).
- Tests d'intégration contre un vrai démon Docker (la CI reste sur tests unitaires + TestBackend ratatui).

---

Note architecte : chaque étape laisse le repo compilable et les tests verts, dans l'ordre domain → application → infrastructure → presentation → wiring → docs. Aucune étape ne viole le flux `04_presentation → 02_application → 01_domain ← 03_infrastructure`.
