use crate::state::{AppState, ListMode};
use egui::{Color32, ComboBox, RichText, ScrollArea, Ui};

/// Returns: (start_install, start_uninstall)
pub fn show(ui: &mut Ui, state: &mut AppState) -> (bool, bool) {
    let mut start_install = false;
    let mut start_uninstall = false;

    // Mode tab bar
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let install_selected = state.list_mode == ListMode::Install;
        let uninstall_selected = state.list_mode == ListMode::Uninstall;

        let install_btn = egui::Button::new(
            RichText::new("⬇  Installer").color(Color32::WHITE),
        )
        .fill(if install_selected {
            Color32::from_rgb(40, 100, 160)
        } else {
            Color32::from_rgb(40, 40, 50)
        });
        if ui.add(install_btn).clicked() {
            state.list_mode = ListMode::Install;
        }

        let uninstall_btn = egui::Button::new(
            RichText::new("🗑  Désinstaller").color(Color32::WHITE),
        )
        .fill(if uninstall_selected {
            Color32::from_rgb(150, 40, 40)
        } else {
            Color32::from_rgb(40, 40, 50)
        });
        if ui.add(uninstall_btn).clicked() {
            state.list_mode = ListMode::Uninstall;
        }
    });

    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);

    match state.list_mode {
        ListMode::Install => {
            start_install = show_install_tab(ui, state);
        }
        ListMode::Uninstall => {
            start_uninstall = show_uninstall_tab(ui, state);
        }
    }

    (start_install, start_uninstall)
}

fn show_install_tab(ui: &mut Ui, state: &mut AppState) -> bool {
    let mut start_install = false;

    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new("Programmes disponibles")
                .size(17.0)
                .strong()
                .color(Color32::WHITE),
        );
        ui.label(
            RichText::new("Tous sont activés par défaut.")
                .size(12.0)
                .color(Color32::from_rgb(160, 160, 160)),
        );
        ui.add_space(6.0);
    });

    if !state.common_languages.is_empty()
        && !state.common_languages.contains(&state.install_options.selected_language)
    {
        state.install_options.selected_language = state.common_languages[0].clone();
    }

    ui.horizontal(|ui| {
        if ui.small_button("Tout sélectionner").clicked() {
            state.programs.iter_mut().for_each(|p| p.selected = true);
        }
        if ui.small_button("Tout désélectionner").clicked() {
            state.programs.iter_mut().for_each(|p| p.selected = false);
        }
    });
    ui.add_space(4.0);

    let avail_h = ui.available_size().y - 260.0;
    ScrollArea::vertical().id_salt("install_scroll").max_height(avail_h).show(ui, |ui| {
        for prog in state.programs.iter_mut() {
            let installed = prog.installed_version.is_some();
            let update = prog.needs_update;
            let is_selected = prog.selected;

            ui.horizontal(|ui| {
                ui.toggle_value(&mut prog.selected, if is_selected { "●" } else { "○" });

                if installed {
                    let (badge, color) = if update {
                        ("MÀJ", Color32::from_rgb(220, 160, 60))
                    } else {
                        ("✓", Color32::from_rgb(80, 210, 80))
                    };
                    ui.label(RichText::new(badge).color(color).size(11.0));
                }

                ui.label(
                    RichText::new(&prog.repo.name)
                        .strong()
                        .color(if is_selected { Color32::WHITE } else { Color32::GRAY }),
                );

                if let Some(ver) = &prog.installed_version {
                    ui.label(RichText::new(format!("v{ver}")).size(11.0).color(Color32::from_rgb(140, 140, 140)));
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
                        ui.label(RichText::new(label).size(11.0).color(Color32::from_rgb(100, 180, 240)));
                    }
                }
            });

            if let Some(desc) = &prog.repo.description {
                if !desc.is_empty() {
                    ui.indent("desc", |ui| {
                        ui.label(RichText::new(desc).size(12.0).color(Color32::from_rgb(150, 150, 150)));
                    });
                }
            }
            if !prog.languages.is_empty() {
                ui.indent("languages", |ui| {
                    let langs = prog.languages.iter().map(|l| language_label(l)).collect::<Vec<_>>().join(", ");
                    ui.label(RichText::new(format!("langues: {langs}")).size(11.0).color(Color32::from_rgb(120, 120, 120)));
                });
            }
            ui.add_space(4.0);
        }
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // Language selection
    ui.horizontal(|ui| {
        ui.label(RichText::new("Langue :").color(Color32::WHITE));
        if state.common_languages.is_empty() {
            ui.label(RichText::new("aucune langue commune").color(Color32::from_rgb(220, 80, 80)));
        } else {
            ComboBox::from_id_salt("language_select")
                .selected_text(language_label(&state.install_options.selected_language))
                .show_ui(ui, |ui| {
                    for language in &state.common_languages {
                        ui.selectable_value(
                            &mut state.install_options.selected_language,
                            language.clone(),
                            language_label(language),
                        );
                    }
                });
        }
    });
    ui.add_space(6.0);

    // Shortcut options
    ui.label(RichText::new("Raccourcis").strong().color(Color32::WHITE));
    ui.add_space(4.0);
    ui.checkbox(&mut state.install_options.desktop_shortcut, "Raccourci sur le bureau");
    ui.checkbox(&mut state.install_options.quicklaunch_shortcut, "Ajouter au menu Démarrer (Rusty Suite)");
    ui.add_space(10.0);

    let selected = state.programs.iter().filter(|p| p.selected).count();
    let can = selected > 0 && !state.common_languages.is_empty();

    let label = if selected == 0 {
        "Aucun programme sélectionné".to_string()
    } else if state.common_languages.is_empty() {
        "Aucune langue commune".to_string()
    } else {
        format!("Installer {} programme(s)", selected)
    };

    let btn = egui::Button::new(RichText::new(label).color(Color32::WHITE).strong())
        .fill(if can { Color32::from_rgb(40, 130, 40) } else { Color32::from_rgb(60, 60, 60) });
    if ui.add_enabled(can, btn).clicked() {
        start_install = true;
    }

    start_install
}

fn show_uninstall_tab(ui: &mut Ui, state: &mut AppState) -> bool {
    let mut start_uninstall = false;

    let installed_programs: Vec<_> = state.programs.iter().filter(|p| p.installed_version.is_some()).cloned().collect();

    if installed_programs.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Aucun programme Rusty Suite n'est installé.").color(Color32::GRAY).size(14.0));
        });
        return false;
    }

    ui.vertical_centered(|ui| {
        ui.label(RichText::new("Programmes installés").size(17.0).strong().color(Color32::WHITE));
        ui.label(
            RichText::new("Sélectionnez les programmes à désinstaller complètement.")
                .size(12.0)
                .color(Color32::from_rgb(160, 160, 160)),
        );
        ui.add_space(6.0);
    });

    ui.horizontal(|ui| {
        if ui.small_button("Tout sélectionner").clicked() {
            state.programs.iter_mut()
                .filter(|p| p.installed_version.is_some())
                .for_each(|p| p.selected = true);
        }
        if ui.small_button("Tout désélectionner").clicked() {
            state.programs.iter_mut()
                .filter(|p| p.installed_version.is_some())
                .for_each(|p| p.selected = false);
        }
    });
    ui.add_space(4.0);

    let avail_h = ui.available_size().y - 140.0;
    ScrollArea::vertical().id_salt("uninstall_scroll").max_height(avail_h).show(ui, |ui| {
        for prog in state.programs.iter_mut() {
            if prog.installed_version.is_none() {
                continue;
            }
            let is_selected = prog.selected;
            ui.horizontal(|ui| {
                ui.toggle_value(&mut prog.selected, if is_selected { "●" } else { "○" });
                ui.label(
                    RichText::new(&prog.repo.name)
                        .strong()
                        .color(if is_selected { Color32::WHITE } else { Color32::GRAY }),
                );
                if let Some(ver) = &prog.installed_version {
                    ui.label(RichText::new(format!("v{ver}")).size(11.0).color(Color32::from_rgb(140, 140, 140)));
                }
            });
            ui.indent("uninstall_detail", |ui| {
                let install_dir = crate::install::paths::program_files_dir(&prog.repo.name);
                let appdata_dir = crate::install::paths::appdata_dir(&prog.repo.name);
                ui.label(RichText::new(format!("📁 {}", install_dir.display())).size(10.0).color(Color32::from_rgb(100, 100, 100)));
                ui.label(RichText::new(format!("📁 {}", appdata_dir.display())).size(10.0).color(Color32::from_rgb(100, 100, 100)));
            });
            ui.add_space(4.0);
        }
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // Warning
    ui.label(
        RichText::new("⚠  La désinstallation supprime définitivement les fichiers, données et raccourcis.")
            .size(11.0)
            .color(Color32::from_rgb(220, 160, 60)),
    );
    ui.add_space(6.0);

    let selected = state.programs.iter()
        .filter(|p| p.selected && p.installed_version.is_some())
        .count();

    let label = if selected == 0 {
        "Aucun programme sélectionné".to_string()
    } else {
        format!("Désinstaller {} programme(s)", selected)
    };

    let btn = egui::Button::new(RichText::new(label).color(Color32::WHITE).strong())
        .fill(if selected > 0 { Color32::from_rgb(160, 40, 40) } else { Color32::from_rgb(60, 60, 60) });
    if ui.add_enabled(selected > 0, btn).clicked() {
        start_uninstall = true;
    }

    start_uninstall
}

fn language_label(file_name: &str) -> String {
    file_name
        .strip_suffix(".default.toml")
        .or_else(|| file_name.strip_suffix(".toml"))
        .unwrap_or(file_name)
        .replace('_', "-")
}
