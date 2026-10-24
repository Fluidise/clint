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
| commitlint | stdin, configuration CJS explicite, `--strict` ; 3 signifie rejet | Adaptateur documenté/testé avec sorties simulées, pas d'exécutable local fourni |
| commitizen | stdin, `--config … check`, `cz_conventional_commits` ; 14/32 signifient rejet | Adaptateur documenté/testé avec sorties simulées, pas d'exécutable local fourni |

La politique commitlint du pilote est volontairement limitée : type/sujet requis,
énumération des types et ligne vide avant le corps. Des options de parsing
explicites prennent en charge scope et `!` sans ajouter de paquet de preset. Elle n'est pas présentée comme
le preset `config-conventional` complet. Les sorties natives ne sont pas tenues
pour du JSON ni pour équivalentes aux diagnostics structurés de clint. Les codes
opérationnels connus et timeouts sont séparés des rejets. Un code natif ambigu
(comme 1 pour cog/convco) ne suffit pas à distinguer toutes les erreurs internes
d'un rejet syntaxique ; les sorties restent conservées pour examen. Les contrôles
positifs réduisent ce risque sans constituer une preuve exhaustive.

Les deux adaptateurs non exécutés restent **non validés en intégration** ; leurs
versions et résultats ne sont pas inventés. Les cinq outils sont réévalués dans
le périmètre, mais seulement trois disposent ici de preuves runtime.

Sources d'interface consultées via Context7 le 23 octobre 2026 :
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

Les mesures restantes de clint sont descriptives : aucun objectif de gain, aucun
seuil bloquant de performance et aucune extrapolation à un agent complet.
L'environnement effectivement mesuré est macOS ARM64. Linux reste reproductible
par le même pilote, mais n'a pas été exécuté ici. Windows est hors premier lot.

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
