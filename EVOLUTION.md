# Rusty Console — Plan d'évolution

## Roadmap globale

| Phase | Thème | Concepts Rust |
|-------|-------|---------------|
| **1** | Quick wins (containers) | Consolider traits, enums, pattern matching |
| **2** | Docker Compose stacks | Grouper des données, nouveaux value objects |
| **3** | Networks (nouveau domaine) | Appliquer le DDD seul de A à Z |
| **4** | Stats temps réel | Streams async, `VecDeque`, `tokio::select!` |

---

## Phase 1 — Quick wins

5 features, ordonnées du plus simple au plus complexe.

### 1.1 Pause / Unpause — `p`

**Fichiers à modifier** :

```
infrastructure/container/adapter.rs  → ajouter pause_container(), unpause_container()
application/container/traits.rs      → ajouter dans le trait ContainerRepository
application/container/service.rs     → déléguer au repository
ui/container/actions.rs              → exposer à la UI
ui/common/keys.rs                    → mapper p → AppAction::PauseUnpause (nouvel variant)
ui/app.rs                            → dans handle_container_list_action(), même pattern que StartStop
ui/container/view.rs                 → ajouter p dans le help text
```

**Concept Rust** : `ContainerState` a déjà `Paused` et `is_active()`. La logique toggle sera
similaire à `toggle_container` mais pour pause/unpause.

---

### 1.2 Restart — `R` (Shift+R)

Même pattern exact que 1.1. bollard : `restart_container(id, options)`.

`RestartContainerOptions` a un champ `t: i64` (timeout en secondes avant SIGKILL). Mets 10.

**Fichiers à modifier** : mêmes 4 fichiers infrastructure/application/ui que 1.1.

**Astuce** : `Shift+R` pour ne pas écraser `r` (refresh). Dans `keys.rs`, matcher
`KeyCode::Char('R')` — la majuscule est automatique avec Shift, pas besoin de vérifier le
modifier.

---

### 1.3 Env vars dans les détails

**Vérifier d'abord** si `env_vars` est déjà dans `application/container/dto.rs`. Si non,
parcourir le chemin complet :

```
domain/container/entity.rs           → ajouter env_vars: Vec<String> dans Container
infrastructure/container/mapper.rs   → peupler depuis inspect.config.env
application/container/dto.rs         → ajouter le champ
application/container/mapper.rs      → mapper domain → dto
ui/container/view.rs                 → afficher dans render_container_details()
```

**Concept Rust** : `Option<Vec<String>>` depuis bollard → `unwrap_or_default()` → afficher
chaque var sur une ligne.

---

### 1.4 Prune — `X` (Shift+X)

Supprime toutes les ressources inutilisées du type affiché :
- Containers : supprime les stoppés (`prune_containers`)
- Volumes : supprime les non-montés (`prune_volumes`)
- Images : supprime les dangling (`prune_images`)

**Fichiers à modifier** :

```
infrastructure/*/adapter.rs          → appel bollard prune_*()
application/*/traits.rs              → ajouter prune() au trait
application/*/service.rs             → déléguer
ui/*/actions.rs                      → exposer
application/container/dto.rs         → PruneResultDto { deleted: u32, space_freed: u64 }
ui/app.rs                            → ajouter ConfirmAction::Prune dans l'enum
                                        afficher le résultat dans un message après confirmation
```

**Concept Rust** : `PruneContainersResponse` de bollard retourne
`containers_deleted: Option<Vec<String>>` et `space_reclaimed: Option<i64>`. Pratiquer
`.map()`, `.unwrap_or_default()`, `.len()` sur des `Option<Vec<_>>`.

---

### 1.5 Filter / Search — `/`

Le plus riche en apprentissage. Appuyer `/` active un mode "saisie", les caractères tapés
filtrent la liste en temps réel, `Esc` annule et vide le filtre.

**Nouveaux champs dans chaque presenter** :

```rust
pub filter: String,
pub filter_active: bool,
```

**Méthode à ajouter dans chaque presenter** :

```rust
pub fn filtered_containers(&self) -> Vec<&ContainerDto> {
    // indice : .iter().filter(|c| ...).collect()
}
```

**Fichiers à modifier** :

```
ui/*/presenter.rs    → ajouter filter, filter_active, filtered_*(), méthodes push/pop char
ui/app.rs            → avant de mapper les touches vers AppAction, intercepter les chars
                       si filter_active est true → envoyer au presenter, ne pas passer par keys.rs
ui/*/view.rs         → afficher le filtre actif dans le titre ou une ligne dédiée
ui/common/keys.rs    → mapper / → AppAction::ActivateFilter (nouvel variant)
```

**Concept Rust clé** : closures dans `filter()`, références dans `Vec<&ContainerDto>`,
gestion d'état booléen dans la machine à états. Tu vas aussi voir pourquoi `String` vs `&str`
compte ici.

---

## Ordre suggéré

```
1.1 Pause/Unpause  →  1.2 Restart  →  1.3 Env vars  →  1.4 Prune  →  1.5 Filter
```

| Feature | Durée estimée | Difficulté |
|---------|--------------|------------|
| 1.1 Pause/Unpause | ~30-60 min | ⭐ |
| 1.2 Restart | ~30-60 min | ⭐ |
| 1.3 Env vars | ~30-60 min | ⭐ |
| 1.4 Prune | ~2h | ⭐⭐ |
| 1.5 Filter | ~3-4h | ⭐⭐⭐ |

---

## Phase 2 — Docker Compose stacks

Docker ne stocke pas les stacks nativement : les containers créés par Compose portent le label
`com.docker.compose.project`. L'idée est de grouper les containers par ce label pour afficher
et gérer les stacks. Les containers sans ce label sont regroupés dans une stack spéciale
`(standalone)`.

### Scope retenu

| # | Écran | Touches disponibles |
|---|-------|---------------------|
| 2.1 | **StackList** — liste toutes les stacks détectées | `j/k` navigation, `Enter` drill-down, `s` Start All, `S` Stop All, `q/Esc` retour menu |
| 2.2 | **StackContainers** — containers d'une stack | mêmes touches que ContainerList (start/stop par container), `q/Esc` retour StackList |

---

### 2.1 Nouveau domaine `stack/`

Structure de dossiers (pattern feature-first) :

```
src/stack/
  domain/
    entity.rs          -> Stack { name: StackName, containers: Vec<Container> }
    value_objects.rs   -> StackName(String)  — invariant : non vide
    mod.rs
  application/
    dto.rs             -> StackDto { name: String, container_count: usize, running_count: usize,
                          containers: Vec<ContainerDto> }
    mapper.rs          -> Stack -> StackDto
    service.rs         -> StackService::list_stacks() -> Vec<StackDto>
                          StackService::start_all(name) / stop_all(name)
    traits.rs          -> trait StackRepository { list_stacks, start_all, stop_all }
    mod.rs
  infrastructure/
    adapter.rs         -> StackAdapter — appelle bollard list_containers, groupe par label
                          containers sans label -> groupe "(standalone)"
    mapper.rs          -> réutilise ContainerInfraMapper pour chaque container du groupe
    mod.rs
  ui/
    presenter.rs       -> StackPresenter { stacks: Vec<StackDto>, selected: usize }
                          méthode selected_stack() -> Option<&StackDto>
    view.rs            -> render_stack_list() — tableau nom / total / running
    actions.rs         -> StackActions { list_stacks(), start_all(name), stop_all(name) }
    mod.rs
  mod.rs
```

---

### 2.2 Concept Rust clé : `HashMap` pour le grouping

```rust
// dans StackAdapter::list_stacks()
let mut groups: HashMap<String, Vec<Container>> = HashMap::new();
for c in containers {
    let key = c.labels
        .get("com.docker.compose.project")
        .cloned()
        .unwrap_or_else(|| "(standalone)".to_string());
    groups.entry(key).or_default().push(c);
}
```

Concepts exercés : `HashMap::entry`, `or_default()`, `cloned()` sur `Option<&String>`,
itération sur les valeurs d'une map pour construire des entités `Stack`.

---

### 2.3 Écran StackContainers (drill-down)

Le domaine `Container` ne porte pas les labels Docker (détail d'infra). Le grouping se fait
entièrement dans `StackAdapter`, et `StackDto` embarque directement ses containers :

```rust
// dans dto.rs
pub struct StackDto {
    pub name: String,
    pub container_count: usize,
    pub running_count: usize,
    pub containers: Vec<ContainerDto>,  // pré-groupé par l'infra
}
```

Le drill-down charge directement les containers de la stack sélectionnée :

```rust
// dans handle_stack_list_action — AppAction::Select
if let Some(stack) = self.stack_presenter.selected_stack() {
    self.container_presenter.set_containers(stack.containers.clone());
    self.screen = Screen::StackContainers;
}
```

**Pourquoi ce choix ?** Le label `com.docker.compose.project` est un détail de bollard,
pas un concept du domaine. L'adapter lit `bollard::models::ContainerSummary.labels`, groupe,
puis construit les `Stack` entités — la UI reçoit des données propres, déjà structurées.

Concepts exercés : ownership dans `Vec<ContainerDto>`, `clone()`, séparation des responsabilités.

---

### 2.4 Fichiers à modifier (hors `stack/`)

```
src/lib.rs              -> déclarer mod stack; re-exporter les types publics
src/main.rs             -> créer StackAdapter, StackService, StackActions; injecter dans App
src/ui/app.rs           -> ajouter Screen::StackList et Screen::StackContainers
                           ajouter stack_presenter: StackPresenter, stack_actions: StackActions
                           render_stack_list(), handle_stack_list_action()
                           menu : ajouter "  Stacks" comme 4e option
src/ui/common/keys.rs   -> ajouter AppAction::StopAll ; réutiliser StartStop pour "Start All"
                           et documenter le drill-down via l'action/navigation existante
```

---

### Ordre suggéré

```
2.1 Domaine Stack  ->  2.2 Infrastructure adapter  ->  2.3 Service + traits
  ->  2.4 UI StackList  ->  2.5 UI StackContainers (drill-down)
```

| Sous-feature | Difficulté |
|---|---|
| 2.1 Domaine Stack | ⭐ |
| 2.2 Infrastructure adapter (grouping) | ⭐⭐ |
| 2.3 Service + traits | ⭐ |
| 2.4 UI StackList | ⭐⭐ |
| 2.5 UI StackContainers drill-down | ⭐⭐⭐ |

---

## Phase 3 — Networks (à venir)

Nouveau domaine `network/` qui reproduit le pattern DDD de A à Z :
`domain` → `application` → `infrastructure` → `ui`.
Objectif : appliquer le pattern seul, sans guide.

---

## Phase 4 — Stats temps réel (à venir)

CPU %, mémoire, réseau I/O via le stream `bollard::stats()`. Widgets ratatui `Gauge` et
`Sparkline`. Historique glissant avec `VecDeque<f64>`. Requiert d'être à l'aise avec les
streams async (`tokio::select!`).
