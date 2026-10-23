# Boucle locale d'amélioration de fluidise-clint

> Rapport historique antérieur à [ADR 0002](adr/0002-explicit-policy-selection.md).
> Les anciens binaires utilisaient la découverte implicite. Pour les rejouer
> avec le harnais actuel, ajouter `--legacy-baseline --legacy-candidate` à
> `scripts/benchmark.py`. Pour comparer un ancien binaire au nouveau, ajouter
> seulement `--legacy-baseline`.

## Contrat et décisions

Objectif : améliorer les performances et la robustesse mesurées, sans nouvelle
fonctionnalité ni modification du contrat public. Maximum trois expériences ;
arrêt anticipé si aucune nouvelle hypothèse causale ne justifie une modification.
Le workflow et ces bornes ont été approuvés dans le chat le 20 octobre 2026
(« ok »), puis l'exécution a été autorisée par « go ».

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

## Référence et hypothèse

Référence source : `7573759d0d10a1f7e770fbbefa7db43c8822ea0b`. Au début de la
mesure, seul le fichier CI était modifié ; aucun script ni résultat brut du
benchmark historique n'a été trouvé dans les fichiers suivis du dépôt.
Les concurrents ne sont ni installés ni évalués ici.

Chemin observé : arguments, découverte de configuration, lecture bornée,
validation UTF-8, moteur, encodage et émission. Le moteur validait tout le
contenu, puis allouait un `Vec<(&str, usize)>` pour **chaque ligne**, avant
d'éliminer les lignes vides finales ou d'appliquer les exemptions.

Le pilote a mesuré environ 16,4 millions d'octets RSS pour 500 000 lignes de
corps et 28,4 millions pour un million de lignes vides finales. Cela n'invalide
pas la borne d'entrée de 1 MiB, mais montre son amplification par l'index de lignes.

**Expérience 1 :** remplacer cet index par un parcours paresseux, conserver les
offsets et examiner seulement les lignes requises pour les exemptions et les
diagnostics. La validation complète des NUL et CR isolés reste préalable : un
message ignoré ne doit pas permettre de contourner cette validation.

Les CR/LF finaux ne sont retirés qu'après validation du contenu. Puisque chaque
CR restant est suivi d'un LF, cette opération équivaut à l'ancien retrait des
lignes finales vides, y compris pour une entrée entièrement vide ou CRLF.

Documentation consultée : bibliothèque standard Rust via Context7,
[`str::trim_end_matches`](https://doc.rust-lang.org/stable/std/primitive.str.html#method.trim_end_matches).
Il s'agit de documentation stable, pas d'une validation du MSRV 1.85 ; le test
local utilise Rust 1.94.1. Aucune API plus récente que celles requises par le
projet n'est intentionnellement introduite.

## Protocole reproductible

Environnement observé : Apple M1 Pro, 16 Go, macOS 26.7.1 ARM64, Rust/Cargo
1.94.1, Python 3.14.3. Profil release inchangé : optimisation 3, LTO, une unité
de génération, panic abort, strip. Le driver moteur est compilé séparément
contre la bibliothèque release avec optimisation, LTO et panic abort.

Pour chaque snapshot, construire et figer les deux exécutables avant de passer
au suivant. Ne pas remplacer une référence source par un simple nom de branche.
Les dossiers de mesure restent sous `target/benchmark-loop/`, ignoré par Git.

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

Répéter pour le candidat en remplaçant `baseline` par `candidate`, puis :

```sh
python3 scripts/benchmark.py \
  target/benchmark-loop/baseline target/benchmark-loop/candidate \
  --output target/benchmark-loop/final.json
```

Le harnais ne construit rien, n'installe rien et ne modifie pas le dépôt. Il
vérifie les codes de sortie et compare exactement stdout/stderr ; le driver
compare la représentation complète du résultat du moteur. Les mesures CLI
utilisent un fichier dans un dossier temporaire extérieur à Git. Elles incluent
le coût de lancement Python/processus mais n'incluent pas un `git rev-parse` de
découverte : elles ne sont pas comparables directement aux chiffres historiques.

Chaque cas comprend trois préchauffages puis quinze échantillons, avec ordre
référence/candidat alterné. Le moteur réutilise une politique préparée, réalise
trois préchauffages par processus, puis chronomètre cent validations ; lecture
du fichier, préparation de politique et formatage du résultat sont exclus de
ce chronométrage. Le résultat est consommé par `black_box`.

Sur macOS, cinq mesures RSS par cas et binaire sont réalisées séparément avec
`/usr/bin/time -l`. La sandbox locale bloque son accès à `kern.clockrate` : les
mesures ont donc nécessité une exécution locale autorisée hors sandbox.

Le corpus comprend treize cas : petits messages valide/invalide, multiligne,
UTF-8 invalide, parenthèse non fermée de 100 000 caractères, nombreuses lignes,
lignes vides finales, nombreux footers, fusion avec gros corps, entrée de
20 MiB et frontières exactes 1 MiB moins un, exacte et plus un octet.

## Preuves et limites

### Résultats de confirmation — 21 octobre 2026

Médianes locales ; les valeurs RSS sont exprimées ici en millions d'octets
(Mo décimaux), et non en MiB.

| Cas | CLI avant / après | Moteur avant / après | RSS avant / après |
| --- | --- | --- | --- |
| 500 000 lignes de corps | 10,512 / 10,302 ms | 6,551 / 6,011 ms | 16,45 / 2,77 Mo |
| Un million de lignes vides finales | 15,172 / 5,420 ms | 10,650 / 1,112 ms | 28,44 / 2,77 Mo |
| 140 000 footers vides | 6,506 / 4,629 ms | 2,200 / 0,471 ms | 7,88 / 2,83 Mo |
| Fusion avec 500 000 lignes de corps | 9,716 / 4,661 ms | 5,641 / 0,473 ms | 16,45 / 2,77 Mo |

Le gain ciblé est confirmé : environ 90 % de temps moteur et 90 % de RSS en
moins sur les lignes vides finales, et environ 92 % de temps moteur en moins
sur la fusion à gros corps. Le parcours d'un corps ordinaire dense reste
linéaire : son gain moteur n'est que de 8 %, malgré une baisse RSS de 83 %.

Le petit message valide passe de 3,341 à 3,421 ms côté CLI (+2,4 %) ; aucune
des treize séries ne dépasse le filtre de régression matérielle défini plus
haut. Aucun gain général de démarrage n'est revendiqué. Les limites d'entrée
et les rejets volumineux ne changent pas.

Les treize cas comparent les sorties CLI complètes ; douze comparent également
le résultat complet du moteur. L'UTF-8 invalide est vérifié uniquement côté
CLI, puisque l'API moteur accepte déjà un `&str` valide.

Le pilote et une première comparaison ont précédé la confirmation. La première
comparaison a partiellement coïncidé avec la compilation des tests ; elle n'est
pas la source du tableau. La confirmation a été relancée sans compilation ni
tests concurrents pilotés par l'agent. Les charges extérieures et la fréquence
du processeur ne sont pas contrôlées.

### Artefacts locaux

Les échantillons bruts, médianes, écarts absolus médians, hash du corpus et
empreintes des sources/binaires sont disponibles dans
`target/benchmark-loop/final.json`. Les manifestes source sont aussi présents
dans les sous-dossiers `baseline/` et `candidate/` ; les exécutables mesurés y
sont conservés. Ces fichiers sont **locaux et ignorés par Git** : ils ne seront
pas inclus automatiquement dans une publication ou disponibles après un clone.

SHA-256 des exécutables CLI mesurés :

```text
baseline  bdf0aba66de060fce4029c60da24227dde78add37e512acb92552839e4a444b3
candidate 652cad25036405037b172110a30721a6114b34c49f452cc195937a9577e1e2e2
```

### Vérifications et bilan

- C1 : deux nouveaux tests caractérisent les fins de lignes LF/CRLF, les
  exemptions, les footers, les offsets Unicode et les entrées denses ; ils
  passent avant et après la modification. Les résultats complets des cas du
  benchmark sont identiques entre les binaires figés.
  Un contrôle différentiel supplémentaire compare exactement code de sortie,
  stdout et stderr sur 512 messages générés avec graine fixe et trois politiques
  (défaut, exemptions désactivées, règles restrictives) : **1 536 comparaisons,
  aucune divergence**. Le script, les messages, les politiques et les empreintes
  des binaires sont conservés localement dans
  `target/benchmark-loop/differential.py` et `differential.json`.
- C2 : aucun changement dans les modules d'entrée, configuration, transport,
  sortie ou dans les constantes de limites. Les tests existants du probe byte,
  de sortie bornée et de diagnostics restent passants ; les nouveaux tests
  vérifient aussi la priorité de limite de taille sur le CR isolé.
- C3 : expérience 1 conservée. Aucun lancement des expériences 2 et 3 : le gain
  ciblé est démontré et aucune autre modification n'est nécessaire à cet objectif.
- C4 : `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --all-targets` (30 tests) et `cargo test --doc` (2 tests) passent.
  `rustfmt --edition=2024 --check scripts/benchmark-engine.rs` et la compilation
  syntaxique Python passent également. Deux tests CLI de l'état initial ont
  d'abord échoué sur les restrictions sandbox de liens symboliques/FIFO ; ils
  passent sans modification une fois exécutés hors sandbox.

Revue indépendante : **proposée, non exécutée, choix utilisateur en attente**.
Elle est recommandée avant livraison, particulièrement pour l'équivalence du
traitement des lignes, des reverts et des offsets. Aucun commit, push ou PR
n'a été réalisé.

Le RSS mesuré n'est jamais une garantie de plafond mémoire. Les résultats ne
prouvent rien sur les versions actuelles des concurrents, leurs vulnérabilités,
la liaison statique du binaire, leurs dépendances ou leurs CVE. Un timeout reste
uniquement un dépassement de délai observé. Aucune CI distante, compatibilité
multiplateforme, installation de hook ou publication n'est validée ici.

## Intégration pour livraison

La livraison reprend uniquement les six fichiers de cette tâche sur
`main` à `4f60848674b49a4c8e6bda943eeb02dfd3a6e979`, dans la branche
`chore/bounded-line-iteration`. Les let-chains de la modernisation Rust déjà
intégrée dans main sont conservées. La CI préexistante du checkout de travail
n'est ni copiée ni incluse.

Les mesures ci-dessus restent celles des binaires Rust 1.94.1 identifiés ; elles
ne sont pas présentées comme de nouvelles mesures du candidat Rust 1.99.1.
Le contrôle de livraison a validé format, Clippy, 30 tests, 2 doctests et le
build release avec la toolchain 1.99.1 du main actuel. Les artefacts locaux restent dans le checkout
initial, et non dans ce worktree de livraison.
