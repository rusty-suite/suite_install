# suite_install

Installeur graphique Rust pour les programmes publics de l'organisation GitHub
`rusty-suite`.

L'application charge la liste des depots, recupere la derniere release de chaque
programme, propose les applications a installer, puis telecharge les binaires
Windows disponibles dans les assets de release.

## Lancement

```powershell
cargo run
```

Pour une version optimisee :

```powershell
cargo build --release
```

## Logs

Les logs sont ecrits sur `stderr`.

En cas d'erreur comme `Erreur : error decoding response body`, relancer
l'application depuis un terminal avec `cargo run` permet de voir :

- l'URL GitHub appelee ;
- le statut HTTP recu ;
- le `content-type` ;
- la taille du corps de reponse ;
- un extrait du corps si le JSON ne peut pas etre decode ;
- les chemins utilises pendant l'installation ;
- l'asset de release selectionne et son statut de telechargement.
- les fichiers de langue trouves dans chaque depot.

## Architecture du code

```text
suite_install/
├── Cargo.toml
├── Cargo.lock
├── README.md
└── src/
    ├── main.rs
    ├── state.rs
    ├── github.rs
    ├── screens/
    │   ├── mod.rs
    │   ├── eula.rs
    │   ├── program_list.rs
    │   └── installing.rs
    └── install/
        ├── mod.rs
        ├── paths.rs
        ├── runner.rs
        ├── certificates.rs
        └── shortcuts.rs
```

### Contenu des fichiers principaux

- `src/main.rs` : point d'entree eframe/egui, creation de la fenetre, cycle UI,
  chargement des programmes et demarrage de l'installation.
- `src/state.rs` : etat de l'application, ecrans disponibles, options
  d'installation et statuts affiches.
- `src/github.rs` : client GitHub, recuperation des depots publics, de la
  derniere release, construction des URLs raw et des URLs de certificat.
- `src/screens/eula.rs` : ecran du contrat de licence.
- `src/screens/program_list.rs` : liste des programmes, versions, selection et
  options de langue/raccourcis.
- `src/screens/installing.rs` : affichage de la progression et des erreurs
  d'installation.
- `src/install/paths.rs` : chemins d'installation, chemins AppData, dossier
  temporaire et fichier `install.json`.
- `src/install/runner.rs` : orchestration de l'installation, telechargement des
  assets, extraction ZIP, copie de la langue choisie et ecriture du record.
- `src/install/certificates.rs` : verification, telechargement et installation
  des certificats publics.
- `src/install/shortcuts.rs` : creation des raccourcis Bureau et menu Demarrer
  sous Windows.

## Architecture des dossiers crees a l'installation

Pour chaque programme `<app>`, l'installeur cree les dossiers suivants.

```text
%PROGRAMFILES%/
└── rusty-suite/
    └── <app>/
        ├── <app>.exe ou asset .exe telecharge
        └── fichiers extraits si l'asset est un .zip
```

Contenu :

- executable installe ;
- fichiers extraits depuis l'asset `.zip`, le cas echeant.

```text
%APPDATA%/
└── rusty-suite/
    └── <app>/
        ├── install.json
        └── lang/
            └── EN_en.default.toml
```

Contenu :

- `install.json` : version installee, chemin de l'executable et date
  d'installation au format timestamp Unix ;
- `lang/<langue>.toml` : fichier de langue choisi dans l'interface, copie
  depuis le dossier `lang/` du depot du programme.

```text
%APPDATA%/
└── rusty-suite/
    └── .tmp/
        └── <app>/
            ├── asset telecharge
            └── <app>.crt
```

Contenu :

- asset de release telecharge avant copie ou extraction ;
- certificat public temporaire si
  `certificat_public/<app>.crt` existe dans le depot du programme.

## Raccourcis crees

Selon les options selectionnees dans l'interface :

- Bureau : `%USERPROFILE%\Desktop\<app>.lnk` ;
- Menu Demarrer : dossier `Rusty Suite` dans le repertoire Programs de
  l'utilisateur Windows.

## Flux d'installation

1. Lecture des depots publics GitHub de `rusty-suite`.
2. Exclusion de `suite_install` et des depots commencant par `.`.
3. Lecture de la derniere release de chaque programme.
4. Lecture des fichiers `.toml` disponibles dans le dossier `lang/` de chaque
   depot.
5. Calcul des langues communes a tous les programmes.
6. Affichage uniquement des langues communes dans l'interface.
7. Lecture du record local `install.json`, si present.
8. Creation des dossiers d'installation.
9. Installation du certificat public si disponible.
10. Telechargement de l'asset Windows depuis la derniere release.
11. Extraction du ZIP ou copie de l'executable.
12. Copie du fichier de langue choisi.
13. Creation des raccourcis demandes.
14. Ecriture du nouveau `install.json`.
