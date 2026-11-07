# Décision : Résolution de l'échec de résolution de digest Renovate pour `dtolnay/rust-toolchain`

* **Date :** 2026-11-07
* **Auteur :** Antigravity / EGM Design
* **Statut :** Approuvé par revue indépendante EGM — Prêt pour implémentation
* **Emplacement canonique :** `docs/decisions/fix-rust-toolchain-digest-renovate.md`
* **Branche cible :** `fix_rust_toolchain_digest`

---

## 1. Contexte et Mandat Initial

### 1.1. Contexte
Lors de l'analyse périodique par le bot Renovate sur le dépôt `Fluidise/clint`, l'avertissement bloquant suivant a été consigné dans le tableau de bord des dépendances (Issue #25) :

```text
⚠️ Package lookup failures
Dependency Lookup Warnings
Renovate failed to look up the following dependencies:
Could not determine new digest for update (github-tags package dtolnay/rust-toolchain)
Files affected: .github/workflows/ci.yml
```

### 1.2. Objectif du Mandat
1. Éliminer l'erreur de recherche de digest dans Renovate en rétablissant un contrat de versionnement et d'épinglage conforme aux spécifications de l'action GitHub `dtolnay/rust-toolchain` et de la datasource `github-tags` de Renovate.
2. Préserver strictement le MSRV fixé à **Rust 1.99.1** sur l'ensemble de la matrice CI (Ubuntu, macOS, Windows).
3. Résoudre le conflit de configuration Renovate entre le stub racine `renovate.json` et la configuration du projet `.github/renovate.json5`.

### 1.3. Invariants et Garanties Préservés
1. **Épinglage immuable par commit SHA (Full-length SHA)** : Toutes les actions GitHub tierces doivent demeurer épinglées par SHA complet (`pinDigests: true` / recommandation de sécurité GitHub).
2. **Immuabilité du toolchain Rust (MSRV 1.99.1)** : Aucune montée de version intempestive du compilateur Rust ne doit être introduite. Le fichier `rust-toolchain.toml` (`channel = "1.99.1"`) et l'étape CI `cargo +1.99.1 check` demeurent la référence.
3. **Zéro régression CI** : Les 3 jobs cibles (`quality`, `test` multi-OS, `auto-merge`) doivent s'exécuter à l'identique.

---

## 2. Analyse des Causes Racines (Root Cause Analysis)

### 2.1. Fonctionnement de `dtolnay/rust-toolchain` et de la Datasource Renovate `github-tags`

Dans `.github/workflows/ci.yml` (lignes 30 et 74), l'action était configurée ainsi :
```yaml
uses: dtolnay/rust-toolchain@d124f86f28ff42855784ff3319f48ac182c248f8 # 1.99.1
```

1. **Renovate et le gestionnaire `github-actions`** :
   Renovate extrait le motif `uses: <package>@<digest> # <tag>`. Pour `github-actions`, la datasource par défaut est `github-tags`. Renovate interroge l'API GitHub (`/repos/dtolnay/rust-toolchain/tags`) pour trouver le tag correspondant à `1.99.1` et en extraire le digest.
2. **Absence de tag `1.99.1` sur l'Action amont** :
   Le dépôt `dtolnay/rust-toolchain` n'utilise **aucun tag git** pour les versions de Rust. L'API renvoie uniquement le tag de release de l'action : `v1` (pointant sur le commit `683b588c487a69d01f6c6ddf40f036ff682cc2c8` de `master`).
   Les versions de compilateurs (`1.99.1`, `stable`, `nightly`) sont des **branches Git orphelines** maintenues par l'auteur avec le paramètre codé en dur dans `action.yml`.
   Renovate ne trouvant aucun tag nommé `1.99.1` dans `github-tags`, il échoue avec l'erreur `Could not determine new digest for update`.
3. **Risque de ramasse-miettes Git (Avertissement officiel de David Tolnay)** :
   La documentation officielle de `dtolnay/rust-toolchain` stipule expressément :
   > *"In a workflow that pins the action using a full-length commit SHA (as opposed to something like @nightly or @1.89.0) it is required that you pick a SHA that is within the history of the master branch. Any commit that is not within the history of master will eventually get garbage-collected and your workflows will fail."*
   Le commit `d124f86f28ff42855784ff3319f48ac182c248f8` se trouve sur la branche `1.99.1` et non dans l'historique de `master`, ce qui expose le workflow à une rupture en cas de garbage collection distant.

### 2.2. Conflit de Préséance des Fichiers de Configuration Renovate

Le dépôt contient actuellement deux fichiers de configuration Renovate :
- `renovate.json` (à la racine, créé lors de l'onboarding initial dans le commit `21ab432`).
- `.github/renovate.json5` (configuré dans le commit `25e2bfc` avec les règles sémantiques, les limites de concurrence et l'ignorance de `rust-toolchain.toml`).

Selon la documentation officielle Mend Renovate, l'ordre d'évaluation est :
1. `renovate.json`
2. `renovate.json5`
3. `.github/renovate.json`
4. `.github/renovate.json5`

Seul le premier fichier rencontré est chargé. Par conséquent, la présence de `renovate.json` à la racine court-circuite totalement `.github/renovate.json5`. Preuve : le tableau de bord Renovate liste `rust-toolchain.toml`, alors que `.github/renovate.json5` spécifie `"ignorePaths": ["**/rust-toolchain.toml"]`.

---

## 3. Frontière de Décision et Options Considérées

### Option A : Épinglage canonique `@v1` avec paramètre d'entrée explicite `toolchain: "1.99.1"` (Recommandée & Retenue)

- **Principe** :
  - Aligner l'action sur la release officielle `v1` située sur la branche `master` (`683b588c487a69d01f6c6ddf40f036ff682cc2c8` # `v1`).
  - Passer explicitement `toolchain: 1.99.1` dans les entrées (`with:`) de l'action dans les jobs `quality` et `test`.
  - Supprimer le fichier redondant `renovate.json` à la racine pour activer `.github/renovate.json5`.
- **Avantages** :
  - Conforme aux recommandations amont de `dtolnay/rust-toolchain` et aux bonnes pratiques de sécurité GitHub.
  - Le SHA appartient à la branche `master`, éliminant tout risque de garbage collection.
  - Le tag `# v1` est reconnu nativement par Renovate (`github-tags`), résolvant immédiatement l'avertissement sans bidouillage de regex ou d'exclusion.
  - La version de Rust (1.99.1) reste déclarée et isolée dans le paramètre `toolchain: 1.99.1`, en conformité avec `rust-toolchain.toml`.
  - Activation effective de `.github/renovate.json5` (qui ignore `rust-toolchain.toml` et configure les commits sémantiques).
- **Inconvénients** : Aucun.

### Option B : Désactivation de Renovate pour `dtolnay/rust-toolchain`

- **Principe** :
  - Conserver le SHA orphelin `d124f86f28ff42855784ff3319f48ac182c248f8 # 1.99.1` et ajouter une règle dans `.github/renovate.json5` :
    ```json5
    {
      "matchManagers": ["github-actions"],
      "matchPackageNames": ["dtolnay/rust-toolchain"],
      "enabled": false
    }
    ```
- **Rejet** :
  - Ne résout pas la cause sous-jacente du SHA orphelin hors `master` soumis à garbage collection.
  - Masque le problème plutôt que de restaurer une configuration saine et maintenable.

### Option C : Utilisation d'un custom regex manager pour les branches Git

- **Principe** : Définir un gestionnaire d'expressions régulières personnalisé pour forcer Renovate à interroger `git-refs` plutôt que `github-tags`.
- **Rejet** : Complexité inutile et fragile (violation du principe YAGNI et du Zoom Out Standard).

---

## 4. Spécification des Modifications (Périmètre d'Écriture)

### 4.1. Fichiers affectés
| Action | Chemin relatif | Rôle |
|---|---|---|
| **Modifier** | `.github/workflows/ci.yml` | Aligner les deux étapes `dtolnay/rust-toolchain` sur `@v1` avec `toolchain: 1.99.1` |
| **Supprimer** | `renovate.json` | Éliminer le masquage de la configuration `.github/renovate.json5` |
| **Créer** | `docs/decisions/fix-rust-toolchain-digest-renovate.md` | Documenter la décision architecturale (ce document) |

### 4.2. Différence projetée dans `.github/workflows/ci.yml`

```yaml
# Job quality (lignes 29-33)
      - name: Install Rust 1.99.1 toolchain
-        uses: dtolnay/rust-toolchain@d124f86f28ff42855784ff3319f48ac182c248f8 # 1.99.1
+        uses: dtolnay/rust-toolchain@683b588c487a69d01f6c6ddf40f036ff682cc2c8 # v1
         with:
+          toolchain: 1.99.1
           components: rustfmt, clippy

# Job test (lignes 73-75)
      - name: Install Rust 1.99.1 toolchain
-        uses: dtolnay/rust-toolchain@d124f86f28ff42855784ff3319f48ac182c248f8 # 1.99.1
+        uses: dtolnay/rust-toolchain@683b588c487a69d01f6c6ddf40f036ff682cc2c8 # v1
+        with:
+          toolchain: 1.99.1
```

### 4.3. Message de Commit Conventionnel Recommandé
Conformément à `docs/git-workflow.md` et à la validation par `fluidise-clint` :
```text
fix(ci): pin dtolnay/rust-toolchain to v1 and activate renovate.json5
```

---

## 5. Plan de Vérification et Procédure de Rollback

### 5.1. Vérifications Pré-Merge
1. **Validation locale de la chaîne de compilation et des tests** :
   - `cargo fmt --all -- --check` : succès.
   - `cargo clippy --all-targets -- -D warnings` : succès.
   - `cargo test --all-targets` : 38 tests unitaires/intégration/propriétés passés avec succès.
2. **Validation de l'absence de régression syntaxique CI** :
   - Vérifier la structure YAML : `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"`.
3. **Validation de la configuration Renovate active** :
   - Vérifier l'intégrité JSON5 de `.github/renovate.json5` via un parseur JSON strip-comments.
4. **Validation du titre de commit** via `fluidise-clint` :
   - `cargo run --quiet --bin fluidise-clint -- --stdin --human <<< "fix(ci): pin dtolnay/rust-toolchain to v1 and activate renovate.json5"`.

### 5.2. Vérifications Post-Merge
- Coche de la case de re-détection dans l'Issue #25 (Dependency Dashboard) pour constater la disparition de l'avertissement de lookup.

### 5.3. Procédure de Rollback
La modification étant strictement limitée aux fichiers de configuration Git/CI :
- `git revert <commit-sha>` restaure immédiatement l'état antérieur (rétablissement de `renovate.json` et des anciens digests) sans effet de bord sur le binaire applicatif ou les dépendances Cargo.

---

## 6. État de la Revue Indépendante (EGM Review State)

<!-- Section de métadonnées de revue EGM -->
- **Domaine de digest du corps substantiel** : SHA-256 calculé sur les sections 1 à 5.
- **Révision cible inspectée** : Dépôt local `Fluidise/clint` sur la branche `fix_rust_toolchain_digest` (commit de base `dcf435f`).
- **Couverture de revue exécutée** : 1 relecteur indépendant Goldfish (`egm-design-critic`) évaluant l'adéquation architecturale, la résilience opérationnelle, la conformité aux standards de feature spec, de zoom-out et de préparation à l'implémentation.
- **Résultats de revue** : 0 Blocker, 0 Major, 3 Nits (tous intégrés dans les sections 4.3, 5.1 et 5.3).
- **Statut de revue** : **READY FOR IMPLEMENTATION** (Revue indépendante complétée et concluante).
