# Benchmarks : justesse, contrats et performances CLI

## Conclusion de l'audit du 23 octobre 2026

Les anciens classements ne constituent pas une preuve suffisante pour choisir un
outil ou une optimisation. Ils sont retirés de cette page : leurs données brutes
et commandes complètes n'étaient pas disponibles dans les fichiers suivis.
Les deux rapports JSON historiques conservés dans [benchmarks/](benchmarks/)
concernent une autre mesure, plus étroite, de processus CLI.

Constats reproductibles dans les anciens harnais :

- `benchmark-agent.py` chronomètre seulement un petit message valide et un invalide.
  Ses 52 contrôles étendus comparent clint à lui-même : une erreur commune peut
  rester invisible. Ils ne démontrent pas l'équivalence fonctionnelle concurrentielle.
- `benchmark.py` couvre 13 charges utiles pour caractériser clint avant/après,
  mais ne possède pas d'oracle indépendant. Ses contrôles utilisaient `assert`,
  désactivable par `python -O` ; ils sont désormais explicites. Les empreintes
  des exécutables sont également revérifiées après mesure.
- Les configurations, exemptions et sorties des outils diffèrent. Refuser une
  entrée de plus de 1 MiB et valider intégralement cette entrée ne sont pas le
  même travail. Un rejet de merge n'est pas nécessairement un faux positif.
- Les tableaux du README combinaient des révisions et des protocoles différents.
  Le nouveau protocole ne réutilise pas leurs ratios.
- Une limite d'entrée ne constitue pas un plafond RSS. Un timeout ne prouve ni
  ReDoS, ni boucle infinie, ni origine précise du ralentissement. Aucun classement
  de sécurité, de CVE ou de supply chain ne découle de ces temps.

## Question mesurée et oracle

Le premier objectif est d'éviter des messages incorrectement acceptés ou refusés.
La latence n'est interprétable qu'après vérification du contrat.

Le corpus [benchmark-corpus.json](../tests/benchmark-corpus.json) contient 37 cas :

- 12 cas du socle Conventional Commits : en-tête, scope, rupture, corps, footers,
  Unicode et erreurs structurelles ;
- 8 cas de politique et transport propres à clint ;
- 7 cas de configuration : remplacement des types, scope sensible à la casse,
  longueur Unicode et erreurs opérationnelles ;
- 3 messages publics : deux messages complets de clint et un sujet de commitlint ;
- 7 charges de robustesse : limite exacte 1 MiB ±1 octet, scopes non fermés à
  trois longueurs et UTF-8 invalide.

Chaque cas possède origine, justification, référence, résultat attendu et domaine.
Les verdicts sont dérivés de la [spécification 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/#specification)
ou du contrat documenté de clint, **pas d'un vote entre outils ni de la sortie
actuelle de clint**. Ce corpus est un oracle explicite et révisable, pas une
certification exhaustive de conformité. Sa rédaction n'a pas fait l'objet d'une
revue indépendante.

Points de séparation importants :

- La spécification autorise d'autres types ; `maintenance:` rejeté par les défauts
  de clint est une différence de politique, pas une violation universelle.
- La casse du type, les exemptions Git, les temporaires, les tailles et les
  configurations personnalisées sont isolés des comparaisons partagées.
- Les messages réels sont sélectionnés intentionnellement, pas aléatoirement.
  Ils ne représentent pas une fréquence d'erreurs observée en production.
  Le sujet seul est identifié comme tel ; aucun historique privé n'est utilisé.
- Aucun taux d'exactitude global ne mélange cas réels, synthétiques et politiques.
  Les comptes par groupe distinguent acceptations erronées, rejets erronés,
  erreurs opérationnelles, timeouts et exclusions.

## Réévaluation des cinq outils

Le [nouveau harnais](../scripts/benchmark_suite.py) utilise uniquement Python
standard, Git et les exécutables locaux fournis explicitement. Il n'installe rien.

| Outil | Contrat du pilote | État local de cette évaluation |
| --- | --- | --- |
| clint | `--stdin --defaults`, ou configuration du cas ; JSON et codes 0/1/2 | Exécuté, 37/37 contrôles réussis |
| cocogitto | `--config … verify --file -`, TOML vide | 7.0.0 exécuté ; divergence sur corps sans ligne vide |
| convco | `check --config … --from-stdin`, `{}` | 0.7.2 exécuté ; même divergence |
| commitlint | stdin, configuration CJS explicite, `--strict` ; 3 signifie rejet | 21.2.4 exécuté sur macOS et Linux ARM64 : 15/15 cas partagés |
| commitizen | stdin, `--config … check`, `cz_conventional_commits` ; 14/32 signifient rejet | 4.19.3 exécuté sur macOS et Linux ARM64 : 15/15 cas partagés |

La politique commitlint du pilote est volontairement limitée : type/sujet requis,
énumération des types et ligne vide avant le corps. Des options de parsing
explicites prennent en charge scope et `!` sans ajouter de paquet de preset. Elle n'est pas présentée comme
le preset `config-conventional` complet. Les sorties natives ne sont pas tenues
pour du JSON ni pour équivalentes aux diagnostics structurés de clint. Les codes
opérationnels connus et timeouts sont séparés des rejets. Un code natif ambigu
(comme 1 pour cog/convco) ne suffit pas à distinguer toutes les erreurs internes
d'un rejet syntaxique ; les sorties restent conservées pour examen. Les contrôles
positifs réduisent ce risque sans constituer une preuve exhaustive.

Les premières preuves du 23 octobre ne couvraient que trois outils. L'intégration
réelle du 24 octobre couvre désormais les cinq sur macOS, et clint/commitlint/
Commitizen sur Linux ARM64. Elle a révélé un défaut du harnais : Commitizen
n'accepte pas `cz --version`, mais expose `cz version`. Le pilote utilise désormais
cette commande native, avec un test de non-régression. Ce correctif et les nouvelles
preuves ne sont pas couverts par la revue indépendante du snapshot précédent.

Sources d'interface consultées via Context7 les 23 et 24 octobre 2026 :
[commitlint CLI](https://commitlint.js.org/reference/cli.html),
[Commitizen check](https://commitizen-tools.github.io/commitizen/commands/check/),
[Commitizen exit codes](https://commitizen-tools.github.io/commitizen/exit_codes/),
[cog verify](https://github.com/cocogitto/cocogitto/blob/main/website/guide/verify.md),
[convco](https://github.com/convco/convco).
Les versions installées de cog et convco ont aussi été interrogées avec `--help`.

## Résultats locaux et conséquence

Les [données brutes macOS](benchmarks/oracle-macos-2026-10-23.json) contiennent
versions, empreintes, commandes, politiques, attentes, sorties et échantillons.
Le binaire clint a été construit par `cargo build --release --locked` avec
Rust 1.99.1 depuis le code produit de
`84f77dea947839da8ca70c0d47dda68bbfcb9c80` ; ce lot ne modifie ni ce code,
ni les dépendances ni le profil release.

Sur les 15 cas partagés (12 synthétiques et 3 publics), cog 7.0.0 et convco 0.7.2
acceptent `fix: prevent duplicate\nExplain cause.`. Le cas attend un rejet : la
règle 6 exige une ligne vide avant le corps. Les 14 autres cas partagés passent.
Cette divergence concerne ces versions et commandes, pas toutes les versions,
ni toutes les possibilités de configuration de ces produits.

**Conséquence : les deux concurrents sont exclus du chronométrage partagé de cette
exécution.** Publier quand même un classement laisserait entendre qu'ils vérifient
les mêmes obligations. Le résultat n'est donc pas « clint gagne en vitesse »,
mais « équivalence du contrat partagé non démontrée ». Le pilote conserve leurs
sorties et le motif d'exclusion. Les cas propres à clint ne leur sont pas imputés.

Dans le rapport du 23 octobre, seules les mesures de clint sont donc conservées.
Les nouvelles mesures du 24 octobre incluent aussi commitlint et Commitizen,
qui passent les 15 cas partagés. Aucun objectif de gain, seuil bloquant de
performance ni extrapolation à un agent complet. Windows reste hors premier lot.

## Exécution réelle complémentaire — 24 octobre 2026

Les preuves sont conservées dans [runtime-2026-10-24/](benchmarks/runtime-2026-10-24/) :

- [macOS natif](benchmarks/runtime-2026-10-24/macos.json) : les cinq outils réellement
  exécutés ; clint 37/37, commitlint 21.2.4 et Commitizen 4.19.3 chacun 15/15 cas
  partagés. Les deux divergences cog/convco déjà décrites restent observées.
- [Linux ARM64 sous OrbStack](benchmarks/runtime-2026-10-24/linux.json) : clint
  37/37 ; commitlint et Commitizen chacun 15/15 cas partagés. Cog et convco ne
  sont pas installés dans cette image ; leur absence est explicitement enregistrée.
- [Validation Linux complète](benchmarks/runtime-2026-10-24/linux-validation.json) :
  format et Clippy réussis, 35 tests Rust, 2 doctests et 16 tests Python réussis.
  Les journaux correspondants sont conservés dans le même dossier.
- Admission au chronométrage confirmée pour clint, commitlint et Commitizen ;
  sept échantillons d'un processus par scénario, après deux préchauffages.
  Ce protocole descriptif est plus court que celui du 23 octobre ; ne pas fusionner
  les séries ni comparer directement une VM Linux à macOS natif.

OrbStack 2.2.5 fournit le moteur Linux ; Docker Desktop n'a pas été démarré.
La commande `docker --context orbstack` n'est ici que le client de son moteur
compatible. Images de base officielles fixées par digest, Node 26.8.1, Rust 1.99.1,
commitlint 21.2.4 et Commitizen 4.19.3. Python est 3.14.3 sur macOS et 3.11.2 dans
Debian Bookworm. Le conteneur de validation n'a ni réseau ni montage de secrets,
et se supprime à sa sortie. Seul le répertoire des résultats est monté en écriture.

Deux incidents ont été conservés plutôt que masqués :

1. Appel de version Commitizen incorrect, découvert avant toute mesure valide ;
   correction de l'adaptateur et ajout du seizième test Python.
2. Première validation Linux rouge : l'image minimale ne contenait pas l'exécutable
   `kill`, invoqué par un test existant. Le paquet système `procps` a été ajouté à
   l'image, sans modifier ni désactiver le test. Le [rapport initial](benchmarks/runtime-2026-10-24/linux-first-attempt.json)
   et son [journal](benchmarks/runtime-2026-10-24/linux-first-attempt-rust.log) restent disponibles.

Le produit testé est celui de `54e0ab15d694c684d6bb8c02249d507db9c64a36`, avec
uniquement les deux adaptations source consignées dans
[source-overrides.patch](benchmarks/runtime-2026-10-24/source-overrides.patch)
(harnais et test Python). Aucun changement moteur, dépendance Cargo, workflow ou
statut de PR n'a été nécessaire. La validation dans OrbStack n'est **pas** une
exécution de GitHub Actions ; la PR reste Draft.

### Rejouer la validation OrbStack

Depuis ce dépôt, après avoir démarré OrbStack, préparer un contexte isolé depuis
le commit et le patch enregistrés :

```sh
evidence="$PWD/docs/benchmarks/runtime-2026-10-24"
context=$(mktemp -d)
mkdir "$context/source" "$context/results"
git archive 54e0ab15d694c684d6bb8c02249d507db9c64a36 | tar -xf - -C "$context/source"
patch -d "$context/source" -p1 < "$evidence/source-overrides.patch"
for file in Dockerfile run-linux.py package.json package-lock.json requirements.txt; do
  cp "$evidence/$file" "$context/"
done
docker --context orbstack build -t clint-benchmark-replay "$context"
docker --context orbstack run --rm --network none --cap-drop ALL \
  --security-opt no-new-privileges --memory 4g --pids-limit 512 \
  --mount "type=bind,src=$context/results,dst=/results" clint-benchmark-replay
```

Le [relevé de provenance et d’empreintes](benchmarks/runtime-2026-10-24/provenance.json),
le [Dockerfile](benchmarks/runtime-2026-10-24/Dockerfile), les manifestes npm/Python,
les versions système et l'identifiant de l'image exécutée accompagnent les résultats.
Les paquets APT ne sont pas servis par un snapshot figé : un rebuild peut donc
recevoir des révisions système différentes. Les empreintes enregistrées décrivent
l'exécution réalisée, pas une promesse de reconstruction bit-à-bit.

## Reproduction

Construire clint avec le lockfile, puis lancer la conformité seule :

```sh
cargo build --release --locked
python3 scripts/benchmark_suite.py \
  --clint target/release/fluidise-clint \
  --output target/benchmark-correctness.json
python3 -m unittest discover -s tests -p 'test_benchmark*.py' -v
```

Ajouter les chemins vers les concurrents **déjà installés** et le chronométrage :

```sh
python3 scripts/benchmark_suite.py \
  --clint target/release/fluidise-clint \
  --cog /absolute/path/to/cog --convco /absolute/path/to/convco \
  --timing --samples 7 --iterations 10 \
  --output target/benchmark-comparison.json
```

Les options `--commitlint /absolute/path/to/commitlint` et
`--commitizen /absolute/path/to/cz` activent les deux autres adaptateurs.
Ne pas fournir un lanceur de téléchargement tel que `npx` ou `uvx`.
L'absence d'un outil est enregistrée, jamais assimilée à un échec de validation.

Les versions, configurations et empreintes doivent accompagner toute publication.
Pour les outils interprétés, l'empreinte du lanceur ne couvre **pas** tous les
modules ou le runtime : figer aussi leur environnement avant une comparaison
publiable. Les chemins de fixtures temporaires dans les commandes servent à
l'audit ; le pilote les recrée, ils ne sont pas destinés à être rejoués tels quels.

### Protocole de temps

- HOME, configuration Git et dépôt isolés ; pas de dépendance au dépôt appelant.
- Entrée stdin identique, sorties natives capturées. Le lancement du processus,
  la lecture, la configuration et la sortie sont inclus ; le jugement est hors temps.
- Admission d'un outil seulement si **tous** les cas partagés passent. Les cas de
  politique et erreurs propres à clint restent exclus de ce contrat partagé.
- Deux lots de préchauffage, puis sept lots par défaut ; dix processus séquentiels
  par lot, ordre des outils mélangé avec graine fixe. Échantillons en ms/processus,
  médiane et MAD conservés séparément pour chaque cas ; aucun score total pondéré.
- Première observation de chaque cas conservée séparément. Les caches OS ne sont
  pas contrôlés : ce n'est **pas** une mesure de démarrage à froid.
- Toute sortie devenue incorrecte pendant la mesure l'interrompt. Un timeout de
  10 s n'est jamais une validation rapide. `complete: false` signale un rapport
  interrompu ; `complete: true` signifie exécution terminée, pas réussite de tous
  les outils. Le code final dépend de la conformité clint ; consulter aussi les
  exclusions et résultats des concurrents.
- Aucune affirmation de significativité statistique depuis la seule MAD. Mesures
  sensibles au scheduler, au cache, au coût Python et au lancement des processus.

La CI macOS/Linux exécute le protocole et l'oracle clint sans chronométrage.
Elle ne télécharge pas de concurrents et ne bloque pas sur un seuil de vitesse.
L'ajout de ces étapes n'est pas une preuve de leur exécution sur GitHub Actions.

## Sort des benchmarks et pistes d'amélioration

| Élément | Décision et problème servi |
| --- | --- |
| Deux petits messages concurrentiels | Gardés dans le corpus élargi ; insuffisants seuls pour établir équivalence |
| Comparaison clint avant/après | Conservée pour non-régression exacte ; ne remplace pas l'oracle |
| Corps, ruptures, footers, Unicode, configuration | Ajoutés à l'oracle pour détecter verdicts incorrects et écarts de politique |
| Messages publics | Ajoutés avec provenance ; représentativité encore limitée |
| Limites et entrées denses des anciens harnais | Conservées comme robustesse clint, pas compétition de vitesse à travail différent |
| Scope non fermé | Tailles croissantes ajoutées comme contrôles de robustesse ; aucune conclusion ReDoS |
| RSS macOS de `benchmark.py` | Observation séparée conservée ; aucune garantie de plafond, pas de RSS Linux inventée |
| Classements historiques, CPU, CVE, « seul au monde » | Retirés faute de preuve suffisante et comparable |
| Boucle réelle d'agent / corrections | Différée : nécessite modèle, prompt, budget et protocole propres |

Pistes à explorer, **pas optimisations déjà justifiées** :

1. Résoudre l'écart de structure avec les configurations natives concurrentes,
   si possible, avant de réintroduire une comparaison de temps équivalente.
2. Enrichir le corpus avec incidents publics et erreurs de rédaction documentées,
   plutôt que multiplier des variations synthétiques sans problème explicite.
3. Séparer coût du lancement CLI et coût moteur avec les harnais existants avant
   de choisir le responsable d'une éventuelle optimisation.
4. Mesurer croissance en fonction de la taille sur une machine stabilisée avant
   toute conclusion de complexité ; RSS et temps restent des observations.
5. Évaluer plus tard l'utilité des diagnostics pour les corrections d'agents.
   Aucun incident utilisateur chiffré ni gain métier n'a été fourni dans cet audit.

Les [mesures CLI historiques](agent-throughput.md) et la
[boucle d'optimisation historique](benchmark-loop.md) restent consultables avec
leurs révisions et limites propres. Ne pas fusionner leurs chiffres avec ceux-ci.
