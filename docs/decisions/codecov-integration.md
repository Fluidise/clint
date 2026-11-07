# Décision d'architecture : Intégration de Codecov pour fluidise-clint

## Contexte et Mandat Initial

- **Objectif** : Mettre en place le service de suivi de couverture de code [Codecov](https://about.codecov.io/) pour le dépôt public `Fluidise/clint`, de façon déterministe, sécurisée et intégrée à l'infrastructure d'intégration continue GitHub Actions existante.
- **Périmètre et frontière d'écriture (Write Boundary)** :
  - **Fichiers créés (Create)** :
    - `codecov.yml` (Configuration racine Codecov).
  - **Fichiers modifiés (Modify)** :
    - `.github/workflows/ci.yml` (Ajout du job dédié `coverage`, permissions OIDC, mise à jour des dépendances d'`auto-merge`).
    - `README.md` (Ajout du badge Codecov stylisé selon la charte du projet).
  - **Fichiers vérifiés sans modification (ReadOnly / Dépendances stables)** :
    - [`scripts/coverage.sh`](file:///Users/acrazie/.gemini/antigravity/worktrees/lint/add_codecov_integration/scripts/coverage.sh) : Déjà conforme, produit `target/llvm-cov/lcov.info` avec le drapeau `--lcov`.
    - [`CONTRIBUTING.md`](file:///Users/acrazie/.gemini/antigravity/worktrees/lint/add_codecov_integration/CONTRIBUTING.md) : Déjà conforme, référence l'usage de `./scripts/coverage.sh`.
  - **Hors périmètre strict** :
    - Aucune modification du code source applicatif Rust (`src/*`) ni des tests (`tests/*`).
    - Aucune altération des benchmarks d'admission ni des contrats d'oracle de performance.
- **Critères de succès** :
  1. Génération reproductible et déterministe du rapport de couverture source au format LCOV via `cargo-llvm-cov`.
  2. Transmission sécurisée vers Codecov sans secret d'API statique grâce à l'authentification OpenID Connect (OIDC) native de GitHub Actions pour les branches et PRs internes.
  3. Actions GitHub de la chaîne CI rigoureusement épinglées par SHA commit immuable (garantie contre la falsification de la supply chain).
  4. Préservation de la vitesse d'exécution de la matrice de test existante (Linux, macOS, Windows) grâce à un job CI dédié `coverage` sur `ubuntu-latest`.
  5. Règles de couverture transparentes et robustes (`codecov.yml`) mesurant la couverture globale (`project`) et incrémentale (`patch`).
  6. Élimination de tout risque de deadlock entre la publication de statut Codecov et le job d'`auto-merge` au sein du workflow CI.
  7. Résilience de la chaîne CI face aux éventuelles indisponibilités transitoires du service Codecov.io.
- **Délégation d'édition** : Ce document constitue la conception d'architecture canonique. Aucune modification de code de production n'est effectuée durant cette phase de design.

---

## État des lieux technique

1. **Outillage local de couverture** :
   - Le script [`scripts/coverage.sh`](file:///Users/acrazie/.gemini/antigravity/worktrees/lint/add_codecov_integration/scripts/coverage.sh) utilise `cargo-llvm-cov`.
   - Il supporte l'option `--lcov` qui produit `target/llvm-cov/lcov.info` et l'option `--summary` qui rapporte un niveau de couverture élevé : **96.80% des lignes** et **94.25% des régions** couvertes sur l'ensemble du projet.

2. **Pipeline CI actuel ([`.github/workflows/ci.yml`](file:///Users/acrazie/.gemini/antigravity/worktrees/lint/add_codecov_integration/.github/workflows/ci.yml))** :
   - Job `quality` : exécute `cargo fmt`, `cargo clippy`, la validation de titre de PR via `fluidise-clint`, `cargo check` MSRV (1.99.1), et `cargo-audit`.
   - Job `test` : exécute les tests sur une matrice d'OS (`ubuntu-latest`, `macos-latest`, `windows-latest`).
   - Job `auto-merge` : fusionne automatiquement les PRs validées (`needs: [test]`).
   - Invariant de sécurité : Toutes les actions GitHub externes sont épinglées par SHA-1 commit immuable avec commentaire de tag.

3. **Visibilité du dépôt** :
   - `Fluidise/clint` est un dépôt open source **public** hébergé sur GitHub.

---

## Frontière de Décision & Choix Structuraux

### 1. Authentification & Upload Codecov (OIDC vs Secret Statique)

- **Contexte** : Historiquement, Codecov nécessitait la configuration d'un secret de dépôt `CODECOV_TOKEN`. Depuis `codecov-action@v5`, Codecov prend en charge l'authentification par jeton OIDC (`id-token: write`) pour les dépôts GitHub publics. Cependant, les PRs provenant de forks externes ne bénéficient pas de la permission `id-token: write` par restriction de sécurité GitHub.
- **Options évaluées** :
  - *Option A (Retenue)* : OIDC conditionnel (`use_oidc: ${{ github.event_name != 'pull_request' || github.event.pull_request.head.repo.full_name == github.repository }}`) avec support transparent de `token: ${{ secrets.CODECOV_TOKEN }}` et `fail_ci_if_error: false`.
  - *Option B* : Secret de dépôt GitHub statique `CODECOV_TOKEN` obligatoire sur tous les événements.
- **Décision** : **Option A**.
  - *Raison* : Élimine la gestion et le risque de fuite de jetons statiques pour toutes les branches et PRs du dépôt principal, tout en empêchant l'échec des workflows sur les PRs provenant de forks tiers.

### 2. Topologie du Job CI (Job Dédié vs Intégré à la Matrice `test`)

- **Contexte** : `cargo-llvm-cov` instrumente le code compilé avec des compteurs d'exécution LLVM, ce qui entraîne une surcharge de compilation (~30-50%) et requiert des artefacts de build distincts.
- **Options évaluées** :
  - *Option A (Retenue)* : Job dédié `coverage` sur `ubuntu-latest`, s'exécutant en parallèle des jobs `test`.
  - *Option B* : Exécuter la couverture à l'intérieur du job `test` pour la branche Linux uniquement.
  - *Option C* : Exécuter la couverture sur tous les systèmes d'exploitation de la matrice et agréger les rapports.
- **Décision** : **Option A**.
  - *Raison* : `fluidise-clint` est du code Rust pur sans logique spécifique à un système d'exploitation particulier. Isoler la couverture dans un job dédié évite de ralentir le job `test` matriciel, évite de polluer le cache de compilation standard avec les artefacts instrumentés par LLVM cov, et offre une traçabilité claire dans l'interface GitHub Actions.

### 3. Résolution d'Ordonnancement et Dépendances d'`auto-merge`

- **Contexte** : Le job `auto-merge` dépend actuellement de `needs: [test]`. Si `coverage` est ajouté à `needs: [test, coverage]`, et que `codecov.yml` était configuré avec `require_ci_to_pass: true`, Codecov attendrait que le workflow entier (incluant `auto-merge`) soit terminé avant de publier ses statuts, tandis qu'`auto-merge` attendrait le statut Codecov : un deadlock circulaire complet se produirait.
- **Options évaluées** :
  - *Option A (Retenue)* :
    - `auto-merge` requiert `needs: [test, coverage]`.
    - Dans `codecov.yml`, `codecov.require_ci_to_pass: false` et `notify.wait_for_ci: false`. Codecov traite et publie son statut d'analyse dès réception du rapport LCOV.
    - Dans le job `coverage`, l'exécution des tests instrumentés (`cargo llvm-cov`) est strictement obligatoire (échec immédiat du job en cas de test rouge).
    - L'étape d'upload Codecov utilise `fail_ci_if_error: false` : une panne réseau externe vers `codecov.io` ne bloque pas la livraison continue de l'équipe.
  - *Option B* : Upload strictement bloquant (`fail_ci_if_error: true`) et attente de CI (`require_ci_to_pass: true`).
- **Décision** : **Option A**.
  - *Raison* : Élimine mathématiquement le risque de deadlock, garantit la rigueur des tests avant merge, et découple la disponibilité du pipeline interne de la résilience d'un tiers SaaS.

### 4. Configuration `codecov.yml` et Seuils de Couverture

- **Contexte** : Codecov applique des statuts d'état de commit pour le projet (`project`) et pour le différentiel introduit par une PR (`patch`), et publie des résumés en commentaire de PR.
- **Décision** :
  - **`coverage.status.project`** : Cible automatique (`target: auto`) avec une tolérance de 1% (`threshold: 1%`).
  - **`coverage.status.patch`** : Cible fixée à `85%` avec `threshold: 1%`.
  - **Exclusions (`ignore`)** :
    - `tests/**/*` : Suites de tests et oracles.
    - `scripts/**/*` : Scripts d'automatisation et bancs de mesure.
    - `licenses/**/*` : Fichiers légaux de runtime.
    - `target/**/*` : Répertoire de compilation.
    - `**/*.md` : Documentation.
  - **Commentaires de PR** : Activés au format compact (`layout: "reach,diff,flags,files"`, `behavior: default`), mis à jour in-place sur chaque nouveau push sur la PR.

---

## Spécification Détaillée des Modifications

### 1. Fichier de Configuration `codecov.yml`

Emplacement : `/codecov.yml` à la racine du dépôt.

```yaml
# Configuration Codecov pour fluidise-clint
codecov:
  require_ci_to_pass: false
  notify:
    wait_for_ci: false

coverage:
  precision: 2
  round: down
  range: "80...100"

  status:
    project:
      default:
        target: auto
        threshold: 1%
        if_ci_failed: error
    patch:
      default:
        target: 85%
        threshold: 1%
        if_ci_failed: error

ignore:
  - "tests/**/*"
  - "scripts/**/*"
  - "licenses/**/*"
  - "target/**/*"
  - "**/*.md"

comment:
  layout: "reach,diff,flags,files"
  behavior: default
  require_changes: false
```

### 2. Workflow GitHub Actions `.github/workflows/ci.yml`

- **Permissions requises** : Le job `coverage` nécessite `id-token: write` pour l'authentification OIDC et `contents: read` pour le checkout de code.
- **Épinglage de sécurité des actions** :
  - `taiki-e/install-action@3a3daafd3b7f23b8f65d9f5ba2db203fb59b79e2 # v2.90.2`
  - `codecov/codecov-action@0fb7174895f61a3b6b78fc075e0cd60383518dac # v5.5.5`
- **Mise à jour d'`auto-merge`** : `needs: [test, coverage]`.

Extrait de spécification du job CI à insérer dans `.github/workflows/ci.yml` :

```yaml
  coverage:
    name: Code Coverage
    needs: quality
    runs-on: ubuntu-latest
    timeout-minutes: 15
    permissions:
      contents: read
      id-token: write
    steps:
      - name: Checkout repository
        uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7
        with:
          persist-credentials: false

      - name: Install Rust 1.99.1 toolchain
        uses: dtolnay/rust-toolchain@d124f86f28ff42855784ff3319f48ac182c248f8 # 1.99.1
        with:
          components: llvm-tools-preview

      - name: Cache Cargo dependencies
        uses: Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0 # v2
        with:
          key: coverage

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@3a3daafd3b7f23b8f65d9f5ba2db203fb59b79e2 # v2.90.2
        with:
          tool: cargo-llvm-cov

      - name: Generate LCOV coverage report
        run: cargo llvm-cov --locked --all-targets --lcov --output-path target/llvm-cov/lcov.info

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@0fb7174895f61a3b6b78fc075e0cd60383518dac # v5.5.5
        with:
          use_oidc: ${{ github.event_name != 'pull_request' || github.event.pull_request.head.repo.full_name == github.repository }}
          token: ${{ secrets.CODECOV_TOKEN }}
          files: target/llvm-cov/lcov.info
          flags: unittests
          name: fluidise-clint
          fail_ci_if_error: false
```

Et mise à jour du job `auto-merge` :

```yaml
  auto-merge:
    name: Auto-merge PR
    needs: [test, coverage]
    if: github.event_name == 'pull_request' && !github.event.pull_request.draft
```

### 3. Badge dans `README.md`

Insertion du badge Codecov dans le bloc centré `<p align="center">` de [`README.md`](file:///Users/acrazie/.gemini/antigravity/worktrees/lint/add_codecov_integration/README.md), en respectant la charte graphique shields.io et la palette exacte (`labelColor=24141E&color=603840`) :

```html
    <a href="https://codecov.io/gh/Fluidise/clint">
        <img src="https://img.shields.io/codecov/c/github/Fluidise/clint?style=flat&labelColor=24141E&color=603840&logo=Codecov&logoColor=white"></a>
```

---

## Risques, Rollback et Sécurité

1. **Sécurité de la chaîne logistique logicielle (Supply Chain)** :
   - Toutes les actions GitHub introduites (`taiki-e/install-action` et `codecov/codecov-action`) sont épinglées par SHA commit immuable.
   - Les permissions GitHub Actions sont cantonnées au strict minimum (`contents: read`, et `id-token: write` au niveau du job `coverage` uniquement).
2. **Gestion des forks externes** :
   - L'activation d'OIDC est conditionnée au fait que la PR provienne du dépôt source, évitant les erreurs de permission `id-token: write` sur les forks publics.
3. **Résilience opérationnelle & Rollback** :
   - `fail_ci_if_error: false` garantit qu'une panne de Codecov.io ne paralyse pas les merges de correctifs urgents.
   - Si Codecov doit être retiré, la désactivation consiste uniquement à supprimer `coverage` de `ci.yml` et le fichier `codecov.yml`, sans aucun impact sur la compilation ou les tests de `clint`.

---

## Plan de Vérification

1. **Validation locale de la génération LCOV** :
   - Exécution de `./scripts/coverage.sh --lcov` : vérification de la présence et de la conformité du rapport `target/llvm-cov/lcov.info`.
2. **Validation statique de la syntaxe YAML** :
   - Validation de la syntaxe YAML pour `codecov.yml` et `.github/workflows/ci.yml`.
3. **Validation de non-régression locale** :
   - `cargo fmt --all -- --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test --all-targets`
4. **Validation de l'intégration continue** :
   - Déclenchement de la CI sur la PR : vérification de l'exécution en parallèle du job `coverage`, de l'installation ultra-rapide de `cargo-llvm-cov`, de la transmission OIDC vers Codecov, et du déblocage normal d'`auto-merge`.

---

## État de la Revue (Review State)

- **Digest du corps substantiel** : `2d19c6ca59b9d074760974ff75c5e6810bf4e3ee5b8e5c8ffe8d32c4e8565ade`
- **Portée couverte** : Intégration Codecov complète (CI, configuration, outillage local, ordonnancement auto-merge, sécurité supply chain, charte graphique).
- **Reviewers mobilisés** : `egm-design-critic` (revue d'architecture, adéquation des contrats et préparation à l'implémentation).
- **Résolution des constats de revue** :
  - *Constat 1 (Major - Deadlock auto-merge)* : Résolu (`require_ci_to_pass: false`, `notify.wait_for_ci: false`).
  - *Constat 2 (Major - Frontière d'écriture floue)* : Résolu (classification explicite Create / Modify / ReadOnly).
  - *Constat 3 (Major - Épinglage SHA supply chain)* : Résolu (`taiki-e/install-action@3a3daafd3b7f23b8f65d9f5ba2db203fb59b79e2`, `codecov/codecov-action@0fb7174895f61a3b6b78fc075e0cd60383518dac`).
  - *Constat 4 (Major - Forks sous OIDC)* : Résolu (`use_oidc` conditionnel sur l'origine du repo).
  - *Constat 5 (Nit - Style du badge)* : Résolu (aligné sur shields.io avec palette `labelColor=24141E&color=603840`).
- **Statut** : Approuvé par l'utilisateur humain le 2026-11-07. Implémentation effectuée conformément à la conception validée.
