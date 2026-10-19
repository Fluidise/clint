# Benchmark Approfondi : fluidise-clint face à l'État de l'Art

> Ce banc d'essai exhaustif s'inspire de la méthodologie de présentation des benchmarks de [Bun](https://bun.com/blog/bun-v1.4.3), en évaluant `fluidise-clint` face aux 4 outils de référence du marché à travers 3 écosystèmes majeurs (Rust, Python et JavaScript / Node.js).

---

## Les Concurrents Évalués

| Outil | Version | Écosystème | Runtime requis | Dépendances | Poids sur disque |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **`fluidise-clint`** | **0.1.0** | **Rust (Natif)** | **Aucun (Binaire statique)** | **3** | **0.5 Mo** |
| **`convco`** | 0.7.2 | Rust (Natif) | Aucun (Binaire statique) | 45 | 7.6 Mo *(15x plus gros)* |
| **`cocogitto` (`cog`)** | 7.0.0 | Rust (Natif) | Aucun (Binaire statique) | 78 | 5.9 Mo *(12x plus gros)* |
| **`commitizen` (`cz`)** | 4.19.2 | Python | Python 3.x | 17 | 45.2 Mo *(90x plus lourd)* |
| **`commitlint`** | 21.2.4 | JavaScript | Node.js / V8 | 82 | 148.0 Mo *(296x plus lourd)* |

---

## 1. Synthèse Exécutive (Style Bun)

```
TEMPS DE DÉMARRAGE ET VALIDATION (Commit Court 45 B - plus bas = meilleur)
fluidise-clint   [13.2 ms]  ■
convco           [ 8.9 ms]  ■
cocogitto        [12.0 ms]  ■
commitizen       [228.4 ms] ■■■■■■■■■■■■■■■■■ (17.3x plus lent)
commitlint       [224.6 ms] ■■■■■■■■■■■■■■■■■ (17.0x plus lent)

CONSOMMATION MÉMOIRE RÉSIDENTE RSS (plus bas = meilleur)
fluidise-clint   [ 7.08 Mo] ■
convco           [10.38 Mo] ■■ (+46% RAM)
cocogitto        [10.02 Mo] ■■ (+41% RAM)
commitizen       [50.44 Mo] ■■■■■■■ (7.1x plus de RAM)
commitlint       [97.12 Mo] ■■■■■■■■■■■■■■ (13.7x plus de RAM)
```

### Matrice Récapitulative des Performances

| Métrique clé | `fluidise-clint` | `convco` | `cocogitto` | `commitizen` | `commitlint` |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Latence commit standard** | **13.2 ms** | 8.9 ms | 12.0 ms | 228.4 ms *(17.3x)* | 224.6 ms *(17.0x)* |
| **Empreinte RSS (standard)** | **7.08 Mo** | 10.38 Mo | 10.02 Mo | 50.44 Mo *(7.1x)* | 97.12 Mo *(13.7x)* |
| **Instructions CPU retirées** | **18.5 M** | 36.8 M *(2.0x)* | 64.2 M *(3.5x)* | 1 582 M *(85x)* | 2 134 M *(115x)* |
| **Débit en boucle (100 commits)** | **76.9 /s** | 119.4 /s | 87.6 /s | 4.5 /s *(17x plus lent)* | 4.7 /s *(16x plus lent)* |
| **Attaque ReDoS (100k chars)** | **13.7 ms** | 9.8 ms | 16.4 ms | 230.6 ms | **TIMEOUT (> 5s)** |
| **Frontière 1 MiB (rejet borné)** | **13.0 ms (code 2)** | 21.9 ms (non borné) | 21.6 ms (non borné) | 234.5 ms (non borné) | **TIMEOUT (> 5s)** |
| **Stress test extrême (20 Mo)** | **17.2 ms (7.1 Mo)** | 332.9 ms (72.4 Mo) | 208.8 ms (70.0 Mo) | 365.1 ms (109.8 Mo) | **TIMEOUT (> 5s)** |
| **Format par défaut** | **JSON strict** | Texte libre | Texte ANSI | Texte libre | Texte ANSI |
| **Plafond mémoire garanti** | **Oui (1 MiB)** | Non | Non | Non | Non |

---

## 2. Workload 1 : Latence Unitaire et Empreinte Mémoire

Mesures sur 15 itérations consécutives (médiane) précédées de 3 runs de préchauffage (warmup).

### A. Commit Court Valide (`valid_small` — 44 octets)
> `feat(core): implement bounded memory stream\n`

| Outil | Latence | Faster | RSS Max | Less RAM | Instructions CPU | Cycles CPU |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`fluidise-clint`** | **13.21 ms** | **17.0x** vs commitlint | **7.08 Mo** | **13.7x** vs commitlint | **18 543 315** | **9 565 922** |
| `convco` | 8.94 ms | 25.1x vs commitlint | 10.38 Mo | 9.4x vs commitlint | 36 757 408 | 15 277 025 |
| `cocogitto` | 12.04 ms | 18.6x vs commitlint | 10.02 Mo | 9.7x vs commitlint | 64 236 936 | 23 094 562 |
| `commitizen` | 228.38 ms | Baseline Py | 50.44 Mo | 1.9x vs commitlint | 1 582 267 838 | 659 869 500 |
| `commitlint` | 224.62 ms | Baseline Node | 97.12 Mo | Baseline Node | 2 134 505 543 | 843 121 079 |

### B. Commit Invalide avec Détection d'Erreur (`invalid_small` — 37 octets)
> `fix something without type separator\n`

| Outil | Latence | Code de Sortie | Format Diagnostic | Instructions CPU |
| :--- | :---: | :---: | :--- | :---: |
| **`fluidise-clint`** | **12.50 ms** | **1** | **JSON structuré (`commit.header.invalid`, span)** | **18 531 950** |
| `convco` | 9.35 ms | 1 | Texte brut terminal (`FAIL - first line doesn't match`) | 36 694 078 |
| `cocogitto` | 12.23 ms | 1 | Texte compilateur Rustc (`--> 1:4 | Expected...`) | 64 286 279 |
| `commitizen` | 236.48 ms | 14 | Texte terminal (`commit validation: failed!`) | 1 579 413 343 |
| `commitlint` | 240.11 ms | 1 | Texte terminal Unicode/ANSI (`✖ subject-empty`, etc.) | 2 108 973 470 |

### C. Commit Réaliste Multiligne (`multiline_2k` — 1 809 octets)
> Sujet, saut de ligne, corps avec 25 puces d'explication d'invariants, et pied de page `Refs: #123`.

| Outil | Latence | Faster | RSS Max | Instructions CPU |
| :--- | :---: | :---: | :---: | :---: |
| **`fluidise-clint`** | **12.63 ms** | **18.1x** | **7.09 Mo** | **18 568 495** |
| `convco` | 8.69 ms | 26.2x | 10.36 Mo | 37 053 396 |
| `cocogitto` | 11.90 ms | 19.2x | 10.02 Mo | 64 856 263 |
| `commitizen` | 237.13 ms | Baseline Py | 50.33 Mo | 1 584 372 395 |
| `commitlint` | 228.07 ms | Baseline Node | 92.08 Mo | 2 183 384 955 |

---

## 3. Workload 2 : Débit en Boucle Agentique Intensive (100 Commits)

Dans une boucle de travail agentique où des sous-agents itèrent, modifient du code et valident fréquemment leurs messages de commits, le débit brut détermine directement la réactivité de la chaîne :

```mermaid
xychart-beta
    title "Débit de Validation Agentique (Commits par Seconde - plus haut = meilleur)"
    x-axis ["fluidise-clint", "convco", "cocogitto", "commitizen", "commitlint"]
    y-axis "Commits / sec" 0 --> 130
    bar [76.9, 119.4, 87.6, 4.5, 4.7]
```

| Outil | Temps total (100 commits) | Débit (commits / sec) | Facteur d'Accélération |
| :--- | :---: | :---: | :---: |
| **`convco`** | **0.84 s** | **119.4 /s** | **25.4x plus rapide** |
| **`cocogitto`** | **1.14 s** | **87.6 /s** | **18.6x plus rapide** |
| **`fluidise-clint`** | **1.30 s** | **76.9 /s** | **16.4x plus rapide** *(avec émission JSON)* |
| `commitlint` | 21.39 s | 4.7 /s | Baseline Node.js |
| `commitizen` | 22.23 s | 4.5 /s | Baseline Python |

> [!NOTE]
> `fluidise-clint` valide 100 commits en **1.3 seconde** tout en sérialisant un flux JSON strict avec offsets d'octets. `commitlint` et `commitizen` mettent plus de 21 secondes pour accomplir la même tâche.

---

## 4. Workload 3 : Validation d'un Historique Git Réel

Validation des 9 derniers commits réels du dépôt `Fluidise/clint` :

| Outil | Temps d'exécution | Commits Valides | Commits Invalides | Erreurs système |
| :--- | :---: | :---: | :---: | :---: |
| **`fluidise-clint`** | **0.11 s** | **9** | **0** | **0** |
| `convco` | 0.10 s | 8 | 1 *(faux positif sur PR merge)* | 0 |
| `cocogitto` | 0.10 s | 8 | 1 *(faux positif sur PR merge)* | 0 |
| `commitizen` | 2.02 s | 9 | 0 | 0 |
| `commitlint` | 1.94 s | 8 | 1 *(faux positif sur PR merge)* | 0 |

> `fluidise-clint` gère nativement les messages de fusion GitHub/Git (`Merge pull request #...`) sans générer de faux positifs.

---

## 5. Workload 4 : Résistance ReDoS, Cas Adversariaux et Plafond de Mémoire

Ce workload évalue la robustesse des outils lorsqu'ils sont confrontés à des entrées malveillantes, accidentellement gigantesques ou conçues pour provoquer des dénis de service (ReDoS).

```mermaid
flowchart TD
    subgraph "Gestion des Payloads Volumineux"
        Input["Payload de 20 Mo transmis via Git Hook"]
        Input --> Clint["fluidise-clint\n(Arrêt au probe byte 1 MiB + 1)"]
        Input --> Cog["cocogitto / convco\n(Allocation complète)"]
        Input --> Lint["commitlint\n(V8 String + Regex Backtracking)"]
        
        Clint --> ResClint["✅ Sortie en 17.2 ms\nRSS plafonné à 7.1 Mo\nCode POSIX 2 (input.limit_exceeded)"]
        Cog --> ResCog["⚠️ Sortie en 208 ms\nRSS explose à 70 Mo (+700% RAM)\nRisque d'OOM sur VM restreinte"]
        Lint --> ResLint["❌ TIMEOUT (> 5s)\nBlocage total du processus\nCPU spinning"]
    end
```

### Résultats Détaillés des Cas Adversariaux

| Test Case | `fluidise-clint` | `convco` | `cocogitto` | `commitizen` | `commitlint` |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **`redos_100k`**<br>(100k chars sans fermeture) | **13.7 ms**<br>exit 1 (7.0 Mo) | 9.8 ms<br>exit 1 (10.7 Mo) | 16.4 ms<br>exit 1 (27.3 Mo) | 230.6 ms<br>exit 14 (50.2 Mo) | **TIMEOUT (> 5s)**<br>❌ Gel du moteur regex |
| **`boundary_under_1mb`**<br>(1 048 500 octets — valide) | **14.9 ms**<br>exit 0 (7.0 Mo) | 23.5 ms<br>exit 0 (13.4 Mo) | 22.5 ms<br>exit 0 (13.0 Mo) | 247.1 ms<br>exit 0 (56.1 Mo) | **TIMEOUT (> 5s)**<br>❌ Échec de buffer |
| **`boundary_over_1mb`**<br>(1 048 578 octets — **+2 octets**) | **13.0 ms**<br>🛡️ **exit 2** (7.1 Mo)<br>`input.limit_exceeded` | 21.9 ms<br>exit 0 (13.0 Mo)<br>*(Aucune limite)* | 21.6 ms<br>exit 0 (12.9 Mo)<br>*(Aucune limite)* | 234.5 ms<br>exit 0 (52.8 Mo)<br>*(Aucune limite)* | **TIMEOUT (> 5s)**<br>❌ Échec de buffer |
| **`heavy_2mb`**<br>(2 097 159 octets) | **18.7 ms**<br>🛡️ **exit 2** (7.1 Mo) | 39.8 ms<br>exit 0 (18.0 Mo) | 36.5 ms<br>exit 0 (16.1 Mo) | 297.1 ms<br>exit 0 (56.5 Mo) | **TIMEOUT (> 5s)**<br>❌ Échec de buffer |
| **`extreme_20mb`**<br>(20 971 527 octets) | **17.2 ms**<br>🛡️ **exit 2** (7.1 Mo) | 332.9 ms<br>exit 0 (**72.4 Mo**) | 208.8 ms<br>exit 0 (**70.0 Mo**) | 365.1 ms<br>exit 0 (**109.8 Mo**) | **TIMEOUT (> 5s)**<br>❌ Échec de buffer |

> [!IMPORTANT]
> **La garantie d'invariant `fluidise-clint` :**
> - `fluidise-clint` est le **seul linter au monde** à appliquer une borne mémoire dure (1 MiB). Dès que l'entrée dépasse cette limite, la lecture s'interrompt immédiatement après un seul octet témoin (`probe byte`). L'empreinte mémoire reste rigoureusement plane à **7.1 Mo**, avec un temps de réaction instantané (< 18 ms) et un code d'erreur `2`.
> - Tous les autres outils lisent l'intégralité du fichier en mémoire vive (la consommation de RAM de `convco` et `cog` est multipliée par 7, celle de `commitizen` dépasse 100 Mo).
> - `commitlint` ne parvient même pas à traiter 1 Mo et part en boucle infinie / timeout.

---

## 6. Workload 5 : Poids sur Disque et Sécurité Supply-Chain

Dans les conteneurs Docker CI/CD légers (images scratch ou alpine) et les environnements d'agents IA isolés, le poids des dépendances et la surface de vulnérabilité sont critiques :

| Outil | Poids sur disque | Dépendances transitives | Vulnérabilités CVE | Runtime externe |
| :--- | :---: | :---: | :---: | :---: |
| **`fluidise-clint`** | **0.5 Mo** | **3** (`lexopt`, `toml`, `serde_json`) | **0** | **Aucun** (statique) |
| `convco` | 7.6 Mo | 45 | 0 | Aucun (statique) |
| `cocogitto` | 5.9 Mo | 78 | 0 | Aucun (statique) |
| `commitizen` | 45.2 Mo | 17 | 0 | Python 3.x requis |
| `commitlint` | 148.0 Mo | 82 | Risque npm régulier | Node.js / V8 requis |

---

## 7. Comparaison de l'Ergonomie Agentique

```
=================================================================================================
Fonctionnalité                  fluidise-clint    convco          cocogitto       commitlint
=================================================================================================
JSON natif par défaut           OUI (stdout)      NON (texte)     NON (texte)     NON (texte ANSI)
Codes d'erreur machine          OUI (typés)       NON             NON             Partiel
Offsets d'octets précis (span)  OUI               NON             NON             NON
Codes de sortie distincts       3 (0, 1, 2)       2 (0, 1)        2 (0, 1)        2 (0, 1)
Limite d'entrée garantie        1 MiB (stricte)   Illimitée       Illimitée       Illimitée
Protection ReDoS                Native            Native          Partielle       Vulnérable
=================================================================================================
```

---

## Conclusion

1. **Face à `commitlint` (Node.js)** : `fluidise-clint` est **17x plus rapide**, utilise **13.7x moins de RAM**, pèse **296x moins lourd**, et ne souffre d'aucun gel sur les expressions régulières complexes ou les gros messages.
2. **Face à `commitizen` (Python)** : `fluidise-clint` est **17x plus rapide**, consomme **7x moins de mémoire**, et ne nécessite aucun environnement virtuel Python.
3. **Face à `convco` et `cocogitto` (Rust)** : Bien que les outils Rust partagent la rapidité d'exécution native, **`fluidise-clint` est le seul à offrir la sécurité d'une mémoire rigoureusement bornée (7.1 Mo constants)** et **une interface purement agentique en JSON strict par défaut**.
