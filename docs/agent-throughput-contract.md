# Amélioration du débit de validation agentique CLI

Date : 2026-10-23.
Statut : contrat et exécution locale approuvés le 2026-10-23 (« vasy »).

## Objectif confirmé

Réduire le temps réellement consommé par les agents sur des usages représentatifs. Le classement concurrentiel est une comparaison, pas un critère suffisant de réussite.

## Périmètre confirmé

- Mesurer des boucles de 100 invocations CLI indépendantes dans un dépôt Git.
- Couvrir la configuration automatique et explicite, ainsi que des messages valides et invalides.
- Conserver les commandes, le JSON, les diagnostics, le déterminisme, les bornes mémoire et l'absence d'effets externes du linter.
- Comparer clint avant/après et refaire les mesures de Cocogitto et convco avec des versions figées et un protocole commun.
- Distinguer la validation commune du coût du contrat JSON propre à clint ; ne pas présenter des services différents comme équivalents.
- Exclure un mode batch ou un daemon et soumettre séparément tout changement de contrat public.

## État des preuves

Les résultats concurrents de [benchmarks.md](benchmarks.md) sont historiques et non reproduits. Ils ne démontrent pas le classement actuel.

Le harnais local décrit dans [benchmark-loop.md](benchmark-loop.md) compare uniquement clint avant/après, hors Git. Il ne couvre donc pas à lui seul le scénario retenu ici.

La découverte de configuration peut lancer Git à chaque invocation dans un dépôt. Son coût est une hypothèse à mesurer, pas une cause démontrée.

## Critères confirmés

- Viser une réduction d'au moins 20 % du temps médian d'une boucle de 100 validations courtes, avec configuration automatique dans Git. Il s'agit d'un objectif, pas d'un gain garanti.
- Ne conserver aucune différence fonctionnelle. Toute régression supérieure à 10 % sur les autres scénarios impose une investigation et une nouvelle mesure.
- Effectuer un diagnostic mesuré, puis au maximum deux cycles d'optimisation et de mesure.
- Ne pas ajouter de dépendance ni modifier l'interface publique.
- Si le gain reste insuffisant, présenter les résultats et demander une décision avant d'élargir le périmètre.

## Moyens disponibles

Cocogitto 7.0.0 et convco 0.7.2 sont déjà installés. Rust et Cargo 1.99.1, Python et hyperfine sont disponibles. Aucune installation supplémentaire n'est prévue. Le protocole doit conserver une baseline release immuable, utiliser des dépôts Git temporaires et enregistrer les mesures brutes ainsi que les versions et paramètres nécessaires à leur reproduction.

## Autorisations

Le contrat, les builds locaux, les fixtures temporaires, l'adaptation du harnais, les mesures et les optimisations bornées sont approuvés. Commit, push et pull request nécessitent des accords séparés.

## Documentation des décisions

Le vocabulaire est défini dans [CONTEXT.md](../CONTEXT.md). Ce contrat n'est pas un ADR : aucune décision architecturale difficile à inverser n'est prise à ce stade.

## Révision approuvée : sélection explicite de la politique

Le 2026-10-23, après clarification du principe agent-first, l'utilisateur approuve
la proposition par « ok go » : exactement un choix parmi `--config <path>` et
`--defaults` devient obligatoire pour toute validation. `--defaults` sélectionne
les règles intégrées sans fichier ; `--config` charge uniquement le fichier
indiqué. La découverte automatique et les appels Git du linter sont supprimés.
Cette autorisation remplace les exclusions précédentes relatives au changement
d'interface et à la conservation de la découverte automatique ; elle n'autorise
ni nouvelle dépendance, ni installation de hooks, ni livraison Git.

Critères :

1. Choix absent, contradictoire, répété ou valeur attachée à `--defaults` :
   erreur d'arguments JSON, code 2, sans lecture d'entrée ni attente sur stdin.
2. `--help` et `--version` restent utilisables seuls ; erreurs d'arguments JSON
   même avec `--human` ; mêmes règles de choix de la source du message.
3. Même moteur, mêmes règles intégrées et mêmes rapports à politique équivalente ;
   configuration explicite invalide ou inaccessible reste une erreur prioritaire.
4. Aucune recherche de configuration ni exécution Git, même dans un dépôt cassé,
   avec des variables Git hostiles ou sans Git dans PATH.
5. Adapter les appels maintenus (tests, exemples, hook généré, smoke de
   distribution et validation de titre CI) sans installer ni exécuter les hooks
   ou les workflows distants. Préserver les rapports historiques et distinguer
   la rupture de contrat d'une optimisation transparente.
6. Vérifier format, Clippy, tests, harnais et comparer les binaires figés sur le
   protocole existant adapté à la sélection explicite. Maximum deux cycles.

La décision durable est enregistrée dans ADR 0002. Revue indépendante : à proposer
après les validations, aucune délégation de revue ni livraison autorisée.

## Preuves de la révision et état de livraison

- Critères 1–2 : tests de parsing portable, erreurs avec stdin maintenu ouvert,
  `--human`, help/version et rejet des choix absents/contradictoires/répétés.
- Critère 3 : moteur inchangé, tests des fichiers et erreurs prioritaires,
  52 comparaisons byte-à-byte sur le corpus étendu, quatre contextes.
- Critère 4 : code de découverte supprimé et test CLI sans Git dans PATH,
  métadonnées Git cassées et variable `GIT_DIR` hostile.
- Critère 5 : appels CI, exemples, hook généré, smoke et recette locale de test
  migrés ; aucune installation ni exécution distante. Recette externe inchangée.
- Critère 6 : format, Clippy strict, build release, 32 tests Rust, 2 doctests,
  20 tests Python ; cycle 1 mesuré, réduction de 64,7 à 66,3 % sur les anciens
  chemins de découverte. Aucun deuxième cycle nécessaire pour cette cible.

Détails, limites et résultats bruts : [rapport](agent-throughput.md).
Revue indépendante : proposée, décision utilisateur en attente ; recommandée
pour la rupture CLI et ses consommateurs, avec coût d'un passage supplémentaire.
Le comportement est implémenté et vérifié localement ; livraison finale en attente
du choix de revue. Aucun commit, push, PR ou déploiement autorisé ou effectué.

## Autorisation de revue indépendante

L'utilisateur a choisi « 1 » pour lancer la revue, puis « 1 » pour sélectionner
`gpt-6.1-sol` parmi les options proposées (`gpt-6.1-sol`, recommandé pour la revue
ciblée, et `gpt-6-astra` ; coût et latence comparés non vérifiés).
Lot approuvé : un reviewer en contexte neuf, lecture seule, périmètre CLI, JSON,
intégrations et tests, avec la revue initiale et jusqu'à deux relectures après
corrections éventuelles. Modèle demandé : `gpt-6.1-sol` ; identité réelle non
vérifiée tant que le runtime ne la rapporte pas. Aucune autorisation de livraison.
