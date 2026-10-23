# Débit de validation agentique CLI — 23 octobre 2026

**État actuel :** la révision approuvée impose `--config` ou `--defaults`,
sans découverte Git. La [mesure avant/après](#révision-approuvée--politique-explicite-obligatoire)
montre 64,7 à 66,3 % de réduction sur les anciens scénarios de découverte.
Les premières sections conservent le diagnostic initial, antérieur à cette décision.

## Périmètre et statut

Voir le [contrat approuvé](agent-throughput-contract.md) et le [glossaire](../CONTEXT.md).
Cette mesure concerne des processus CLI indépendants, pas des appels répétés du moteur Rust.

La référence est le code produit de `c742e55f7b2d935fd31ff0fb5efd4166d0f1a178`,
compilé avec Rust/Cargo 1.99.1, `Cargo.lock` et le profil release existant.
Lors de cette mesure initiale, aucun code produit, dépendance ou interface publique n'avait été modifié. La révision CLI ultérieure est décrite ci-dessous.
Les emplacements « baseline » et « candidate » pointent volontairement vers
le même binaire : c'est une mesure de référence avec contrôle de variabilité,
**pas une preuve de gain avant/après**.

## Protocole

Le harnais [benchmark-agent.py](../scripts/benchmark-agent.py) réutilise le corpus,
les empreintes et les résumés du harnais existant. Il réalise deux préchauffages
puis sept mesures de 100 processus par scénario. L'ordre des outils est mélangé
avec une graine fixe ; les mesures brutes, médianes et écarts absolus médians
(MAD) sont conservés. Les deux emplacements identiques de clint contrôlent
l'amplitude du bruit, sans en faire une preuve statistique de significativité.

- Même message sur stdin ; stdout et stderr capturés, jamais supprimés.
- Coûts Python, lancement du processus, configuration, lecture et sortie inclus.
- Vérification des résultats après la boucle chronométrée.
- Dépôts Git et HOME temporaires ; aucune configuration personnelle.
- Identité Git fictive nécessaire à `cog verify` 7.0.0. Sans elle, cette version
  échoue avec un panic ; cet échec n'est pas chronométré comme une validation.
- Configurations natives équivalentes aux défauts : `[commit]\nversion=1` pour
  clint, TOML vide pour cog, `{}` pour convco. Leur chargement reste propre à
  chaque outil ; l'équivalence s'arrête au petit corpus commun accepté/refusé.
- Chaque sortie clint doit rester exactement identique entre les emplacements,
  codes et stderr compris. Les concurrents doivent retourner le code attendu,
  mais leur texte n'est pas tenu pour déterministe (`cog` affiche une date).
- Hors Git, seul clint est comparé à lui-même : convco 0.7.2 exige un dépôt
  même avec `check --from-stdin`. Aucun classement artificiel de ce cas.
- Les treize cas du corpus étendu (limites, UTF-8 invalide, grandes entrées,
  exemptions) sont aussi comparés exactement pour clint dans les quatre modes.
- Une mesure JSON/`--human` séparée estime un écart entre modes de sortie.
  Elle ne mesure pas isolément le CPU de sérialisation JSON et ne remplace
  jamais le JSON du résultat principal.

Environnement local : macOS 26.7.1 ARM64, Python 3.14.3, Git 2.53.0,
Cocogitto 7.0.0, convco 0.7.2. Exécution finale hors sandbox ; les essais
préparatoires incomplets ne sont pas les résultats de confirmation.

## Reproduction

Pour reproduire la mesure initiale, reconstruire les sources produit historiques,
pas la CLI modifiée. Depuis la racine du dépôt, dans un dossier de sortie neuf :

```sh
snapshot=$(mktemp -d)
git archive c742e55f7b2d935fd31ff0fb5efd4166d0f1a178 | tar -x -C "$snapshot"
cargo build --manifest-path "$snapshot/Cargo.toml" --release --locked --offline
python3 -B scripts/benchmark-agent.py \
  "$snapshot/target/release/fluidise-clint" \
  "$snapshot/target/release/fluidise-clint" \
  --legacy-baseline --legacy-candidate \
  --cog /opt/homebrew/bin/cog \
  --convco /opt/homebrew/bin/convco \
  --samples 7 \
  --output target/agent-throughput/reproduced-initial.json
```

Le dossier temporaire `snapshot` contient seulement cet export et sa compilation ;
le supprimer après inspection et conservation des preuves voulues.

Adapter les chemins des concurrents à des exécutables déjà installés aux
versions indiquées ; le harnais n'installe rien. Pour une expérience future,
construire et figer séparément le candidat puis le passer en second argument.
Les empreintes des binaires sont contrôlées au début et à la fin de la mesure.
Un `sources.json` voisin du binaire est inclus dans le rapport s'il existe ;
la mesure présente y conserve HEAD et les SHA-256 de Cargo.toml, Cargo.lock
et des sources Rust. Les fixtures sont supprimées à la fin ; les commandes
brutes contiennent donc des chemins temporaires à recréer via le harnais.

## Résultats de confirmation

Médianes en millisecondes par **100 validations indépendantes**. Les deux
colonnes clint utilisent exactement le même binaire ; leur écart est du bruit,
pas une optimisation. Aucun score agrégé ne masque les scénarios.

| Scénario | clint référence | clint contrôle identique | Cocogitto | convco |
| --- | ---: | ---: | ---: | ---: |
| Git, configuration absente, valide | 1078,13 | 1064,31 | 1003,68 | 677,19 |
| Git, configuration absente, invalide | 1153,22 | 1194,00 | 1139,45 | 801,03 |
| Git, configuration présente, valide | 1105,33 | 1134,94 | 1017,93 | 746,86 |
| Git, configuration présente, invalide | 1160,95 | 1093,17 | 1024,77 | 722,52 |
| Git, configuration explicite, valide | 369,63 | 402,77 | 984,38 | 703,94 |
| Git, configuration explicite, invalide | 354,51 | 372,50 | 984,99 | 651,99 |
| Hors Git, valide | 365,84 | 378,34 | non comparé | non comparé |
| Hors Git, invalide | 377,25 | 392,05 | non comparé | non comparé |

**Configuration connue :** à politique et sorties JSON identiques sur les deux
petits messages, passer explicitement le fichier existant réduit ici le temps
médian d’environ **65 %** face à sa découverte automatique. Pour le message
valide, le premier contrôle passe de 1 105,33 à 369,63 ms pour 100 appels
(90,5 à 270,5 validations/s) ; le second, de 1 134,94 à 402,77 ms.
Ce résultat réemploie une interface existante, pas une modification du moteur.

**JSON :** dans la mesure séparée, JSON/texte valent respectivement
1 007,51/1 017,61 ms au premier contrôle et 984,56/959,49 ms au second.
Le signe de l’écart change ; aucun gain de 20 % lié au mode texte n’est démontré.
Retirer le JSON ne résoudrait pas la cible et violerait le contrat approuvé.

**Objectif automatique : non atteint.** À ce stade initial, aucun patch produit spéculatif n’était
retenu. La révision de contrat approuvée ensuite est décrite ci-dessous. La différence entre configuration automatique et explicite soutient
l’attribution au chemin de découverte Git, sans constituer un profil CPU :
elle inclut aussi les accès aux marqueurs et au chemin de configuration.
Les sondes de lancement de Git n’ont pas montré de gain stable en déplaçant
le changement de répertoire vers `git -C` ; cette piste n’est pas intégrée.

Les [résultats bruts](benchmarks/agent-throughput-2026-10-23.json) contiennent
les échantillons, MAD, commandes, sorties, versions et empreintes. Les 52 cas
de comparaison exacte du corpus étendu passent. Cela vérifie le protocole sur
un binaire identique, pas une nouvelle implémentation ni des garanties RSS.

Limites : une seule machine macOS ARM64, caches chauds, deux petits messages
communs aux concurrents, règles et contrats de sortie différents. Les chiffres
historiques ne sont pas une baseline comparable et aucun gain produit entre
leurs dates et cette mesure ne peut en être déduit.

## Vérifications

- `cargo test --all-targets --locked --offline` : 30 tests passent hors sandbox.
- `cargo test --doc --locked --offline` : 2 doctests passent.
- `cargo fmt --all -- --check` : passe.
- `cargo clippy --all-targets --locked --offline -- -D warnings` : passe.
- `python3 -m unittest discover -s tests -p test_benchmark_agent.py -v` :
  4 tests du protocole passent (isolation, parcours complet, refus des sorties
  clint modifiées et des erreurs opérationnelles concurrentes).

Deux tests Unix échouaient dans la sandbox sur `Operation not permitted` lors
de la préparation des fixtures ; ils passent hors sandbox sans correction du
code produit. Les essais préparatoires ont également corrigé le harnais pour
l’identité Git exigée par cog et le refus de convco hors dépôt. Aucun de ces
échecs n’est comptabilisé comme une validation plus rapide.

## Décision de périmètre

La découverte automatique dans un dépôt ordinaire lance déjà un seul processus
Git. Le marqueur `.git` ne garantit ni la racine effective (`core.worktree`,
worktrees liés, indirection), ni la validité ou la confiance du dépôt. Le prendre
pour racine ou retourner les défauts sans Git pourrait masquer les erreurs
actuelles. Un cache persistant ajouterait des effets externes et une politique
d'invalidation ; une bibliothèque Git embarquée ajouterait une dépendance et
exigerait une étude d'équivalence. Ces changements ne sont pas autorisés ici.

La voie existante `--config` évite Git lorsque l'appelant connaît déjà le fichier
voulu. Elle ne change pas le binaire et ne satisfait pas l'objectif distinct
d'accélération de la découverte automatique. Ne pas déplacer le coût vers un
nouveau `git rev-parse` par message dans l'appelant et annoncer un gain global.

Aucune micro-optimisation du moteur ou de JSON n'est justifiée sans coût mesuré
suffisant. Deux cycles constituent un maximum, pas une obligation de produire
des patches spéculatifs. Toute modification de ces frontières nécessite une
nouvelle décision ; aucune ADR architecturale n'est créée pour ce simple constat.

## Révision approuvée : politique explicite obligatoire

Après la mesure initiale et la clarification agent-first, l'utilisateur a approuvé
la rupture décrite dans [ADR 0002](adr/0002-explicit-policy-selection.md).
Toute validation exige maintenant exactement `--config <path>` ou `--defaults` ;
la découverte Git et ses erreurs sont délibérément supprimées. L'API du moteur,
les règles, le JSON et la lecture bornée des fichiers ne changent pas.

Il s'agit donc d'un **gain de workflow avec changement de contrat approuvé**,
pas d'une optimisation transparente de la découverte automatique. L'appelant
assume la sélection de la politique ; il ne doit pas remplacer cette sélection
par un nouveau processus Git pour chaque message.

### Mesure avant/après, cycle 1

Même protocole local à sept échantillons, deux préchauffages, 100 processus et
versions concurrentes figées. Le harnais est lancé avec `--legacy-baseline` :

- `auto_absent` : ancien appel implicite dans Git contre nouveau `--defaults` ;
- `auto_present` : ancienne découverte contre nouveau `--config` du même fichier ;
- `explicit` : `--config` du même fichier dans les deux exécutables ;
- `outside` : anciens défauts implicites contre nouveau `--defaults`, hors Git.

Les noms des scénarios décrivent le contexte historique de la référence, **pas
un mode automatique restant dans le candidat**. Codes, stdout et stderr doivent
être identiques pour chaque message à politique équivalente. Les deux petits
messages ne démontrent pas l'équivalence de toutes les règles concurrentes.

Médianes en millisecondes par 100 validations :

| Scénario | Ancien clint | Nouveau clint | Cocogitto | convco | Réduction du temps clint |
| --- | ---: | ---: | ---: | ---: | ---: |
| Git, fichier absent, valide | 1202,41 | 420,17 | 1089,31 | 753,15 | 65,1 % |
| Git, fichier absent, invalide | 1159,81 | 409,67 | 1081,04 | 732,47 | 64,7 % |
| Git, fichier présent, valide | 1189,19 | 413,55 | 1052,29 | 745,74 | 65,2 % |
| Git, fichier présent, invalide | 1122,97 | 377,94 | 1062,81 | 731,28 | 66,3 % |
| Git, fichier explicite, valide | 345,84 | 332,44 | 978,51 | 674,53 | 3,9 % |
| Git, fichier explicite, invalide | 398,12 | 382,80 | 1043,18 | 733,90 | 3,8 % |
| Hors Git, valide | 429,08 | 379,72 | non comparé | non comparé | 11,5 % |
| Hors Git, invalide | 424,56 | 374,07 | non comparé | non comparé | 11,9 % |

Sur les anciens scénarios de découverte dans Git, le temps diminue de **64,7 à
66,3 %**. Pour le message valide sans fichier, le débit passe de **83,2 à
238,0 validations/s** ; convco atteint 132,8/s et Cocogitto 91,8/s sur ce cas.
Le nouveau clint dépasse les deux sur les six scénarios concurrents mesurés,
sans que cela établisse une supériorité générale sur tous les messages ou systèmes.
Les écarts plus petits sur les chemins déjà explicites ne sont pas attribués
à une optimisation spécifique. Aucun scénario mesuré ne régresse de plus de 10 %.

Les [résultats bruts de la révision](benchmarks/agent-throughput-explicit-policy-2026-10-23.json)
contiennent les 52 comparaisons exactes du corpus étendu. Le candidat a été
reconstruit après l'ajout d'un test unitaire de parsing : le binaire release final
est byte-identique au candidat mesuré (vérification consignée dans le rapport).
Aucune nouvelle garantie RSS ni preuve multiplateforme d'exécution n'est revendiquée.

### Rejouer la révision

Après avoir reconstruit la baseline historique comme ci-dessus et compilé la
CLI actuelle, utiliser des exécutables figés et un nouveau chemin de résultat :

```sh
cargo build --release --locked --offline
python3 -B scripts/benchmark-agent.py \
  "$snapshot/target/release/fluidise-clint" \
  target/release/fluidise-clint \
  --legacy-baseline \
  --cog /opt/homebrew/bin/cog \
  --convco /opt/homebrew/bin/convco \
  --samples 7 \
  --output target/agent-throughput/reproduced-explicit-policy.json
```

### Vérifications de la révision

- 32 tests Rust all-targets et 2 doctests passent localement ; les fixtures Unix
  sont exécutées hors sandbox. Le test ajouté a d'abord échoué parce que l'ancien
  parseur refusait `--defaults`, puis passe avec la sélection explicite.
- Format, Clippy strict et build release verrouillé passent.
- Le harnais historique adapté passe aussi les treize cas CLI/moteur avec trois
  échantillons, `--legacy-baseline` et des moteurs identiques (bibliothèque inchangée).
  C'est un smoke de compatibilité du harnais, pas une preuve de gain moteur ; ses
  observations RSS ne constituent pas un plafond mémoire.
- 20 tests Python passent : cinq pour le protocole et quinze pour la distribution.
- Le smoke de distribution passe avec le binaire release final ; le hook généré
  est vérifié par `bash -n`, mais n'est ni installé ni exécuté.
- L'adaptation de la recette Homebrew de test est vérifiée par test unitaire ;
  aucun tap externe, installation Homebrew ou workflow distant n'est modifié/exécuté.
- Les commandes du titre de PR en CI, les exemples et les instructions de migration
  utilisent le nouveau choix explicite. Aucune exécution CI distante n'est prouvée.
- Documentation lexopt 0.3.2 consultée via Context7 : `Parser::optional_value`
  ne consomme que les valeurs attachées ; `--defaults=true` et `--defaults=` sont
  donc refusés. Source : https://docs.rs/lexopt/0.3.2/lexopt/struct.Parser.html#method.optional_value.

Une revue indépendante est proposée avant livraison ; elle n'a pas encore été
acceptée ni exécutée. Aucun commit, push, PR, installation ou déploiement.
