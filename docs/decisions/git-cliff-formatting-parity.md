# Décision : Alignement du formatage Git-Cliff avec le dépôt Relay

## Contexte et Mandat

- **Objectif** : Vérifier et assurer la stricte parité de configuration `git-cliff` et de formatage de changelog/release notes entre le dépôt `fluidise-clint` (`Fluidise/clint`) et le dépôt de référence `relay` (`Fluidise/relay`).
- **Mode d'exécution** : Mode 2 (Décisions + Documentation + Implémentation locale et validation).
- **Périmètre** : Configuration `cliff.toml`, structure du fichier `CHANGELOG.md`, intégration GitHub Release et génération des notes de release.

---

## Analyse comparative (Frontière de décision)

L'inspection de la configuration de référence sous `/Users/acrazie/.gemini/antigravity/worktrees/relay/add_git_cliff_changelog/cliff.toml` a mis en évidence plusieurs écarts structuraux et visuels avec la configuration initiale de `clint` :

| Élément | Format Clint initial | Format Référence Relay | Décision d'alignement |
|---|---|---|---|
| **Emoji Refactor** | `🚜 Refactor` | `🔄 Refactor` | Aligné sur `🔄 Refactor` |
| **Emoji Features** | `⛰️ Features` | `🚀 Features` | Aligné sur `🚀 Features` |
| **Emoji Build** | Non groupé séparément | `📦 Build & Packaging` | Ajouté |
| **Emoji CI** | Fusionné dans Miscellaneous | `👷 Continuous Integration` | Séparé sous `👷 CI` |
| **Emoji Miscellaneous** | `⚙️ Miscellaneous Tasks` | `⚙️ Miscellaneous Tasks` | Conservé |
| **Format du Scope** | `*(scope)* ` (italique parenthèses) | `**scope:** ` (gras + deux-points) | Aligné sur `**scope:** ` |
| **Entête Changelog** | Entête minimaliste | Entête complet [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) & [SemVer](https://semver.org/spec/v2.0.0.html) | Aligné sur Keep a Changelog |
| **Pied de page (Footer)** | Commentaire HTML seul | Liens de comparaison de version Keep a Changelog (`compare/v...`) | Aligné sur les liens de comparaison |
| **Ordre des commits** | `"newest"` | `"oldest"` (chronologique par section) | Aligné sur `"oldest"` |
| **Filtre commits non-conv.** | `filter_unconventional = true` | `filter_unconventional = false` | Aligné sur `false` |
| **Filtre commits release CI** | `chore(release)` seul | Ignore également `^chore.*[sS]kip [cC][iI]` | Aligné |

---

## Décisions arrêtées

1. **Adoption intégrale du template et des parseurs de `relay`** :
   - Mise à jour de [`cliff.toml`](file:///Users/acrazie/.gemini/antigravity/worktrees/lint/prepare_official_release/cliff.toml) avec les emojis unifiés de l'organisation Fluidise (`🚀`, `🐛`, `⚡`, `🔄`, `📚`, `🎨`, `🧪`, `📦`, `👷`, `◀️`, `⚙️`, `💥`).
   - Formatage des scopes sous la forme `**scope:** ` et liaison stricte des commits et PRs vers `Fluidise/clint`.
   - Entête et pied de page conformes à Keep a Changelog 1.1.0 et Semantic Versioning 2.0.0.

2. **Rétro-application sur la release GitHub v0.1.0** :
   - Les notes de la release officielle [v0.1.0](https://github.com/Fluidise/clint/releases/tag/v0.1.0) sur GitHub ont été régénérées avec ce formatage aligné, préservant la mention obligatoire de politique de signature non-signée pour 0.1.0.

3. **Génération du fichier de référence `CHANGELOG.md`** :
   - Régénération complète du fichier [`CHANGELOG.md`](file:///Users/acrazie/.gemini/antigravity/worktrees/lint/prepare_official_release/CHANGELOG.md) à la racine du dépôt.

4. **Activation de la section Contributeurs et déclenchement des avatars GitHub Releases** :
   - **Mécanisme GitHub identifié** : L'affichage natif du pied de page `Contributors` avec les avatars ronds des profils sur les GitHub Releases n'est pas un composant markdown généré par l'outil, mais un composant d'interface natif de GitHub (conteneur HTML `Box-footer`). GitHub n'injecte ce composant que lorsque la note de release contient au moins une mention explicite `@username` (`mentions_count > 0` dans l'API Releases).
   - **Configuration `cliff.toml`** :
     - Déclaration explicite du remote GitHub (`[remote.github]` avec `owner = "Fluidise"` et `repo = "clint"`).
     - Ajout de la section `### ❤️ New Contributors` pour les primo-contributeurs (`is_first_time: true`).
     - Ajout systématique de la section `### 👥 Contributors` énumérant tous les contributeurs de la version.
     - Filtrage systématique des comptes d'automatisation (`[bot]`, ex. `renovate[bot]`).
   - **Effet** : Les mentions `@username` permettent à GitHub d'afficher nativement le carrousel d'avatars sous la release, tout en garantissant un historique clair des contributions dans `CHANGELOG.md`.

---

## Validation et Résultats

- `git-cliff --output CHANGELOG.md` exécuté avec succès (avec sections `❤️ New Contributors` et `👥 Contributors`).
- `gh release edit v0.1.0` appliqué et vérifié : `mentions_count: 1`, section HTML `Box-footer` contenant les avatars de contributeurs active.
- `cargo fmt --all -- --check` : conforme.
- `cargo clippy --all-targets -- -D warnings` : aucun avertissement.
- `cargo test --all-targets` : 38 tests unitaires/intégration et 2 doctests passés avec succès.
- `python3 -m unittest discover -s tests -p 'test_distribution.py'` : 20 tests d'invariants passés avec succès.
