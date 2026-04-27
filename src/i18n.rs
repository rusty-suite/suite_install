/// All UI strings used by the installer.
/// Template placeholders: {n}, {total}, {error} — replace with .replace() at call sites.
pub struct Translations {
    // ── App-wide ──────────────────────────────────────────────────────────────
    pub app_title:              &'static str,

    // ── Language selection screen ─────────────────────────────────────────────
    pub select_lang_subtitle:   &'static str,
    pub other_languages:        &'static str,
    pub continue_btn:           &'static str,

    // ── EULA screen ───────────────────────────────────────────────────────────
    pub eula_title:             &'static str,
    pub eula_text:              &'static str,
    pub eula_accept_label:      &'static str,
    pub accept_btn:             &'static str,
    pub cancel_btn:             &'static str,

    // ── Program list — toolbar & options ─────────────────────────────────────
    pub tab_install:            &'static str,
    pub tab_uninstall:          &'static str,
    pub select_all:             &'static str,
    pub deselect_all:           &'static str,
    /// Template: {n}/{total}
    pub n_selected:             &'static str,
    pub language_label:         &'static str,
    pub no_language:            &'static str,
    pub shortcuts_label:        &'static str,
    pub desktop_label:          &'static str,
    pub start_menu_label:       &'static str,

    // ── Program list — action button labels ───────────────────────────────────
    pub no_program_selected:    &'static str,
    pub no_language_available:  &'static str,
    /// Template: {n}
    pub install_n_programs:     &'static str,
    pub no_programs_installed:  &'static str,
    pub uninstall_warning:      &'static str,
    /// Template: {n}
    pub uninstall_n_programs:   &'static str,

    // ── Card badges ───────────────────────────────────────────────────────────
    pub badge_update:           &'static str,
    pub badge_installed:        &'static str,

    // ── Installing screen ─────────────────────────────────────────────────────
    pub install_done:           &'static str,
    pub uninstall_done:         &'static str,
    pub installing:             &'static str,
    pub uninstalling:           &'static str,
    pub active_actions_header:  &'static str,
    /// Template: {n}/{total}
    pub n_programs_done:        &'static str,
    pub no_active_action:       &'static str,
    pub close_btn:              &'static str,
    /// Template: {msg}
    pub status_error:           &'static str,

    // ── Loading screen ────────────────────────────────────────────────────────
    pub loading:                &'static str,
    /// Template: {error}
    pub loading_error:          &'static str,

    // ── Runner log messages (install) — templates use {branch}, {path}, etc. ──
    pub log_starting_install:   &'static str,
    pub log_creating_install_dir: &'static str,
    pub log_creating_data_dir:  &'static str,
    pub log_checking_cert:      &'static str,
    pub log_installing_cert:    &'static str,
    pub log_searching_asset:    &'static str,
    pub log_copying_lang:       &'static str,
    pub log_creating_desktop:   &'static str,
    pub log_creating_start:     &'static str,
    pub log_writing_record:     &'static str,
    pub log_install_done:       &'static str,
    pub log_no_release:         &'static str,
    pub log_no_windows_asset:   &'static str,
    /// Template: {asset} {size}
    pub log_asset_info:         &'static str,
    /// Template: {size}
    pub log_size_verified:      &'static str,
    pub log_sha256_checking:    &'static str,
    pub log_sha256_ok:          &'static str,
    pub log_no_sha256:          &'static str,
    pub log_extracting_zip:     &'static str,
    /// Template: {path}
    pub log_copying_to:         &'static str,

    // ── Runner log messages (uninstall) ───────────────────────────────────────
    /// Template: {name}
    pub log_starting_uninstall: &'static str,
    /// Template: {path}
    pub log_removing:           &'static str,
    /// Template: {path}
    pub log_dir_already_removed: &'static str,
    pub log_removing_shortcuts: &'static str,
    pub log_uninstall_done:     &'static str,
    /// Template: {path} {error}
    pub log_cannot_remove:      &'static str,

    // ── Pre-check phase ───────────────────────────────────────────────────────
    pub precheck_title:         &'static str,
    pub precheck_checking_conn: &'static str,
    /// Template: {ms}
    pub precheck_conn_ok:       &'static str,
    /// Template: {error}
    pub precheck_conn_failed:   &'static str,
    /// Template: {size}
    pub precheck_total_size:    &'static str,
    pub precheck_speed_test:    &'static str,
    /// Template: {speed}
    pub precheck_speed:         &'static str,
    /// Template: {eta}
    pub precheck_eta:           &'static str,
    pub precheck_done:          &'static str,
}

// ── English ───────────────────────────────────────────────────────────────────

static EN: Translations = Translations {
    app_title:             "Rusty Suite — Installer",
    select_lang_subtitle:  "Select install language",
    other_languages:       "Other available languages",
    continue_btn:          "Continue →",

    eula_title:            "End User License Agreement",
    eula_text:             "\
END USER LICENSE AGREEMENT — RUSTY SUITE
==========================================

By installing this software, you accept the following terms:

1. RIGHTS & INTELLECTUAL PROPERTY
   All Rusty Suite programs are the exclusive property of rusty-suite.com.
   Any unauthorised reproduction, redistribution or modification is strictly prohibited.

2. PERMITTED USE
   The programs are provided for legal and ethical purposes only.
   Any use for illegal, malicious, harmful purposes or contrary to applicable law
   is strictly prohibited.

3. LIMITATION OF LIABILITY
   The author (rusty-suite.com) disclaims all liability for any direct
   or indirect damage caused by the use of the installed programs.
   You use this software at your own risk.

4. THIRD-PARTY LICENCES
   Each program may include libraries under separate open-source licences.
   By installing a program, you also accept the licence terms of each
   of these libraries, available in the corresponding repositories on GitHub.

5. DATA PROTECTION (GDPR)
   \u{2713} No personal data is collected, transmitted or stored by this installer
     or by the Rusty Suite programs.
   \u{2713} No tracking, telemetry or analytics is implemented.
   \u{2713} The only network accesses performed are: downloading binaries from GitHub,
     certificate verification, and downloading language files.

6. LANGUAGE OPTION
   The installer offers a language selection during installation.
   Available languages are determined by the translation files present
   in each program's GitHub repository.
   The chosen language is applied to all installed programs.

7. UPDATES
   This installer may offer updates to installed programs.
   These updates are subject to the same licence terms.

By clicking \u{ab}Accept and continue\u{bb}, you confirm that you have read, understood
and accepted all of these terms.
",
    eula_accept_label:     "I have read and accept the terms of the license agreement",
    accept_btn:            "Accept and continue \u{2192}",
    cancel_btn:            "Cancel",

    tab_install:           "\u{2b07}  Install",
    tab_uninstall:         "\u{1f5d1}  Uninstall",
    select_all:            "Select all",
    deselect_all:          "Deselect all",
    n_selected:            "{n}/{total} selected",
    language_label:        "Language",
    no_language:           "no language available",
    shortcuts_label:       "Shortcuts",
    desktop_label:         "Desktop",
    start_menu_label:      "Start",

    no_program_selected:   "No program selected",
    no_language_available: "No language available",
    install_n_programs:    "Install {n} program(s)",
    no_programs_installed: "No Rusty Suite program is installed.",
    uninstall_warning:     "Uninstalling permanently deletes files, data and shortcuts.",
    uninstall_n_programs:  "Uninstall {n} program(s)",

    badge_update:          "UPD",
    badge_installed:       "\u{2713} installed",

    install_done:          "Installation complete \u{2713}",
    uninstall_done:        "Uninstallation complete \u{2713}",
    installing:            "Installing\u{2026}",
    uninstalling:          "Uninstalling\u{2026}",
    active_actions_header: "Current actions",
    n_programs_done:       "{n}/{total} program(s) processed",
    no_active_action:      "No active action.",
    close_btn:             "Close",
    status_error:          "ERROR: {msg}",

    loading:               "Loading program list\u{2026}",
    loading_error:         "Error: {error}",

    log_starting_install:    "Starting installation from branch {branch}",
    log_creating_install_dir: "Creating install directory: {path}",
    log_creating_data_dir:   "Creating data directory: {path}",
    log_checking_cert:       "Checking certificate: {url}",
    log_installing_cert:     "Installing public certificate",
    log_searching_asset:     "Searching for and downloading Windows asset",
    log_copying_lang:        "Copying selected language: {lang}",
    log_creating_desktop:    "Creating desktop shortcut",
    log_creating_start:      "Creating Start Menu shortcut",
    log_writing_record:      "Writing install.json file",
    log_install_done:        "Installation complete",
    log_no_release:          "no release available",
    log_no_windows_asset:    "no Windows asset found in release",
    log_asset_info:          "Asset: {asset} ({size})",
    log_size_verified:       "Size verified: {size}",
    log_sha256_checking:     "Verifying SHA-256\u{2026}",
    log_sha256_ok:           "SHA-256 validated \u{2713}",
    log_no_sha256:           "No .sha256 file provided \u{2014} size check only",
    log_extracting_zip:      "Extracting ZIP archive",
    log_copying_to:          "Copying to {path}",
    log_starting_uninstall:  "Starting uninstallation of {name}",
    log_removing:            "Removing {path}",
    log_dir_already_removed: "Install directory not found (already removed): {path}",
    log_removing_shortcuts:  "Removing shortcuts",
    log_uninstall_done:      "Uninstallation complete",
    log_cannot_remove:       "\u{26a0} Cannot remove {path}: {error}",

    precheck_title:         "Pre-installation check",
    precheck_checking_conn: "Checking GitHub connectivity\u{2026}",
    precheck_conn_ok:       "GitHub reachable ({ms} ms)",
    precheck_conn_failed:   "GitHub unreachable: {error}",
    precheck_total_size:    "Total to download: {size}",
    precheck_speed_test:    "Measuring connection speed\u{2026}",
    precheck_speed:         "Speed: {speed}",
    precheck_eta:           "Estimated time: {eta}",
    precheck_done:          "Pre-check complete",
};

// ── French (Switzerland) ──────────────────────────────────────────────────────

static FR: Translations = Translations {
    app_title:             "Rusty Suite — Installeur",
    select_lang_subtitle:  "Sélectionnez la langue d'installation",
    other_languages:       "Autres langues disponibles",
    continue_btn:          "Continuer \u{2192}",

    eula_title:            "Contrat de Licence Utilisateur Final",
    eula_text:             "\
CONTRAT DE LICENCE UTILISATEUR FINAL — RUSTY SUITE
=====================================================

En installant ce logiciel, vous acceptez les conditions suivantes :

1. DROITS & PROPRIÉTÉ INTELLECTUELLE
   Tous les programmes de la Rusty Suite sont la propriété exclusive de rusty-suite.com.
   Toute reproduction, redistribution ou modification non autorisée est strictement interdite.

2. UTILISATION AUTORISÉE
   Les programmes sont fournis à des fins légales et éthiques uniquement.
   Toute utilisation à des fins illégales, malveillantes, nuisibles ou contraires
   à la législation en vigueur est formellement interdite.

3. LIMITATION DE RESPONSABILITÉ
   L'auteur (rusty-suite.com) décline toute responsabilité pour tout dommage direct
   ou indirect causé par l'utilisation des programmes installés.
   Vous utilisez ces logiciels à vos propres risques.

4. LICENCES TIERCES
   Chaque programme peut inclure des bibliothèques sous licences open source distinctes.
   En installant un programme, vous acceptez également les termes de licence de chacune
   de ces bibliothèques, disponibles dans les dépôts correspondants sur GitHub.

5. PROTECTION DES DONNÉES (RGPD)
   \u{2713} Aucune donnée personnelle n'est collectée, transmise ou stockée par cet installeur
     ou par les programmes de la Rusty Suite.
   \u{2713} Aucun tracking, télémétrie ou analytique n'est mis en place.
   \u{2713} Les seuls accès réseau effectués sont : le téléchargement des binaires depuis GitHub,
     la vérification des certificats, et le téléchargement des fichiers de langue.

6. OPTION DE LANGUE
   L'installeur propose une sélection de langue lors de l'installation.
   Les langues disponibles sont déterminées par les fichiers de traduction présents
   dans chaque dépôt GitHub des programmes de la suite.
   La langue choisie est appliquée à l'ensemble des programmes installés.

7. MISES À JOUR
   Cet installeur peut proposer des mises à jour des programmes installés.
   Ces mises à jour sont soumises aux mêmes conditions de licence.

En cliquant sur « Accepter et continuer », vous confirmez avoir lu, compris
et accepté l'intégralité des présentes conditions.
",
    eula_accept_label:     "J'ai lu et j'accepte les termes du contrat de licence",
    accept_btn:            "Accepter et continuer \u{2192}",
    cancel_btn:            "Annuler",

    tab_install:           "\u{2b07}  Installer",
    tab_uninstall:         "\u{1f5d1}  Désinstaller",
    select_all:            "Tout activer",
    deselect_all:          "Tout désactiver",
    n_selected:            "{n}/{total} sélectionné(s)",
    language_label:        "Langue",
    no_language:           "aucune langue disponible",
    shortcuts_label:       "Raccourcis",
    desktop_label:         "Bureau",
    start_menu_label:      "Démarrer",

    no_program_selected:   "Aucun programme sélectionné",
    no_language_available: "Aucune langue disponible",
    install_n_programs:    "Installer {n} programme(s)",
    no_programs_installed: "Aucun programme Rusty Suite n'est installé.",
    uninstall_warning:     "La désinstallation supprime définitivement les fichiers, données et raccourcis.",
    uninstall_n_programs:  "Désinstaller {n} programme(s)",

    badge_update:          "MÀJ",
    badge_installed:       "\u{2713} installé",

    install_done:          "Installation terminée \u{2713}",
    uninstall_done:        "Désinstallation terminée \u{2713}",
    installing:            "Installation en cours\u{2026}",
    uninstalling:          "Désinstallation en cours\u{2026}",
    active_actions_header: "Actions exactes en cours",
    n_programs_done:       "{n}/{total} programme(s) traités",
    no_active_action:      "Aucune action active.",
    close_btn:             "Fermer",
    status_error:          "ERREUR: {msg}",

    loading:               "Chargement de la liste des programmes\u{2026}",
    loading_error:         "Erreur : {error}",

    log_starting_install:    "Démarrage de l'installation depuis la branche {branch}",
    log_creating_install_dir: "Création du dossier d'installation: {path}",
    log_creating_data_dir:   "Création du dossier de données: {path}",
    log_checking_cert:       "Vérification du certificat: {url}",
    log_installing_cert:     "Installation du certificat public",
    log_searching_asset:     "Recherche et téléchargement de l'asset Windows",
    log_copying_lang:        "Copie de la langue sélectionnée: {lang}",
    log_creating_desktop:    "Création du raccourci Bureau",
    log_creating_start:      "Création du raccourci Menu Démarrer",
    log_writing_record:      "Écriture du fichier install.json",
    log_install_done:        "Installation terminée",
    log_no_release:          "aucune release disponible",
    log_no_windows_asset:    "pas d'asset Windows trouvé dans la release",
    log_asset_info:          "Asset: {asset} ({size})",
    log_size_verified:       "Taille vérifiée: {size}",
    log_sha256_checking:     "Vérification SHA-256\u{2026}",
    log_sha256_ok:           "SHA-256 validé \u{2713}",
    log_no_sha256:           "Aucun fichier .sha256 fourni \u{2014} vérification de taille uniquement",
    log_extracting_zip:      "Extraction de l'archive ZIP",
    log_copying_to:          "Copie vers {path}",
    log_starting_uninstall:  "Démarrage de la désinstallation de {name}",
    log_removing:            "Suppression de {path}",
    log_dir_already_removed: "Dossier install absent (déjà supprimé): {path}",
    log_removing_shortcuts:  "Suppression des raccourcis",
    log_uninstall_done:      "Désinstallation terminée",
    log_cannot_remove:       "\u{26a0} Impossible de supprimer {path}: {error}",

    precheck_title:         "Vérification préalable",
    precheck_checking_conn: "Vérification de la connectivité GitHub\u{2026}",
    precheck_conn_ok:       "GitHub accessible ({ms} ms)",
    precheck_conn_failed:   "GitHub inaccessible : {error}",
    precheck_total_size:    "Total à télécharger : {size}",
    precheck_speed_test:    "Mesure de la vitesse de connexion\u{2026}",
    precheck_speed:         "Vitesse : {speed}",
    precheck_eta:           "Durée estimée : {eta}",
    precheck_done:          "Vérification terminée",
};

// ── German (Switzerland) ──────────────────────────────────────────────────────

static DE: Translations = Translations {
    app_title:             "Rusty Suite — Installationsprogramm",
    select_lang_subtitle:  "Installationssprache auswählen",
    other_languages:       "Weitere verfügbare Sprachen",
    continue_btn:          "Weiter \u{2192}",

    eula_title:            "Endbenutzer-Lizenzvertrag",
    eula_text:             "\
ENDBENUTZER-LIZENZVERTRAG — RUSTY SUITE
=========================================

Durch die Installation dieser Software akzeptieren Sie die folgenden Bedingungen:

1. RECHTE & GEISTIGES EIGENTUM
   Alle Programme der Rusty Suite sind ausschliessliches Eigentum von rusty-suite.com.
   Jede nicht autorisierte Vervielfältigung, Weitergabe oder Änderung ist streng verboten.

2. ERLAUBTE NUTZUNG
   Die Programme werden ausschliesslich für legale und ethische Zwecke bereitgestellt.
   Jede Nutzung für illegale, schädliche oder gegen geltendes Recht verstossende Zwecke
   ist ausdrücklich verboten.

3. HAFTUNGSBESCHRÄNKUNG
   Der Autor (rusty-suite.com) übernimmt keine Haftung für direkte
   oder indirekte Schäden, die durch die Nutzung der installierten Programme entstehen.
   Sie verwenden diese Software auf eigenes Risiko.

4. DRITTANBIETER-LIZENZEN
   Jedes Programm kann Bibliotheken unter separaten Open-Source-Lizenzen enthalten.
   Mit der Installation eines Programms akzeptieren Sie auch die Lizenzbedingungen
   dieser Bibliotheken, die in den entsprechenden GitHub-Repositories verfügbar sind.

5. DATENSCHUTZ (DSGVO)
   \u{2713} Es werden keine personenbezogenen Daten durch dieses Installationsprogramm
     oder durch die Rusty Suite-Programme gesammelt, übertragen oder gespeichert.
   \u{2713} Kein Tracking, Telemetrie oder Analyse ist implementiert.
   \u{2713} Die einzigen Netzwerkzugriffe sind: Herunterladen von Binärdateien von GitHub,
     Zertifikatsüberprüfung und Herunterladen von Sprachdateien.

6. SPRACHOPTION
   Das Installationsprogramm bietet während der Installation eine Sprachauswahl.
   Verfügbare Sprachen werden durch die Übersetzungsdateien in den GitHub-Repositories
   jedes Programms bestimmt.
   Die gewählte Sprache wird auf alle installierten Programme angewendet.

7. AKTUALISIERUNGEN
   Dieses Installationsprogramm kann Updates für installierte Programme anbieten.
   Diese Updates unterliegen denselben Lizenzbedingungen.

Durch Klicken auf \u{ab}Akzeptieren und weiter\u{bb} bestätigen Sie, dass Sie alle
diese Bedingungen gelesen, verstanden und akzeptiert haben.
",
    eula_accept_label:     "Ich habe die Lizenzbedingungen gelesen und stimme ihnen zu",
    accept_btn:            "Akzeptieren und weiter \u{2192}",
    cancel_btn:            "Abbrechen",

    tab_install:           "\u{2b07}  Installieren",
    tab_uninstall:         "\u{1f5d1}  Deinstallieren",
    select_all:            "Alle auswählen",
    deselect_all:          "Alle abwählen",
    n_selected:            "{n}/{total} ausgewählt",
    language_label:        "Sprache",
    no_language:           "keine Sprache verfügbar",
    shortcuts_label:       "Verknüpfungen",
    desktop_label:         "Desktop",
    start_menu_label:      "Start",

    no_program_selected:   "Kein Programm ausgewählt",
    no_language_available: "Keine Sprache verfügbar",
    install_n_programs:    "{n} Programm(e) installieren",
    no_programs_installed: "Kein Rusty Suite-Programm ist installiert.",
    uninstall_warning:     "Die Deinstallation löscht Dateien, Daten und Verknüpfungen dauerhaft.",
    uninstall_n_programs:  "{n} Programm(e) deinstallieren",

    badge_update:          "AKT",
    badge_installed:       "\u{2713} installiert",

    install_done:          "Installation abgeschlossen \u{2713}",
    uninstall_done:        "Deinstallation abgeschlossen \u{2713}",
    installing:            "Installation läuft\u{2026}",
    uninstalling:          "Deinstallation läuft\u{2026}",
    active_actions_header: "Aktuelle Aktionen",
    n_programs_done:       "{n}/{total} Programm(e) verarbeitet",
    no_active_action:      "Keine aktive Aktion.",
    close_btn:             "Schliessen",
    status_error:          "FEHLER: {msg}",

    loading:               "Programmliste wird geladen\u{2026}",
    loading_error:         "Fehler: {error}",

    log_starting_install:    "Installation wird gestartet von Branch {branch}",
    log_creating_install_dir: "Installationsverzeichnis wird erstellt: {path}",
    log_creating_data_dir:   "Datenverzeichnis wird erstellt: {path}",
    log_checking_cert:       "Zertifikat wird geprüft: {url}",
    log_installing_cert:     "Öffentliches Zertifikat wird installiert",
    log_searching_asset:     "Windows-Asset wird gesucht und heruntergeladen",
    log_copying_lang:        "Ausgewählte Sprache wird kopiert: {lang}",
    log_creating_desktop:    "Desktop-Verknüpfung wird erstellt",
    log_creating_start:      "Startmenü-Verknüpfung wird erstellt",
    log_writing_record:      "install.json wird geschrieben",
    log_install_done:        "Installation abgeschlossen",
    log_no_release:          "keine Release verfügbar",
    log_no_windows_asset:    "kein Windows-Asset in der Release gefunden",
    log_asset_info:          "Asset: {asset} ({size})",
    log_size_verified:       "Grösse überprüft: {size}",
    log_sha256_checking:     "SHA-256 wird überprüft\u{2026}",
    log_sha256_ok:           "SHA-256 validiert \u{2713}",
    log_no_sha256:           "Keine .sha256-Datei vorhanden \u{2014} nur Grössenprüfung",
    log_extracting_zip:      "ZIP-Archiv wird entpackt",
    log_copying_to:          "Wird kopiert nach {path}",
    log_starting_uninstall:  "Deinstallation von {name} wird gestartet",
    log_removing:            "Wird entfernt: {path}",
    log_dir_already_removed: "Installationsverzeichnis nicht gefunden (bereits entfernt): {path}",
    log_removing_shortcuts:  "Verknüpfungen werden entfernt",
    log_uninstall_done:      "Deinstallation abgeschlossen",
    log_cannot_remove:       "\u{26a0} Kann {path} nicht entfernen: {error}",

    precheck_title:         "Vorinstallationsprüfung",
    precheck_checking_conn: "GitHub-Verbindung wird geprüft\u{2026}",
    precheck_conn_ok:       "GitHub erreichbar ({ms} ms)",
    precheck_conn_failed:   "GitHub nicht erreichbar: {error}",
    precheck_total_size:    "Gesamtdownload: {size}",
    precheck_speed_test:    "Verbindungsgeschwindigkeit wird gemessen\u{2026}",
    precheck_speed:         "Geschwindigkeit: {speed}",
    precheck_eta:           "Geschätzte Zeit: {eta}",
    precheck_done:          "Vorprüfung abgeschlossen",
};

// ── Italian (Switzerland) ─────────────────────────────────────────────────────

static IT: Translations = Translations {
    app_title:             "Rusty Suite — Programma di installazione",
    select_lang_subtitle:  "Seleziona la lingua di installazione",
    other_languages:       "Altre lingue disponibili",
    continue_btn:          "Continua \u{2192}",

    eula_title:            "Contratto di Licenza con l'Utente Finale",
    eula_text:             "\
CONTRATTO DI LICENZA CON L'UTENTE FINALE — RUSTY SUITE
========================================================

Installando questo software, si accettano le seguenti condizioni:

1. DIRITTI E PROPRIETÀ INTELLETTUALE
   Tutti i programmi della Rusty Suite sono di esclusiva proprietà di rusty-suite.com.
   Qualsiasi riproduzione, ridistribuzione o modifica non autorizzata è severamente vietata.

2. USO CONSENTITO
   I programmi sono forniti esclusivamente per scopi legali ed etici.
   Qualsiasi utilizzo per scopi illegali, dannosi o contrari alla legge vigente
   è espressamente vietato.

3. LIMITAZIONE DI RESPONSABILITÀ
   L'autore (rusty-suite.com) declina ogni responsabilità per danni diretti
   o indiretti causati dall'uso dei programmi installati.
   L'utilizzo di questo software avviene a proprio rischio.

4. LICENZE DI TERZE PARTI
   Ogni programma può includere librerie con licenze open source separate.
   Installando un programma, si accettano anche i termini di licenza di ciascuna
   di queste librerie, disponibili nei repository corrispondenti su GitHub.

5. PROTEZIONE DEI DATI (GDPR)
   \u{2713} Nessun dato personale viene raccolto, trasmesso o archiviato da questo installer
     o dai programmi della Rusty Suite.
   \u{2713} Nessun tracciamento, telemetria o analisi è implementato.
   \u{2713} Gli unici accessi di rete effettuati sono: download dei binari da GitHub,
     verifica dei certificati e download dei file di lingua.

6. OPZIONE DI LINGUA
   L'installer propone una selezione della lingua durante l'installazione.
   Le lingue disponibili sono determinate dai file di traduzione presenti
   nei repository GitHub di ciascun programma.
   La lingua scelta viene applicata a tutti i programmi installati.

7. AGGIORNAMENTI
   Questo installer può proporre aggiornamenti per i programmi installati.
   Questi aggiornamenti sono soggetti agli stessi termini di licenza.

Facendo clic su \u{ab}Accetta e continua\u{bb}, si conferma di aver letto, compreso
e accettato integralmente queste condizioni.
",
    eula_accept_label:     "Ho letto e accetto i termini del contratto di licenza",
    accept_btn:            "Accetta e continua \u{2192}",
    cancel_btn:            "Annulla",

    tab_install:           "\u{2b07}  Installa",
    tab_uninstall:         "\u{1f5d1}  Disinstalla",
    select_all:            "Seleziona tutto",
    deselect_all:          "Deseleziona tutto",
    n_selected:            "{n}/{total} selezionato/i",
    language_label:        "Lingua",
    no_language:           "nessuna lingua disponibile",
    shortcuts_label:       "Scorciatoie",
    desktop_label:         "Desktop",
    start_menu_label:      "Avvio",

    no_program_selected:   "Nessun programma selezionato",
    no_language_available: "Nessuna lingua disponibile",
    install_n_programs:    "Installa {n} programma/i",
    no_programs_installed: "Nessun programma Rusty Suite è installato.",
    uninstall_warning:     "La disinstallazione elimina definitivamente i file, i dati e le scorciatoie.",
    uninstall_n_programs:  "Disinstalla {n} programma/i",

    badge_update:          "AGG",
    badge_installed:       "\u{2713} installato",

    install_done:          "Installazione completata \u{2713}",
    uninstall_done:        "Disinstallazione completata \u{2713}",
    installing:            "Installazione in corso\u{2026}",
    uninstalling:          "Disinstallazione in corso\u{2026}",
    active_actions_header: "Azioni in corso",
    n_programs_done:       "{n}/{total} programma/i elaborato/i",
    no_active_action:      "Nessuna azione attiva.",
    close_btn:             "Chiudi",
    status_error:          "ERRORE: {msg}",

    loading:               "Caricamento dell'elenco dei programmi\u{2026}",
    loading_error:         "Errore: {error}",

    log_starting_install:    "Avvio dell'installazione dal branch {branch}",
    log_creating_install_dir: "Creazione della cartella di installazione: {path}",
    log_creating_data_dir:   "Creazione della cartella dati: {path}",
    log_checking_cert:       "Verifica del certificato: {url}",
    log_installing_cert:     "Installazione del certificato pubblico",
    log_searching_asset:     "Ricerca e download dell'asset Windows",
    log_copying_lang:        "Copia della lingua selezionata: {lang}",
    log_creating_desktop:    "Creazione del collegamento sul Desktop",
    log_creating_start:      "Creazione del collegamento nel Menu Start",
    log_writing_record:      "Scrittura del file install.json",
    log_install_done:        "Installazione completata",
    log_no_release:          "nessuna release disponibile",
    log_no_windows_asset:    "nessun asset Windows trovato nella release",
    log_asset_info:          "Asset: {asset} ({size})",
    log_size_verified:       "Dimensione verificata: {size}",
    log_sha256_checking:     "Verifica SHA-256\u{2026}",
    log_sha256_ok:           "SHA-256 validato \u{2713}",
    log_no_sha256:           "Nessun file .sha256 fornito \u{2014} solo verifica della dimensione",
    log_extracting_zip:      "Estrazione dell'archivio ZIP",
    log_copying_to:          "Copia in {path}",
    log_starting_uninstall:  "Avvio della disinstallazione di {name}",
    log_removing:            "Rimozione di {path}",
    log_dir_already_removed: "Cartella di installazione non trovata (già rimossa): {path}",
    log_removing_shortcuts:  "Rimozione dei collegamenti",
    log_uninstall_done:      "Disinstallazione completata",
    log_cannot_remove:       "\u{26a0} Impossibile rimuovere {path}: {error}",

    precheck_title:         "Verifica preliminare",
    precheck_checking_conn: "Verifica della connettività GitHub\u{2026}",
    precheck_conn_ok:       "GitHub raggiungibile ({ms} ms)",
    precheck_conn_failed:   "GitHub non raggiungibile: {error}",
    precheck_total_size:    "Totale da scaricare: {size}",
    precheck_speed_test:    "Misurazione della velocità di connessione\u{2026}",
    precheck_speed:         "Velocità: {speed}",
    precheck_eta:           "Tempo stimato: {eta}",
    precheck_done:          "Verifica preliminare completata",
};

// ── Selector ──────────────────────────────────────────────────────────────────

/// Returns the translations for the given language file name.
/// Defaults to English for any unknown code.
pub fn get(lang: &str) -> &'static Translations {
    if lang.starts_with("CH_fr") { &FR }
    else if lang.starts_with("CH_de") { &DE }
    else if lang.starts_with("CH_it") { &IT }
    else { &EN }
}
