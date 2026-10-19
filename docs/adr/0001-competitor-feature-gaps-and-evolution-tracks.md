# ADR 0001: Évaluation des Écarts Fonctionnels Face aux Concurrents et Pistes d'Évolution

* **Statut :** Accepté (Pistes d'exploration approuvées)
* **Date :** 2026-10-20
* **Contexte :** Benchmark comparatif à 5 outils (`fluidise-clint`, `commitlint`, `cocogitto`, `convco`, `commitizen`) et analyse de la matrice de fonctionnalités.

---

## 1. Contexte

Le benchmark approfondi confrontant `fluidise-clint` aux leaders du marché (`commitlint` en Node.js, `cocogitto` et `convco` en Rust, `commitizen` en Python) a mis en évidence la suprématie de `fluidise-clint` en matière de **latence** (~13 ms), de **sobriété processeur** (18,5 M d'instructions vs 2,13 milliards pour `commitlint`), de **consommation mémoire résidente** (7,08 Mo) et d'**invariants de sécurité** (mémoire strictement bornée à 1 MiB).

Cependant, l'analyse comparative détaillée des fonctionnalités a révélé un ensemble de capacités offertes par les concurrents mais actuellement absentes de `fluidise-clint`. Ce document formalise :
1. Les fonctionnalités **délibérément exclues** du périmètre pour préserver nos invariants fondamentaux (Non-Goals).
2. Les **pistes d'amélioration prioritaires** à explorer pour étendre la couverture fonctionnelle de `fluidise-clint` sans compromettre ses performances ni sa fiabilité.

---

## 2. Décisions Fondatrices : Fonctionnalités Exclues (Non-Goals Assumés)

Afin de préserver la philosophie Unix (*« Ne faire qu'une seule chose et la faire de manière déterministe »*) et la sécurité des agents autonomes, les fonctionnalités suivantes sont expressément exclues du cœur de `fluidise-clint` :

| Fonctionnalité concurrente | Présente dans | Statut pour `clint` | Rationale & Justification |
| :--- | :---: | :---: | :--- |
| **Assistant interactif (TUI Wizard / Prompt)** | `commitizen`, `cocogitto`, `convco` | **Rejeté (Non-Goal)** | `clint` est conçu pour les agents IA et les hooks CI non interactifs. Un wizard terminal alourdirait le binaire sans valeur ajoutée pour les pipelines autonomes. |
| **Gestion de release SemVer (`bump`)** | `cocogitto`, `commitizen`, `convco` | **Rejeté (Non-Goal)** | Le calcul de version sémantique relève de l'orchestration de release, pas de la validation d'un message. Cette responsabilité est déléguée à des outils spécialisés (ex: `git-cliff`). |
| **Génération automatique de Changelog** | `cocogitto`, `commitizen`, `convco` | **Rejeté (Non-Goal)** | Générer un changelog implique des écritures sur le disque, en violation de notre principe de séparation stricte des responsabilités. |
| **Auto-installation de Git Hooks (`init`)** | `cocogitto`, `commitizen` | **Rejeté (Non-Goal)** | `clint` respecte un invariant strict de **non-mutation** : il ne modifie jamais `.git`, n'altère aucun fichier et ne s'auto-installe pas en sous-main. |
| **Extensions de configuration distantes (`extends`)** | `commitlint` (npm packages) | **Rejeté (Non-Goal)** | `clint` interdit formellement tout appel réseau ou chargement de modules dynamiques pour préserver une exécution hermétique et instantanée. |

---

## 3. Pistes d'Amélioration et d'Exploration Prioritaires

Quatre fonctionnalités absentes constituent de véritables opportunités d'amélioration pour les prochaines versions de `fluidise-clint` :

### Piste 1 : Nettoyage des Commentaires Git et Lignes de Ciseaux (`--strip-comments`)
* **Problème identifié :**
  Lorsqu'un développeur exécute `git commit -v`, Git génère des commentaires (`#`) et insère une ligne de ciseaux (`# ------------------------ >8 ------------------------`) suivie du diff complet dans `.git/COMMIT_EDITMSG`.
  Actuellement, `fluidise-clint` lit les octets bruts sans nettoyage, ce qui peut générer des faux positifs si le fichier brut de hook contient des commentaires ou des lignes de diff verbeuses.
* **Proposition d'évolution :**
  - Introduire une option en ligne de commande `--strip-comments` et un paramètre de configuration `strip-comments = true` dans `[commit]` de `fluidise.toml`.
  - Le filtre élimine en streaming les lignes débutant par `#` (ou par le `core.commentChar` configuré) ainsi que toute section postérieure à la ligne de ciseaux `>8`, **avant** l'évaluation grammaticale du commit.
  - *Invariant respecté :* L'opération se fait en mémoire en lecture seule, sans jamais modifier le fichier sur disque.

### Piste 2 : Validation d'un Range de Commits Git (`--rev-range`)
* **Problème identifié :**
  Dans les pipelines de CI (validation d'une Pull Request comportant plusieurs commits) ou lors d'audits d'historique, `clint` ne peut analyser qu'un seul commit par invocation. Les utilisateurs doivent orchestrer une boucle shell externe (`git log --format=%B | ...`) pour valider un ensemble de commits.
* **Proposition d'évolution :**
  - Explorer l'ajout d'une option `--rev-range <range>` (ex: `origin/main..HEAD` ou `HEAD~5..HEAD`).
  - Implémentation via une lecture streaming des commits depuis Git, en maintenant une mémoire résidente bornée par commit.
  - Le rapport JSON émis sur `stdout` agrégerait les diagnostics par SHA de commit :
    ```json
    {
      "schema_version": 1,
      "status": "invalid",
      "commits": [
        { "sha": "a1b2c3d", "status": "valid", "diagnostics": [] },
        { "sha": "e4f5a6b", "status": "invalid", "diagnostics": [...] }
      ]
    }
    ```

### Piste 3 : Règles Granulaires de Casse et de Ponctuation (`header-case`, `full-stop`)
* **Problème identifié :**
  La spécification Conventional Commits recommande une description en minuscules (`lowercase`) et sans point final. Bien que `clint` valide la présence d'une description non vide, il n'impose pas de contraintes de casse sur le sujet, fonctionnalités couramment exploitées dans `commitlint`.
* **Proposition d'évolution :**
  - Ajouter des options déclaratives dans `fluidise.toml` sous la section `[commit]` :
    ```toml
    [commit]
    subject-case = "lowercase"         # "lowercase" | "any" (défaut)
    forbid-trailing-period = true      # booléen, interdit le point final
    min-title-length = 10              # borne inférieure optionnelle
    ```
  - Chaque violation émet un diagnostic unitaire dédié avec offset d'octet précis (`commit.description.casing`, `commit.description.trailing_period`).

### Piste 4 : Support Monorepo et Scopes Délimités par Sous-Dossier
* **Problème identifié :**
  Dans un monorepo complexe hébergeant plusieurs bibliothèques ou services, la liste `allowed-scopes` globale est partagée pour tout le projet. `cocogitto` gère cela via une section `[packages]` permettant des scopes spécifiques par répertoire.
* **Proposition d'évolution :**
  - Permettre dans `fluidise.toml` la déclaration de profils ou de sous-sections par chemin :
    ```toml
    [commit.scopes-by-path]
    "crates/core" = ["engine", "alloc"]
    "crates/cli" = ["args", "transport"]
    ```
  - Lors de la validation via `--file` ou en présence d'un contexte de chemin, `clint` restreint les scopes valides au sous-projet concerné.

---

## 4. Conséquences et Prochaines Étapes

1. **Priorisation :**
   - **Priorité 1 :** Nettoyage des commentaires Git (`--strip-comments`) — impact direct sur la fiabilité des hooks locaux `commit-msg`.
   - **Priorité 2 :** Règles de casse et ponctuation (`subject-case`, `forbid-trailing-period`) — enrichissement des capacités sans complexité structurelle.
   - **Priorité 3 :** Validation de range Git (`--rev-range`) — confort pour les workflows de Pull Request.
   - **Priorité 4 :** Scopes monorepo par chemin.
2. **Critères d'Invariance Non Négociables :**
   Toute implémentation future d'une de ces pistes devra prouver :
   - Le maintien strict de la borne d'entrée d'1 MiB et du plafond mémoire résidente (~7 Mo).
   - L'absence d'effets de bord (zéro écriture sur disque, lecture seule absolue).
   - L'émission exclusive de JSON strict par défaut avec codes d'erreur structurés.
