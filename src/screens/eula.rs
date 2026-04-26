use egui::{Color32, RichText, ScrollArea, Ui};

const EULA_TEXT: &str = "\
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
   ✓ Aucune donnée personnelle n'est collectée, transmise ou stockée par cet installeur
     ou par les programmes de la Rusty Suite.
   ✓ Aucun tracking, télémétrie ou analytique n'est mis en place.
   ✓ Les seuls accès réseau effectués sont : le téléchargement des binaires depuis GitHub,
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
";

pub fn show(ui: &mut Ui, accepted: &mut bool) -> bool {
    let mut proceed = false;

    ui.vertical_centered(|ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new("Rusty Suite — Installeur")
                .size(22.0)
                .strong()
                .color(Color32::WHITE),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new("Contrat de Licence Utilisateur Final")
                .size(14.0)
                .color(Color32::from_rgb(180, 180, 180)),
        );
        ui.add_space(12.0);
    });

    let avail = ui.available_size();
    ScrollArea::vertical()
        .max_height(avail.y - 110.0)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut EULA_TEXT.to_string())
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace)
                    .interactive(false),
            );
        });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.checkbox(accepted, "");
        ui.label(
            RichText::new("J'ai lu et j'accepte les termes du contrat de licence")
                .color(Color32::WHITE),
        );
    });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let btn = egui::Button::new(
            RichText::new("Accepter et continuer →")
                .color(Color32::WHITE)
                .strong(),
        )
        .fill(if *accepted {
            Color32::from_rgb(40, 140, 40)
        } else {
            Color32::from_rgb(60, 60, 60)
        });

        if ui.add_enabled(*accepted, btn).clicked() {
            proceed = true;
        }

        ui.add_space(16.0);
        if ui
            .button(RichText::new("Annuler").color(Color32::from_rgb(220, 80, 80)))
            .clicked()
        {
            std::process::exit(0);
        }
    });

    proceed
}
