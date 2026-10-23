# Vocabulaire de fluidise-clint

`fluidise-clint` valide des messages de commit pour des agents et des hooks non interactifs.

## Langage

**Validation agentique CLI** :
Validation d'un message de commit par une invocation indépendante de l'outil, avec son résultat exploitable par un agent. Pour clint, ce résultat comprend le rapport JSON et le code de sortie.
_Éviter_ : commit créé, validation en mémoire, traitement par lot.

**Débit de validation agentique CLI** :
Nombre de validations agentiques CLI terminées par seconde dans un scénario défini. Ce débit ne désigne ni celui du moteur seul ni celui de validations regroupées dans un processus persistant.
_Éviter_ : débit moteur, vitesse des agents, commits créés par seconde.

**Sélection de politique** :
Choix explicite des règles applicables à une validation : règles intégrées ou configuration fournie par l'appelant.
_Éviter_ : découverte implicite, deux versions de l'outil.
