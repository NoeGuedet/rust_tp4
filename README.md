# TP4 - Gestionnaire de Fichiers en Rust

## Bibliotèques vu en cours et utilité

### `std::io::{self, Read, Write}`

La bibliothèque `std::io` permet (avec les fonctions) :

- `Read` : De lire des fichiers
- `Write` : D'écrire des des fichiers.

On utilisera aussi les librairies suivantes pour intéragir avec les fichiers
- `std::fs::*`
Et pour facilier les chemins d'accès des fichiers
- `std::path::Path`



### `chrono::{DateTime, Local}`

La bibliothèque `chrono` est une permet de manipuler les dates et heures.