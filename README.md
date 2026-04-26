# suite_install

Installeur graphique Rust pour les programmes publics de l'organisation GitHub
`rusty-suite`.

L'application charge la liste des depots, recupere la derniere release de chaque
programme, propose les applications a installer ou desinstaller, puis telecharge
les binaires Windows disponibles dans les assets de release.

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

En cas d'erreur, relancer depuis un terminal avec `cargo run` permet de voir :

- l'URL GitHub appelee ;
- le statut HTTP recu ;
- le `content-type` de la reponse ;
- la taille du corps de reponse ;
- un extrait du corps si le JSON ne peut pas etre decode ;
- les chemins utilises pendant l'installation/desinstallation ;
- l'asset de release selectionne et son statut de telechargement ;
- les fichiers de langue trouves dans chaque depot.

### Erreur "error decoding response body"

Cause : reqwest construit sans le feature `gzip` ne peut pas decompresser les
reponses compressees de l'API GitHub.
Correctif : le feature `gzip` est present dans `Cargo.toml` depuis la v0.1.1.

## Architecture du code

```text
suite_install/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── assets/
│   └── img/
│       └── Rusty_suite_install_1.webp   (animation affichee pendant l'install)
└── src/
    ├── main.rs            point d'entree eframe/egui, creation fenetre, cycle UI
    ├── state.rs           etat de l'application, ecrans, modes, options, statuts
    ├── github.rs          client GitHub, repos, releases, fichiers de langue, certs
    ├── screens/
    │   ├── mod.rs
    │   ├── eula.rs        ecran du contrat de licence (EULA)
    │   ├── program_list.rs  liste install/desinstall avec onglets
    │   └── installing.rs  progression en temps reel (install et desinstall)
    └── install/
        ├── mod.rs
        ├── paths.rs       chemins AppData, Program Files, temp, install.json
        ├── runner.rs      orchestration install et desinstall, telechargement
        ├── certificates.rs  verification et installation des certificats publics
        └── shortcuts.rs   creation/suppression des raccourcis Windows (.lnk)
```

## Architecture des dossiers crees a l'installation

Pour chaque programme `<app>`, l'installeur cree et gere les dossiers suivants.

### Binaire installe

```text
%PROGRAMFILES%\
└── rusty-suite\
    └── <app>\
        ├── <app>.exe          executable principal
        └── ...                autres fichiers extraits si asset .zip
```

### Donnees de l'application

```text
%APPDATA%\
└── rusty-suite\
    └── <app>\
        ├── install.json       version, chemin exe, timestamp installation
        └── lang\
            └── <langue>.toml  fichier de langue copie depuis le depot GitHub
```

**install.json** contient :
```json
{
  "version": "v1.2.3",
  "exe_path": "C:\\Program Files\\rusty-suite\\<app>\\<app>.exe",
  "installed_at": "1714134000"
}
```

La presence de ce fichier indique qu'une version est installee.
L'installeur le lit au demarrage pour detecter les mises a jour disponibles.

### Fichiers temporaires

```text
%APPDATA%\
└── rusty-suite\
    └── .tmp\
        └── <app>\
            ├── <asset_release>    binaire telecharge avant copie/extraction
            └── <app>.crt          certificat temporaire si present dans le depot
```

Les dossiers `.tmp` sont nettoyes automatiquement apres installation.

### Raccourcis crees (optionnels)

Selon les options selectionnees dans l'interface :

```text
Bureau    : %USERPROFILE%\Desktop\<app>.lnk
Demarrer  : %APPDATA%\Microsoft\Windows\Start Menu\Programs\Rusty Suite\<app>.lnk
```

## Flux d'installation

1. Lecture des depots publics GitHub de `rusty-suite` (org ou utilisateur).
2. Exclusion de `suite_install` et des depots commencant par `.`.
3. Lecture de la derniere release de chaque programme.
4. Lecture des fichiers `.toml` dans le dossier `lang/` de chaque depot.
5. Calcul des langues communes a tous les programmes.
6. Lecture du record local `install.json` pour detecter les versions installees.
7. Affichage EULA — acceptation obligatoire.
8. Affichage de la liste avec les onglets Installer / Desinstaller.
9. Pour chaque programme selectionne :
   a. Creation des dossiers `Program Files\rusty-suite\<app>\` et `%APPDATA%\rusty-suite\<app>\`.
   b. Verification et installation du certificat public si present dans `certificat_public/<app>.crt`.
   c. Telechargement de l'asset Windows depuis la derniere release.
   d. Extraction du ZIP ou copie de l'executable.
   e. Copie du fichier de langue choisi.
   f. Creation des raccourcis demandes.
   g. Ecriture du nouveau `install.json`.

## Flux de desinstallation

1. Onglet "Desinstaller" dans l'ecran de liste.
2. Seuls les programmes ayant un `install.json` sont affiches.
3. Les dossiers a supprimer sont affiches sous chaque programme.
4. Pour chaque programme selectionne :
   a. Suppression de `%PROGRAMFILES%\rusty-suite\<app>\`.
   b. Suppression des raccourcis bureau et menu Demarrer.
   c. Suppression de `%APPDATA%\rusty-suite\.tmp\<app>\`.
   d. Suppression de `%APPDATA%\rusty-suite\<app>\` (contient `install.json`).
   e. Nettoyage des dossiers parents `rusty-suite\` s'ils sont vides.

## Systeme de langue (Rusty Suite unifie)

Chaque programme suit la convention :

```text
%APPDATA%\rusty-suite\<app>\lang\
    PAYS_langue.toml          ex: CH_fr.toml, FR_fr.toml
    PAYS_langue.default.toml  ex: EN_en.default.toml  (langue de secours)
```

L'installeur copie le fichier de langue choisi depuis le depot GitHub du programme.
Si le programme est lance sans installeur, il tente de telecharger
`EN_en.default.toml` depuis son depot au premier demarrage.

## Certificats publics

Si le depot du programme contient `certificat_public/<app>.crt`, l'installeur :

1. Telecharge le fichier `.crt`.
2. L'installe dans le magasin de l'utilisateur via `certutil -addstore -user Root`.

Cela permet aux programmes signes d'etre reconnus par Windows sans alerte.
