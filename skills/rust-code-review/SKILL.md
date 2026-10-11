---
name: rust-code-review
description: Relire une pull request rusty-console sur GitHub — examen systématique par couche DDD, contrôle qualité, verdict actionnable. Utiliser ce skill pour toute revue de PR (demandée, assignée, ou en surveillance de CI).
---

# Skill : Revue de code Rust (pull requests)

## Procédure systématique — 6 étapes

### 1. Collecter le contexte
```bash
gh pr view <n> --repo TataneCode/rusty-console
gh pr diff <n>
```
- Lire le titre, la description et l'issue liée.
- Vérifier la taille du diff : une PR > ~400 lignes de logique est un signal de découpage insuffisant (à noter en commentaire, pas bloquant d'office).

### 2. Vérifier le périmètre
- Le diff fait-il exactement ce que le titre/description annonce ? Tout changement non lié = demande d'extraction.
- Aucun fichier généré ni `Cargo.lock` inutilement churné (sauf changement de dépendance justifié).

### 3. Examiner par couche (règles de blocage)
- **Domain (`01_domain`)** : aucun crate externe ; invariants validés dans les types ; `DomainError` pour tout échec de construction.
- **Application (`02_application`)** : traits `#[async_trait]` + `#[cfg_attr(test, mockall::automock)]` ; services stateless ; DTO/mappers dédiés ; pas d'entité domain exposée.
- **Infrastructure (`03_infrastructure`)** : types bollard confinés ici ; mappers vers le domaine ; conversion `InfraError → AppError`.
- **Presentation (`04_presentation`)** : triade presenter/view/actions respectée ; logique métier absente de l'UI ; `AppAction`/`Screen` étendus proprement ; erreurs catchées via `App::error_message`.
- **Transverse** : chaîne d'erreurs thiserror respectée ; pas de nouveau `unwrap()`/`expect()` sur chemin d'erreur ; wiring `main.rs` cohérent.

### 4. Contrôler la qualité et les tests
- Nouvelle logique métier → tests présents et significatifs (pas de test qui teste le mock).
- Bugfix → test de régression présent (voir skill `rust-bugfix`).
- Si la CI tourne : vérifier son verdict ; en cas d'échec, chercher la cause racine dans les logs avant tout commentaire.
- `cargo clippy -- -D warnings` et `cargo fmt` doivent être nets (vérifiable en local sur la branche si besoin).

### 5. Rédiger la revue
- Structure : **Verdict** (approve / request changes / comment) en tête, puis findings par sévérité :
  - 🔴 **Bloquant** — viole l'architecture, casse la CI, bug, sécurité
  - 🟡 **Important** — pattern contourné, test manquant, lisibilité
  - 🟢 **Mineur / nit** — style, nommage (non bloquant)
- Chaque finding : `fichier:ligne`, explication du problème, suggestion concrète. Pas de « c'est moche » sans alternative.
- Saluer explicitement les bonnes décisions (une revue n'est pas qu'une liste de défauts).

### 6. Publier et suivre
- Publier via `gh pr review` avec un corps structuré (fichier temp + `--body-file`).
- Après retours de l'auteur : re-vérifier uniquement les points corrigés, pas toute la PR.

## Anti-patterns interdits

- Revue superficielle « LGTM » sur un diff contenant de la logique métier.
- Findings sans localisation ni suggestion.
- Demander du style personnel contraire aux conventions du repo (ce sont elles qui font foi, via `AGENTS.md` et les skills).
- Approuver une PR qui viole le flux de dépendances DDD « parce que c'est petit ».
