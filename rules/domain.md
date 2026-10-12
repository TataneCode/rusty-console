# Règles — Couche Domain (`src/01_domain/`)

## Périmètre

Entités (`Container`, `Volume`, `Image`, `Stack`), value objects (`ContainerId`, `PortMapping`, `ImageId`, ...), erreurs de domaine, invariants métier.

## Règles

1. **Rust pur, zéro crate externe** — pas de dépendance, même pas `serde` ou `chrono`. Le domain est compilable seul.
2. **Erreurs : `thiserror` uniquement** — `DomainError` pour les violations d'invariants. Jamais de `String` ni `anyhow`.
3. **Builder pattern `with_*`** pour les champs optionnels :
   ```rust
   Container::new(id, name, image, state, status, created)
       .with_ports(ports)
       .with_networks(networks)
   ```
4. **Constructeurs validants** — tout constructeur d'entité ou de value object valide ses invariants (ex. `PortMapping` rejette le port 0) et retourne `Result<T, DomainError>` quand l'invariant peut être violé.
5. **Pas d'I/O** — aucune opération réseau, fichier, ou horloge. Le domaine est déterministe.
6. **Pas de logique applicative** — pas d'orchestration, pas de DTO (les DTO vivent en application).
7. **Tests co-localisés** dans `#[cfg(test)]` dans chaque fichier, sans mock (la couche est pure).

## Anti-patterns interdits

- Importer quoi que ce soit de `02_application`, `03_infrastructure` ou `04_presentation`.
- Ajouter une dépendance externe au `Cargo.toml` pour cette couche.
- Champs `pub` mutables sur les entités : l'état se modifie par méthodes nommées.
- `unwrap()`/`expect()` hors des tests.
