# ADR 0002 : sélection explicite de la politique CLI

Statut : accepté le 2026-10-23 (« ok go »).

Toute validation exige exactement `--config <path>` ou `--defaults`. Le linter
valide le message avec la politique choisie ; l'appelant possède le choix du
fichier. La découverte Git est supprimée, sans embarquer une bibliothèque Git.
La mesure locale a montré un coût important du chemin de découverte ; le principe
agent-first privilégie aussi les entrées explicites et le coût du workflow entier.

Imposer uniquement un fichier aurait forcé les consommateurs des règles intégrées
à créer et maintenir un fichier inutile. `--defaults` conserve cette autonomie
sans découverte implicite. En contrepartie, les anciens appels sans sélection de
politique deviennent des erreurs et les intégrations doivent migrer. JSON, règles,
bornes et validation du fichier explicite restent inchangés ; les erreurs de
découverte Git disparaissent délibérément avec cette responsabilité. Cette décision
remplace la découverte et l'absence implicite de fichier du contrat CLI antérieur,
pas l'API Rust `prepare_policy(None)`.
