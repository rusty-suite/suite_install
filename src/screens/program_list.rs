use crate::state::AppState;
use egui::{Color32, RichText, ScrollArea, Ui};

pub fn show(ui: &mut Ui, state: &mut AppState) -> bool {
    let mut start_install = false;

    ui.vertical_centered(|ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new("Programmes disponibles")
                .size(18.0)
                .strong()
                .color(Color32::WHITE),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new("Sélectionnez les programmes à installer. Tous sont activés par défaut.")
                .size(13.0)
                .color(Color32::from_rgb(160, 160, 160)),
        );
        ui.add_space(10.0);
    });

    // Quick select toolbar
    ui.horizontal(|ui| {
        if ui.small_button("Tout sélectionner").clicked() {
            state.programs.iter_mut().for_each(|p| p.selected = true);
        }
        if ui.small_button("Tout désélectionner").clicked() {
            state.programs.iter_mut().for_each(|p| p.selected = false);
        }
    });
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);

    let avail_h = ui.available_size().y - 200.0;
    ScrollArea::vertical().max_height(avail_h).show(ui, |ui| {
        for prog in state.programs.iter_mut() {
            let installed = prog.installed_version.is_some();
            let update = prog.needs_update;

            let is_selected = prog.selected;
            ui.horizontal(|ui| {
                // Toggle switch (checkbox styled)
                ui.toggle_value(&mut prog.selected, if is_selected { "●" } else { "○" });

                // Status badge
                if installed {
                    if update {
                        ui.label(
                            RichText::new("MÀAAAAAJ")
                                .color(Color32::from_rgb(220, 160, 60))
                                .size(11.0),
                        );
                    } else {
                        ui.label(
                            RichText::new("INSTALLÉ")
                                .color(Color32::from_rgb(80, 210, 80))
                                .size(11.0),
                        );
                    }
                }

                ui.label(
                    RichText::new(&prog.repo.name)
                        .strong()
                        .color(if is_selected {
                            Color32::WHITE
                        } else {
                            Color32::GRAY
                        }),
                );

                if let Some(ver) = &prog.installed_version {
                    ui.label(
                        RichText::new(format!("v{}", ver))
                            .size(11.0)
                            .color(Color32::from_rgb(140, 140, 140)),
                    );
                }
                if let Some(release) = &prog.release {
                    let label = if update {
                        format!("→ v{}", release.tag_name)
                    } else if !installed {
                        format!("v{}", release.tag_name)
                    } else {
                        String::new()
                    };
                    if !label.is_empty() {
                        ui.label(
                            RichText::new(label)
                                .size(11.0)
                                .color(Color32::from_rgb(100, 180, 240)),
                        );
                    }
                }
            });

            if let Some(desc) = &prog.repo.description {
                if !desc.is_empty() {
                    ui.indent("desc", |ui| {
                        ui.label(
                            RichText::new(desc)
                                .size(12.0)
                                .color(Color32::from_rgb(150, 150, 150)),
                        );
                    });
                }
            }
            ui.add_space(4.0);
        }
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // Shortcut options
    ui.label(RichText::new("Raccourcis").strong().color(Color32::WHITE));
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut state.install_options.desktop_shortcut,
            "Créer un raccourci sur le bureau",
        );
    });
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut state.install_options.quicklaunch_shortcut,
            "Ajouter au menu Démarrer (Rusty Suite)",
        );
    });

    ui.add_space(12.0);

    let selected_count = state.programs.iter().filter(|p| p.selected).count();
    ui.horizontal(|ui| {
        let label = if selected_count == 0 {
            "Aucun programme sélectionné".to_string()
        } else {
            format!("Installer {} programme(s)", selected_count)
        };

        let btn = egui::Button::new(RichText::new(label).color(Color32::WHITE).strong()).fill(
            if selected_count > 0 {
                Color32::from_rgb(40, 130, 40)
            } else {
                Color32::from_rgb(60, 60, 60)
            },
        );

        if ui.add_enabled(selected_count > 0, btn).clicked() {
            start_install = true;
        }
    });

    start_install
}
