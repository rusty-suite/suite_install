use crate::state::{AppState, ListMode};
use egui::{
    vec2, Color32, ComboBox, Frame, Margin, Rect, Response, RichText, Rounding, ScrollArea,
    Sense, Stroke, Ui, Vec2,
};

const CARD_BG: Color32 = Color32::from_rgb(30, 30, 40);
const CARD_BG_HOVER: Color32 = Color32::from_rgb(38, 38, 52);
const CARD_SELECTED: Color32 = Color32::from_rgb(28, 48, 72);
const CARD_UNSELECT: Color32 = Color32::from_rgb(28, 28, 36);

/// Returns: (start_install, start_uninstall)
pub fn show(ui: &mut Ui, state: &mut AppState) -> (bool, bool) {
    let mut start_install = false;
    let mut start_uninstall = false;

    // ── Tab bar ──────────────────────────────────────────────────────────────
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        tab_button(ui, "⬇  Installer", state.list_mode == ListMode::Install,
            Color32::from_rgb(35, 90, 150), |_| { state.list_mode = ListMode::Install; });
        ui.add_space(4.0);
        tab_button(ui, "🗑  Désinstaller", state.list_mode == ListMode::Uninstall,
            Color32::from_rgb(130, 35, 35), |_| { state.list_mode = ListMode::Uninstall; });
    });
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    match state.list_mode {
        ListMode::Install   => { start_install   = show_install_tab(ui, state); }
        ListMode::Uninstall => { start_uninstall = show_uninstall_tab(ui, state); }
    }

    (start_install, start_uninstall)
}

// ── Install tab ───────────────────────────────────────────────────────────────

fn show_install_tab(ui: &mut Ui, state: &mut AppState) -> bool {
    let mut start_install = false;

    if !state.common_languages.is_empty()
        && !state.common_languages.contains(&state.install_options.selected_language)
    {
        state.install_options.selected_language = state.common_languages[0].clone();
    }

    // Quick-select row
    ui.horizontal(|ui| {
        small_action_btn(ui, "Tout activer",     || state.programs.iter_mut().for_each(|p| p.selected = true));
        small_action_btn(ui, "Tout désactiver",  || state.programs.iter_mut().for_each(|p| p.selected = false));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let n = state.programs.iter().filter(|p| p.selected).count();
            ui.label(RichText::new(format!("{}/{} sélectionné(s)", n, state.programs.len()))
                .size(11.0).color(Color32::from_rgb(130, 130, 130)));
        });
    });
    ui.add_space(6.0);

    // Program cards
    let avail_h = ui.available_size().y - 195.0;
    ScrollArea::vertical().id_salt("install_scroll").max_height(avail_h).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        let programs_len = state.programs.len();
        for idx in 0..programs_len {
            let selected   = state.programs[idx].selected;
            let installed  = state.programs[idx].installed_version.is_some();
            let needs_upd  = state.programs[idx].needs_update;
            let name       = state.programs[idx].repo.name.clone();
            let desc       = state.programs[idx].repo.description.clone().unwrap_or_default();
            let inst_ver   = state.programs[idx].installed_version.clone();
            let avail_ver  = state.programs[idx].release.as_ref().map(|r| r.tag_name.clone());
            let langs      = state.programs[idx].languages.iter().map(|l| language_label(l)).collect::<Vec<_>>().join(" · ");

            let toggled = program_card(
                ui, &name, &desc, inst_ver.as_deref(), avail_ver.as_deref(),
                &langs, installed, needs_upd, selected, false,
            );
            if toggled { state.programs[idx].selected = !selected; }
            ui.add_space(4.0);
        }
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // Bottom options row
    ui.horizontal(|ui| {
        // Language
        ui.label(RichText::new("Langue").size(12.0).color(Color32::from_rgb(180, 180, 180)));
        ui.add_space(4.0);
        if state.common_languages.is_empty() {
            ui.label(RichText::new("aucune langue disponible").size(12.0).color(Color32::from_rgb(200, 80, 80)));
        } else {
            ComboBox::from_id_salt("language_select")
                .selected_text(language_label(&state.install_options.selected_language))
                .show_ui(ui, |ui| {
                    let langs = state.common_languages.clone();
                    for language in &langs {
                        ui.selectable_value(
                            &mut state.install_options.selected_language,
                            language.clone(),
                            language_label(language),
                        );
                    }
                });
        }

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Shortcuts
        ui.label(RichText::new("Raccourcis").size(12.0).color(Color32::from_rgb(180, 180, 180)));
        ui.add_space(6.0);
        toggle_inline(ui, &mut state.install_options.desktop_shortcut, "Bureau");
        ui.add_space(8.0);
        toggle_inline(ui, &mut state.install_options.quicklaunch_shortcut, "Démarrer");
    });

    ui.add_space(10.0);

    let selected = state.programs.iter().filter(|p| p.selected).count();
    let can = selected > 0 && !state.common_languages.is_empty();
    let label = if selected == 0 { "Aucun programme sélectionné".into() }
                else if state.common_languages.is_empty() { "Aucune langue disponible".into() }
                else { format!("Installer {} programme(s)", selected) };

    action_button(ui, &label, if can { Color32::from_rgb(34, 120, 34) } else { Color32::from_rgb(55, 55, 55) }, can,
        || { start_install = true; });

    start_install
}

// ── Uninstall tab ─────────────────────────────────────────────────────────────

fn show_uninstall_tab(ui: &mut Ui, state: &mut AppState) -> bool {
    let mut start_uninstall = false;

    let has_installed = state.programs.iter().any(|p| p.installed_version.is_some());
    if !has_installed {
        ui.add_space(50.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Aucun programme Rusty Suite n'est installé.")
                .color(Color32::from_rgb(120, 120, 120)).size(14.0));
        });
        return false;
    }

    ui.horizontal(|ui| {
        small_action_btn(ui, "Tout activer",    || {
            state.programs.iter_mut().filter(|p| p.installed_version.is_some()).for_each(|p| p.selected = true);
        });
        small_action_btn(ui, "Tout désactiver", || {
            state.programs.iter_mut().filter(|p| p.installed_version.is_some()).for_each(|p| p.selected = false);
        });
    });
    ui.add_space(6.0);

    let avail_h = ui.available_size().y - 130.0;
    ScrollArea::vertical().id_salt("uninstall_scroll").max_height(avail_h).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        let programs_len = state.programs.len();
        for idx in 0..programs_len {
            if state.programs[idx].installed_version.is_none() { continue; }

            let selected  = state.programs[idx].selected;
            let name      = state.programs[idx].repo.name.clone();
            let desc      = state.programs[idx].repo.description.clone().unwrap_or_default();
            let inst_ver  = state.programs[idx].installed_version.clone();
            let avail_ver = state.programs[idx].release.as_ref().map(|r| r.tag_name.clone());

            let install_dir = crate::install::paths::program_files_dir(&name);
            let appdata_dir = crate::install::paths::appdata_dir(&name);
            let paths_hint = format!(
                "{}  ·  {}",
                install_dir.display(), appdata_dir.display()
            );

            let toggled = program_card(
                ui, &name, &paths_hint, inst_ver.as_deref(), avail_ver.as_deref(),
                &desc, true, false, selected, true,
            );
            if toggled { state.programs[idx].selected = !selected; }
            ui.add_space(4.0);
        }
    });

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("⚠").color(Color32::from_rgb(220, 160, 60)).size(13.0));
        ui.label(RichText::new("La désinstallation supprime définitivement les fichiers, données et raccourcis.")
            .size(11.0).color(Color32::from_rgb(180, 140, 60)));
    });
    ui.add_space(8.0);

    let selected = state.programs.iter().filter(|p| p.selected && p.installed_version.is_some()).count();
    let label = if selected == 0 { "Aucun programme sélectionné".into() }
                else { format!("Désinstaller {} programme(s)", selected) };

    action_button(ui, &label,
        if selected > 0 { Color32::from_rgb(150, 35, 35) } else { Color32::from_rgb(55, 55, 55) },
        selected > 0, || { start_uninstall = true; });

    start_uninstall
}

// ── Card widget ───────────────────────────────────────────────────────────────

/// Returns true if the toggle was clicked.
fn program_card(
    ui: &mut Ui,
    name: &str,
    subtitle: &str,
    installed_ver: Option<&str>,
    available_ver: Option<&str>,
    langs: &str,
    installed: bool,
    needs_update: bool,
    selected: bool,
    is_uninstall_mode: bool,
) -> bool {
    let width = ui.available_width();
    let desired = vec2(width, 0.0);

    let (outer_rect, _outer_resp) =
        ui.allocate_exact_size(desired, Sense::hover());

    let mut child = ui.child_ui(outer_rect, egui::Layout::top_down(egui::Align::Min), None);

    let bg = if is_uninstall_mode && selected {
        Color32::from_rgb(50, 26, 26)
    } else if selected {
        CARD_SELECTED
    } else {
        CARD_UNSELECT
    };

    let resp = Frame::none()
        .fill(bg)
        .stroke(Stroke::new(
            1.0,
            if selected && !is_uninstall_mode { Color32::from_rgb(60, 120, 200) }
            else if selected && is_uninstall_mode { Color32::from_rgb(180, 60, 60) }
            else { Color32::from_rgb(50, 50, 60) },
        ))
        .inner_margin(Margin::same(10.0))
        .rounding(Rounding::same(8.0))
        .show(&mut child, |ui| {
            ui.set_min_width(ui.available_width());

            ui.horizontal(|ui| {
                // ── Left: info ──────────────────────────────────────────────
                ui.vertical(|ui| {
                    ui.set_min_width(ui.available_width() - 64.0);

                    // Name + badges row
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(name)
                                .size(14.0)
                                .strong()
                                .color(if selected { Color32::WHITE } else { Color32::from_rgb(170, 170, 170) }),
                        );
                        ui.add_space(6.0);

                        if installed && !is_uninstall_mode {
                            if needs_update {
                                badge(ui, "MÀJ", Color32::from_rgb(220, 160, 60), Color32::from_rgb(60, 44, 10));
                            } else {
                                badge(ui, "✓ installé", Color32::from_rgb(80, 210, 80), Color32::from_rgb(14, 44, 14));
                            }
                        }

                        // Version chips
                        if let Some(iv) = installed_ver {
                            if !is_uninstall_mode {
                                ui.label(RichText::new(format!("v{iv}")).size(11.0).color(Color32::from_rgb(120, 120, 130)));
                            }
                        }
                        if let Some(av) = available_ver {
                            let label = if needs_update && !is_uninstall_mode {
                                format!("→ v{av}")
                            } else if installed_ver.is_none() {
                                format!("v{av}")
                            } else {
                                String::new()
                            };
                            if !label.is_empty() {
                                ui.label(RichText::new(label).size(11.0).color(Color32::from_rgb(90, 160, 230)));
                            }
                        }
                        if is_uninstall_mode {
                            if let Some(iv) = installed_ver {
                                badge(ui, &format!("v{iv}"), Color32::from_rgb(200, 100, 100), Color32::from_rgb(50, 18, 18));
                            }
                        }
                    });

                    // Subtitle / description
                    if !subtitle.is_empty() {
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new(subtitle)
                                .size(11.0)
                                .color(Color32::from_rgb(120, 120, 130)),
                        );
                    }

                    // Language chips
                    if !langs.is_empty() && !is_uninstall_mode {
                        ui.add_space(3.0);
                        ui.label(
                            RichText::new(format!("🌐 {langs}"))
                                .size(10.0)
                                .color(Color32::from_rgb(90, 110, 140)),
                        );
                    }
                });

                // ── Right: toggle ────────────────────────────────────────────
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(4.0);
                    draw_toggle(ui, selected, is_uninstall_mode);
                });
            });
        });

    // The whole card is clickable
    let full_rect = resp.response.rect;
    let click_resp = ui.interact(full_rect, ui.id().with(name), Sense::click());
    if click_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    click_resp.clicked()
}

/// Draws a pill-style toggle switch and returns its response.
fn draw_toggle(ui: &mut Ui, on: bool, danger: bool) -> Response {
    let size = Vec2::new(40.0, 20.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let t = if on { 1.0_f32 } else { 0.0_f32 };
        let track_color = if on {
            if danger { Color32::from_rgb(180, 50, 50) } else { Color32::from_rgb(50, 170, 50) }
        } else {
            Color32::from_rgb(60, 60, 72)
        };

        let painter = ui.painter();
        let radius = rect.height() / 2.0;

        // Track
        painter.rect_filled(rect, Rounding::same(radius), track_color);

        // Knob
        let knob_r = radius - 2.0;
        let knob_x = rect.left() + radius + t * (rect.width() - radius * 2.0);
        let knob_center = egui::pos2(knob_x, rect.center().y);
        painter.circle_filled(knob_center, knob_r, Color32::WHITE);
    }

    resp
}

// ── Helper widgets ────────────────────────────────────────────────────────────

fn tab_button(ui: &mut Ui, label: &str, active: bool, active_color: Color32, on_click: impl FnOnce(&mut Ui)) {
    let btn = egui::Button::new(RichText::new(label).color(Color32::WHITE).size(13.0))
        .fill(if active { active_color } else { Color32::from_rgb(38, 38, 50) })
        .stroke(Stroke::new(1.0, if active { active_color } else { Color32::from_rgb(60, 60, 75) }))
        .min_size(Vec2::new(130.0, 28.0));
    if ui.add(btn).clicked() { on_click(ui); }
}

fn badge(ui: &mut Ui, text: &str, fg: Color32, bg: Color32) {
    let label = RichText::new(text).size(10.0).color(fg).strong();
    Frame::none()
        .fill(bg)
        .rounding(Rounding::same(4.0))
        .inner_margin(Margin { left: 5.0, right: 5.0, top: 1.0, bottom: 1.0 })
        .show(ui, |ui| { ui.label(label); });
}

fn toggle_inline(ui: &mut Ui, value: &mut bool, label: &str) {
    ui.horizontal(|ui| {
        if draw_toggle(ui, *value, false).clicked() { *value = !*value; }
        ui.add_space(4.0);
        ui.label(RichText::new(label).size(12.0).color(Color32::from_rgb(180, 180, 180)));
    });
}

fn small_action_btn(ui: &mut Ui, label: &str, on_click: impl FnOnce()) {
    let btn = egui::Button::new(RichText::new(label).size(11.0).color(Color32::from_rgb(190, 190, 190)))
        .fill(Color32::from_rgb(45, 45, 58))
        .stroke(Stroke::new(1.0, Color32::from_rgb(65, 65, 80)));
    if ui.add(btn).clicked() { on_click(); }
}

fn action_button(ui: &mut Ui, label: &str, color: Color32, enabled: bool, on_click: impl FnOnce()) {
    let btn = egui::Button::new(RichText::new(label).color(Color32::WHITE).strong().size(13.0))
        .fill(color)
        .min_size(Vec2::new(220.0, 32.0));
    if ui.add_enabled(enabled, btn).clicked() { on_click(); }
}

fn language_label(file_name: &str) -> String {
    file_name
        .strip_suffix(".default.toml")
        .or_else(|| file_name.strip_suffix(".toml"))
        .unwrap_or(file_name)
        .replace('_', "-")
}
