# Changelog

Toutes les modifications notables de ce projet sont documentées ici.

Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/)
et le projet respecte le [Versionnage sémantique](https://semver.org/lang/fr/) :
`MAJEUR.MINEUR.CORRECTIF`, tags Git `vX.Y.Z` identiques à la version de `Cargo.toml`.

## [Non publié]

## [0.2.1] - 2026-11-26

### Corrigé
- Le logo animé pendant l'installation laissait encore des traces en cascade :
  les frames du fichier WebP étaient fusionnées les unes sur les autres.
  Chaque frame remplace maintenant entièrement la précédente.

## [0.2.0] - 2026-11-26

### Ajouté
- Vérification au chargement qu'une release est réellement disponible :
  release publiée, binaire Windows présent et non vide, lien de téléchargement joignable.
- Message dédié lorsqu'aucun programme n'a de release disponible (EN, FR, DE, IT).
- Tests automatisés : sliders, sélection d'asset, vérification des releases, animation.
- Binaire Linux x86_64 publié avec la release.

### Modifié
- Les programmes sans release installable sont masqués de l'onglet Installer
  (ils restent désinstallables s'ils sont déjà installés).
- Les releases stables sont préférées aux pré-releases.
- Le User-Agent HTTP reprend automatiquement la version du paquet.

### Corrigé
- Les sliders des cartes de programme ne réagissaient pas au clic.
- Le logo animé pendant l'installation semblait se dupliquer / s'empiler.
- Compilation Linux : suppression de code mort qui faisait planter `rustc` 1.94.1.

## [0.1.0] - 2026-04-28

Première pré-release publique (tag historique `Win_&_Linux`).

[Non publié]: https://github.com/rusty-suite/suite_install/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/rusty-suite/suite_install/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/rusty-suite/suite_install/compare/Win_%26_Linux...v0.2.0
[0.1.0]: https://github.com/rusty-suite/suite_install/releases/tag/Win_%26_Linux
