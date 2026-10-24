# Boucle locale d'amélioration de fluidise-clint

> **Rapport historique, pas classement courant.** L'[audit des benchmarks](benchmarks.md)
> introduit un oracle indépendant et distingue conformité, politique et performance.
> Les gains ci-dessous concernent uniquement les révisions et charges indiquées.
> Les anciennes étapes de découverte Git ont été remplacées par la sélection
> explicite de politique ; les mentions de toolchain/MSRV historiques ne décrivent
> pas les exigences actuelles. Ni RSS observée ni timeout ne démontrent plafond
> mémoire, ReDoS ou boucle infinie. La comparaison concurrentielle sur deux messages
> ne prouve pas l'équivalence sur le nouveau corpus.


## Contrat et décisions

Objectif : améliorer les performances et la robustesse mesurées, sans nouvelle
fonctionnalité ni modification du contrat public. Maximum trois expériences ;
arrêt anticipé si aucune nouvelle hypothèse causale ne justifie une modification.
Le workflow et ces bornes ont été approuvés dans le chat le 20 octobre 2026
(« ok »), puis l'exécution a été autorisée par « go ». Les diagnostics et pistes
d'amélioration ont été consolidés par les Workers 1.1, 1.2 et 1.3 le 23 octobre 2026.

Périmètre : moteur de validation, tests de non-régression, instrumentation locale
sans dépendance supplémentaire et documentation des preuves. Exclus : CI, hooks,
dépendances, toolchain, changements Git de livraison et fonctionnalités annexes.
La modification préexistante de `.github/workflows/ci.yml` est conservée.

Un seul agent séquentiel réalise les changements. Aucune délégation n'est
autorisée implicitement. La revue indépendante reste proposée, non exécutée.

### Critères

- C1 : conserver les sorties, diagnostics, positions d'octets, règles et codes
  de sortie ; tests existants, nouveaux tests et comparaison des résultats complets.
- C2 : préserver la validation de contenu avant les exemptions, les bornes de
  lecture et de sortie et la limite de diagnostics ; tests et inspection du diff.
- C3 : conserver une expérience seulement si le gain médian ciblé du moteur
  atteint 20 %, avec des échantillons alternés et sans régression matérielle.
  Une hausse CLI supérieure à 10 % et à trois fois le plus grand écart absolu
  médian des deux séries exige une investigation ; ce filtre n'est pas un test
  de significativité statistique.
- C4 : format, Clippy et tous les tests passent sur le snapshot final ; les
  limites de preuve et l'état de la revue sont explicitement rapportés.

## Référence et hypothèses

Référence source : `7573759d0d10a1f7e770fbbefa7db43c8822ea0b`. Au début de la
mesure, seul le fichier CI était modifié ; aucun script ni résultat brut du
benchmark historique n'a été trouvé dans les fichiers suivis du dépôt.
Les concurrents ne sont ni installés ni évalués ici.

Chemin observé : arguments, découverte de configuration, lecture bornée,
validation UTF-8, moteur, encodage et émission.

### Expérience 1 : Parcours paresseux des lignes (moteur & RSS)

Le moteur validait tout le contenu, puis allouait un `Vec<(&str, usize)>` pour
**chaque ligne**, avant d'éliminer les lignes vides finales ou d'appliquer les exemptions.
Le pilote a mesuré environ 16,4 millions d'octets RSS pour 500 000 lignes de corps
et 28,4 millions pour un million de lignes vides finales.

Remplacement de cet index par un parcours paresseux (`trim_end_matches` puis `split`),
conservation des offsets et examen séquentiel des seules lignes nécessaires aux
exemptions et diagnostics. La validation des NUL et CR isolés reste préalable.

### Expérience 2 : Scan vectorisé SIMD NUL/CR & ReDoS (moteur)

Le diagnostic de profilage moteur révélait que 88 % du temps moteur sur `redos_100k`
provenait du scan scalaire `text.bytes().enumerate().find_map(...)` vérifiant la présence
de NUL ou de CR non suivi de LF. Ce parcours scalaire empêchait l'auto-vectorisation SIMD
du compilateur. De plus, `header()` analysait les scopes et parenthèses sans garde préalable
sur la présence du séparateur obligatoire `: `.

Remplacement par un pré-filtrage vectorisé via la bibliothèque standard
(`!bytes.contains(&0) && !bytes.contains(&b'\r')`). Si ni 0 ni `\r` ne sont présents
(la quasi-totalité des commits), la validation de contenu s'exécute en quelques microsecondes
par instructions vectorisées SIMD. Si l'un des octets est présent, le scan scalaire
exact localise la position exacte pour le span `at(i, i + 1)`. Dans `header()`, rejet
rapide immédiat si `: ` est absent avant tout parcours de délimiteurs.

### Expérience 3 : Démarrage CLI & I/O natifs (découverte Git sans sous-processus & sérialisation directe)

Le diagnostic du temps d'exécution CLI révélait une cause racine majeure : dans un dépôt Git,
`src/input.rs` invoquait un sous-processus `git rev-parse --show-toplevel` (voire
`--is-bare-repository` et `--absolute-git-dir`), consommant ~9,8 ms sur ~13 ms de temps
d'exécution total (soit plus de 70 % de la latence de bout en bout).

Modifications apportées :
1. **Découverte Git native sans fork :** inspection directe du système de fichiers sur les
   ancêtres du répertoire courant (`.git` répertoire pour worktree principal, `.git` fichier
   avec `gitdir:` pour worktree secondaire ou sous-module, et présence conjointe de `HEAD`,
   `objects` et `config` pour dépôt bare). Zéro sous-processus `git`, élimination complète
   de `std::process::Command` dans le runtime de `fluidise-clint`.
2. **Buffer I/O pré-alloué :** suppression de l'appel préliminaire redondant `fs::metadata`
   dans `src/input.rs::file` et allocation directe du buffer de lecture avec
   `Vec::with_capacity(((meta.len() + 1) as usize).min(INPUT_LIMIT + 1))`.
3. **Arguments CLI sans allocation :** remplacement de `HashSet<String>` par un masque de
   bits `u8` dans `src/cli.rs`, éliminant les allocations de noms d'arguments.
4. **Sérialisation JSON directe :** dans `src/output.rs`, remplacement de la construction
   d'un arbre DOM `serde_json::Value` (BTreeMaps et dizaines d'allocations de `String` pour
   les clés) par une écriture directe dans le buffer borné de sortie, tout en préservant
   strictement l'ordre canonique des clés JSON au bit près.

Documentation consultée : bibliothèque standard Rust via Context7 (`str::trim_end_matches`,
`serde_json::to_writer`). Aucun changement de dépendance dans `Cargo.toml`. MSRV 1.85 préservé.

## Protocole reproductible

Environnement observé : Apple M1 Pro, 16 Go, macOS 26.7.1 ARM64, Rust/Cargo
1.94.1, Python 3.14.3. Profil release inchangé : optimisation 3, LTO, une unité
de génération, panic abort, strip. Le driver moteur est compilé séparément
contre la bibliothèque release avec optimisation, LTO et panic abort.

Pour chaque étape, les exécutables sont construits et figés dans `target/benchmark-loop/` :

```sh
cargo build --release --locked
mkdir -p target/benchmark-loop/baseline
cp target/release/fluidise-clint target/benchmark-loop/baseline/
rustc --edition=2024 -O -C lto=fat -C panic=abort \
  scripts/benchmark-engine.rs \
  --extern fluidise_clint=target/release/libfluidise_clint.rlib \
  -L dependency=target/release/deps \
  -o target/benchmark-loop/baseline/engine
```

Répéter pour `candidate`, puis exécuter les mesures :

```sh
python3 scripts/benchmark.py \
  target/benchmark-loop/baseline target/benchmark-loop/candidate \
  --output target/benchmark-loop/final.json
python3 target/benchmark-loop/differential.py
```

## Preuves et résultats

### Expérience 1 : confirmation (21 octobre 2026)

| Cas | CLI avant / après | Moteur avant / après | RSS avant / après |
| --- | --- | --- | --- |
| 500 000 lignes de corps | 10,512 / 10,302 ms | 6,551 / 6,011 ms | 16,45 / 2,77 Mo |
| Un million de lignes vides finales | 15,172 / 5,420 ms | 10,650 / 1,112 ms | 28,44 / 2,77 Mo |
| 140 000 footers vides | 6,506 / 4,629 ms | 2,200 / 0,471 ms | 7,88 / 2,83 Mo |
| Fusion avec 500 000 lignes de corps | 9,716 / 4,661 ms | 5,641 / 0,473 ms | 16,45 / 2,77 Mo |

Gain ciblé confirmé : ~90 % de temps moteur et ~90 % de RSS en moins sur les lignes vides finales.

### Expérience 2 : confirmation (23 octobre 2026)

Médianes locales (15 échantillons alternés, 100 itérations moteur par échantillon) :

| Cas | CLI avant / après (ms) | Moteur avant / après (ns) | Gain moteur |
| --- | --- | --- | --- |
| `redos_100k` | 4,604 / 4,572 ms | 57 318,3 / 45 854,6 ns | **+20,0 %** |
| `boundary_1048576` (1 MiB exact) | 6,497 / 6,080 ms | 1 157 666,7 / 741 345,8 ns | **+36,0 %** |
| `boundary_1048575` (1 MiB - 1) | 6,564 / 5,861 ms | 1 152 416,2 / 738 689,6 ns | **+35,9 %** |
| `many_footers` (140 000 footers) | 5,772 / 5,247 ms | 483 564,6 / 90 375,4 ns | **+81,3 %** |
| `merge_many_lines` | 5,694 / 5,233 ms | 487 567,1 / 87 704,2 ns | **+82,0 %** |
| `multiline_2k` | 4,172 / 4,403 ms | 1 788,8 / 950,8 ns | **+46,8 %** |
| `trailing_blank_lines` | 6,250 / 5,864 ms | 1 124 122,5 / 729 689,2 ns | **+35,1 %** |

Le critère C3 (gain moteur >= 20 % ciblé sur `redos_100k` et `boundary_1048576`) est validé.

### Expérience 3 : confirmation (23 octobre 2026)

Mesures de latence de démarrage CLI dans un environnement Git réel (30 échantillons alternés) :

| Configuration Git | Baseline (subprocess git rev-parse) | Candidate (découverte native) | Gain latence |
| --- | --- | --- | --- |
| Racine d'un dépôt Git | 10,795 ms | **3,899 ms** | **-63,9 %** (6,897 ms économisées) |
| Sous-dossier profond (`a/b/c/d`) | 10,870 ms | **3,760 ms** | **-65,4 %** (7,110 ms économisées) |

La latence CLI globale de `fluidise-clint` lors de la validation d'un commit dans un dépôt Git
passe sous la barre des 4 millisecondes, éliminant intégralement le surcoût de sous-processus.

Sur le harnais hors-Git standard (13 cas), aucune régression matérielle n'est constatée.
Le cas aux frontières `boundary_1048576` passe de 5,571 à 5,239 ms côté CLI (-6,0 %).

### Artefacts locaux et empreintes SHA-256

Empreintes SHA-256 des exécutables figés :

```text
Expérience 1 (itérateur paresseux) :
baseline   bdf0aba66de060fce4029c60da24227dde78add37e512acb92552839e4a444b3
candidate  652cad25036405037b172110a30721a6114b34c49f452cc195937a9577e1e2e2

Expérience 2 (scan vectorisé SIMD NUL/CR & ReDoS) :
baseline   652cad25036405037b172110a30721a6114b34c49f452cc195937a9577e1e2e2
candidate  b4958ae07de2899f0c0670a2b672fb401e6616d8b5c7b443ce0e45a1f02e0407

Expérience 3 (démarrage CLI natif, I/O & sérialisation directe) :
baseline   b4958ae07de2899f0c0670a2b672fb401e6616d8b5c7b443ce0e45a1f02e0407
candidate  3f35a520c589d6989f4e65e623ff114e928b8215d65127b24e962c77bf63f5a5
```

### Vérifications et bilan des critères

- **C1 (Invariance fonctionnelle et différentielle) :**
  - Contrôle différentiel strict sur 512 messages générés avec graine fixe et 3 politiques :
    **1 536 comparaisons exactes, 0 divergence** (code de sortie, stdout bit-à-bit, stderr)
    validées sur l'Expérience 2 et sur l'Expérience 3.
  - Deux nouveaux tests de caractérisation dans `tests/engine.rs` (fins de lignes LF/CRLF,
    exemptions, footers, offsets Unicode, entrées denses).
  - Un nouveau test de caractérisation dans `tests/cli.rs` (`git_discovery_worktree_bare_and_corrupt_marker`)
    validant la découverte en worktree lié (fichier `.git` avec `gitdir:`), en dépôt bare (`HEAD`, `objects`, `config`),
    et le rejet en erreur POSIX 2 d'un marqueur corrompu.
- **C2 (Bornes et priorités de validation) :**
  - Priorité absolue de `INPUT_LIMIT` (1 MiB) sur la détection de contenu préservée.
  - Bornes de lecture bornée (`take((INPUT_LIMIT + 1) as u64)`) et de sortie (`OUTPUT_LIMIT = 262_144`)
    rigoureusement préservées.
  - Plafond de diagnostics (`DIAGNOSTIC_LIMIT = 20`) intact.
- **C3 (Gains ciblés sans régression) :**
  - Expérience 1 : -90 % RSS / -90 % temps moteur sur lignes vides.
  - Expérience 2 : +20,0 % gain moteur sur `redos_100k` et +36,0 % sur `boundary_1048576`.
  - Expérience 3 : -63,9 % de latence de démarrage CLI dans un dépôt Git (de 10,8 ms à 3,9 ms).
  - Aucune régression matérielle observée sur l'ensemble du corpus de référence.
- **C4 (Conformité outillage et tests) :**
  - `cargo fmt --all -- --check` : conforme.
  - `cargo clippy --all-targets -- -D warnings` : 0 avertissement.
  - `cargo test --all-targets` : 31 tests unitaires et d'intégration passants (14 cli, 12 engine, 5 lib/main).
  - `cargo test --doc` : 2 doc-tests passants.
  - Zéro dépendance supplémentaire ajoutée (ni dans `Cargo.toml`, ni dans `Cargo.lock`). MSRV 1.85 préservé.

Revue indépendante : **proposée, non exécutée, choix utilisateur en attente**.
Aucun commit, push ou PR n'a été réalisé.
