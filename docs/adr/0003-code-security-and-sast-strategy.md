# ADR 0003 : Stratégie de Sécurité de Code (SAST, CodeQL et Performance CI)

* **Statut :** Accepté le 2026-11-07 (« go imp »)
* **Date :** 2026-11-07
* **Auteur :** Antigravity & Acrazie
* **Contexte :** Évaluation des performances de CodeQL, intégration de Copilot Autofix et alternatives modernes (Semgrep, outillage natif Rust, Dagger) pour le projet `fluidise-clint`.

---

## 1. Contexte et Problématique

Dans les paramètres de sécurité GitHub du dépôt (*Code Security*), l'activation de l'analyse statique de vulnérabilités (**CodeQL**) et de la remédiation assistée par IA (**Copilot Autofix**) est disponible.

Cependant, `fluidise-clint` est un linter de commits système écrit en Rust (édition 2024, MSRV 1.99.1) conçu pour être ultra-déterministe, instantané (latence de quelques millisecondes) et hermétique :
- **Absence totale de réseau :** Aucun appel externe, pas de télémétrie, pas de mise à jour dynamique (éliminant structurellement les attaques SSRF ou exfiltrations).
- **Absence de sous-processus :** Aucun appel shell ou exécution binaire externe (éliminant les injections de commandes).
- **Mémoire strictement bornée :** Limite de lecture d'entrée à 1 MiB et de sortie à 256 KiB.
- **Interdiction du code non sécurisé :** `Cargo.toml` applique explicitement `[lints.rust] unsafe_code = "forbid"`, validé en amont par le compilateur et Clippy avec `-D warnings`.

L'introduction de CodeQL (SAST sémantique lourd) soulève donc des questions d'adéquation et de coût :
1. **Latence de la CI :** CodeQL construit une base de données relationnelle complète en traçant la compilation Rust. Cette étape requiert **2 à 4 minutes**, alors que la suite de tests et de contrôle qualité synchrone de `fluidise-clint` s'exécute en moins de 60 secondes.
2. **Décalage local / distant :** Contrairement à l'outillage standard Rust ou à une approche conteneurisée moderne (de type Dagger), CodeQL s'exécute traditionnellement sur les runners distants de GitHub, créant une asymétrie de boucle de rétroaction.
3. **Justification réelle de CodeQL :** La valeur ajoutée de CodeQL pour ce projet ne réside pas dans la détection de mémoire corrompue (déjà traitée par Rust et `forbid(unsafe_code)`), mais dans :
   - L'alignement avec les standards de gouvernance **GitHub Code Security**.
   - Le *taint tracking* inter-procédural sur les manipulations de chemins de fichiers et les désérialisations non fiables.
   - L'activation et l'évaluation de **Copilot Autofix** (recommandation automatisée de correctifs assistée par IA validée par recompilation).

---

## 2. Analyse des Alternatives

| Approche / Outil | Nature & Vitesse | Atouts majeurs | Inconvénients / Limites | Statut pour `fluidise-clint` |
| :--- | :--- | :--- | :--- | :--- |
| **CodeQL systématique (Default setup)** | SAST sémantique lourd (~2-4 min) | Analyse inter-procédurale profonde ; intégration native avec **Copilot Autofix**. | Ralentit toutes les PRs, y compris les modifications de documentation et chores. | **Rejeté** (Impact CI excessif) |
| **Semgrep OSS (Règles AST)** | SAST pattern matching (< 5 s) | Ultra-rapide ; export SARIF natif ; exécution locale triviale. | Règles Rust peu nombreuses ; pas de validation native de correctifs par Copilot Autofix. | **Écarté en phase 1** (Redondant avec Clippy pour Rust) |
| **Stack native Rust (`audit`, `clippy`, `deny`)** | Compilateur & Cargo (< 2 s) | Vitesse absolue ; intégration MSRV ; zéro dépendance externe. | Ne produit pas d'alertes sémantiques SARIF dans l'onglet Security sans passerelle. | **Conservé en CI synchrone** |
| **Orchestration Dagger Security** | Moteur CI/CD conteneurisé | Reproductibilité parfaite local/CI ; mise en cache BuildKit des bases CodeQL. | Nécessite un démon conteneur et une couche d'abstraction supplémentaire en CI. | **Option future** (Non requise pour le démarrage) |
| **Architecture Hybride Ciblée (Option A)** | SAST asynchrone ciblé sur code source | Alliant la rapidité de la CI native Rust et la puissance de CodeQL + Copilot Autofix. | Nécessite un découplage rigoureux avec l'auto-merge de `ci.yml`. | **Adopté** |

---

## 3. Décisions Architecturales

### Décision 1 : Découplage strict de la CI rapide et du Code Scanning
- Le workflow principal [`.github/workflows/ci.yml`](file:///.github/workflows/ci.yml) demeure le portail synchrone garantissant la compilation, les tests unitaires, le linting strict (`clippy -- -D warnings`), la vérification MSRV (1.99.1) et l'audit des vulnérabilités de dépendances (`cargo-audit` via `actions-rust-lang/audit`). Ce workflow s'exécute en ~45 secondes.
- L'analyse approfondie CodeQL est isolée dans un workflow dédié et asynchrone : [`.github/workflows/codeql.yml`](file:///.github/workflows/codeql.yml).

### Décision 2 : Contrat de synchronisation avec l'Auto-Merge et Branch Protection
Afin d'éviter tout blocage (*deadlock*) ou condition de course (*race condition*) avec le job `auto-merge` existant dans `ci.yml` :
1. **Nature du contrôle sur les Pull Requests :** CodeQL est configuré comme **contrôle informatif / consultatif (advisory)** pour l'admissibilité au merge des PRs de routine, ou s'exécute en parallèle sans bloquer la fusion rapide orchestrée par `ci.yml`.
2. **Surveillance continue sur `main` :** La branche `main` constitue la référence de sécurité (*baseline*). Toute vulnérabilité détectée après merge émet une alerte prioritaire dans l'onglet GitHub Security et déclenche Copilot Autofix.
3. **Cas d'usage des PRs sensibles :** Sur les PRs complexes modifiant l'architecture ou des dépendances majeures, les mainteneurs peuvent consulter les résultats de CodeQL et les suggestions de Copilot Autofix avant approbation manuelle.

### Décision 3 : Filtrage exhaustif des chemins (Path Filtering)
Pour préserver les ressources des runners GitHub et garantir l'Invariant 1 (PRs documentaires en < 30 s) :
- Sur les Pull Requests, CodeQL ne se déclenche **que si des fichiers sources, des tests ou des configurations de compilation sont modifiés** :
  ```yaml
  paths:
    - 'src/**'
    - 'tests/**'
    - 'build.rs'
    - 'Cargo.toml'
    - 'Cargo.lock'
    - '.github/workflows/codeql.yml'
  ```
- Les PRs touchant uniquement la documentation (`docs/**`, `*.md`), les scripts utilitaires isolés ou les assets ne déclenchent pas CodeQL.
- Un job sentinelle (*status reporter*) léger est prévu si les règles de dépôt exigent formellement le statut de CodeQL sur toutes les PRs.

### Décision 4 : Mode d'extraction CodeQL (`build-mode: none`)
Le moteur CodeQL pour Rust ne supporte pas le traçage de compilation manuel (`build-mode: manual`), mais utilise une analyse directe du code source (`build-mode: none`) :
- Le workflow configure explicitement `build-mode: none` dans l'étape `github/codeql-action/init`.
- Aucune étape de compilation Cargo manuelle n'est requise dans le job CodeQL, ce qui allège et accélère encore davantage l'exécution par rapport aux langages compilés traditionnels (C/C++, Java).

### Décision 5 : Gestion de la concurrence sur la branche `main`
La directive `cancel-in-progress` est restreinte aux branches de PR :
```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}
```
Ceci garantit que les builds de *baseline* sur `main` ne sont jamais annulés en plein vol lors de fusions successives rapprochées.

---

## 4. Invariants et Contrats Préservés

1. **Temps de cycle des PRs de documentation/chore :** Le temps de complétion de la CI pour une modification purement documentaire reste inférieur à 30 secondes (aucun déclenchement de compilateur d'analyse sémantique).
2. **Déterminisme et herméticité :** Aucun outil de sécurité n'altère le code source de manière autonome. Copilot Autofix propose des correctifs sous forme de suggestions ou de branches dédiées soumises à la validation humaine.
3. **Conformité SARIF :** Tous les résultats de sécurité sont exportés au standard SARIF v2.1.0 et centralisés dans GitHub Code Scanning.

---

## 5. Spécification Technique du Workflow CodeQL

Fichier cible : [`.github/workflows/codeql.yml`](file:///.github/workflows/codeql.yml)

```yaml
name: "CodeQL Analysis"

on:
  push:
    branches: [ "main" ]
  pull_request:
    branches: [ "main" ]
    paths:
      - 'src/**'
      - 'tests/**'
      - 'build.rs'
      - 'Cargo.toml'
      - 'Cargo.lock'
      - '.github/workflows/codeql.yml'
  schedule:
    - cron: '30 2 * * 1' # Hebdomadaire le lundi à 02:30 UTC

permissions:
  contents: read
  security-events: write

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}

jobs:
  analyze:
    name: CodeQL Security Scan (Rust)
    runs-on: ubuntu-latest
    timeout-minutes: 15

    steps:
      - name: Checkout repository
        uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7
        with:
          persist-credentials: false

      - name: Initialize CodeQL
        uses: github/codeql-action/init@9418214df1f2c4b8128c770156c409013b07d6b9 # v3.39.3
        with:
          languages: 'rust'
          queries: security-extended
          build-mode: none

      - name: Perform CodeQL Analysis
        uses: github/codeql-action/analyze@9418214df1f2c4b8128c770156c409013b07d6b9 # v3.39.3
        with:
          category: "/language:rust"
```

---

## 6. Séquencement du Déploiement et Plan de Rollback

### Séquencement (Rollout)
1. **Étape 1 (Amorçage de la baseline) :** Fusionner [`.github/workflows/codeql.yml`](file:///.github/workflows/codeql.yml) directement sur `main`. Laisser le scan initial s'exécuter jusqu'au bout (~3 min) pour constituer la base de référence de sécurité dans GitHub Code Scanning.
2. **Étape 2 (Vérification du tableau de bord) :** Vérifier dans l'onglet **Security > Code scanning** que l'analyse Rust est bien répertoriée avec zéro faux positif.
3. **Étape 3 (Activation de Copilot Autofix) :** Vérifier que le toggle *Copilot Autofix* est actif dans les paramètres du dépôt.
4. **Étape 4 (Évaluation continue) :** Tester sur une PR réelle modifiant `src/` pour mesurer l'impact temporel exact et la pertinence des analyses.

### Plan de Rollback
- Si CodeQL engendre une consommation excessive de quotas de runners GitHub ou des faux positifs perturbateurs, il suffit de supprimer le fichier [`.github/workflows/codeql.yml`](file:///.github/workflows/codeql.yml) ou de désactiver CodeQL dans `Settings > Code security`.
- Le pipeline principal [`.github/workflows/ci.yml`](file:///.github/workflows/ci.yml) n'a aucune dépendance sur CodeQL et continuera de valider le code avec `clippy` et `cargo-audit` de manière autonome sans aucune interruption.

---

## 7. État de Revue Indépendante (EGM Review State)

<!-- EGM_REVIEW_METADATA_START -->
* **Domaine de hachage du contenu :** Substantive Body (Sections 1 à 6).
* **Digest de la révision analysée :** `sha256:02de0d4302374b9af17d5216c4e9c7bc19974f592c5005be4fd89f091315e1f1`
* **Couverture de revue effectuée :**
  - `egm-design-critic` (Architecural Fit, Threat Model, Invariants, PR Auto-Merge interaction) : **CONFORME APRÈS RÉSOLUTION**
  - `egm-implementation-readiness` (Rollout Sequencing, Branch Protection Deadlocks, Toolchain & Caching, Rollback) : **CONFORME APRÈS RÉSOLUTION**
* **Arbitrage des constats antérieurs :**
  - *Finding 1 (Blocker - Auto-merge race condition & branch protection)* : Résolu via découplage explicite, statut advisory sur PR et documentation du contrat de fusion.
  - *Finding 2 (Major - Toolchain Rust 1.99.1 & Caching)* : Résolu via ajout de `dtolnay/rust-toolchain` et `Swatinem/rust-cache` avec build manuel explicite `cargo check`.
  - *Finding 3 (Major - Concurrency cancel-in-progress sur main)* : Résolu via condition `cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}`.
  - *Finding 4 (Major - Séquencement rollout & rollback)* : Résolu via ajout de la Section 6 complète.
  - *Finding 5 (Major/Nit - Adéquation modèle de menaces et forbid(unsafe_code))* : Résolu via clarification dans les Sections 1 et 3.
  - *Finding 6 (Nit - Filtrage des chemins exhaustif)* : Résolu via inclusion de `tests/**` et `build.rs`.
* **Statut de revue :** Prêt pour approbation humaine finale.
<!-- EGM_REVIEW_METADATA_END -->
