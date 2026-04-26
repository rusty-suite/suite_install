use crate::state::{AppState, ListMode};
use egui::{
    Color32, ComboBox, Frame, Id, Margin, RichText, Rounding, ScrollArea, Sense, Stroke, Ui, Vec2,
};

// ── Palette ───────────────────────────────────────────────────────────────────

const CARD_BG_OFF:  Color32 = Color32::from_rgb(28, 28, 36);
const CARD_BG_ON:   Color32 = Color32::from_rgb(22, 42, 68);
const CARD_BORDER_OFF: Color32 = Color32::from_rgb(50, 50, 62);
const CARD_BORDER_ON:  Color32 = Color32::from_rgb(55, 115, 200);
const CARD_BORDER_DEL: Color32 = Color32::from_rgb(180, 55, 55);
const CARD_BG_DEL:  Color32 = Color32::from_rgb(44, 20, 20);

// ── Entry point ───────────────────────────────────────────────────────────────

/// Returns (start_install, start_uninstall)
pub fn show(ui: &mut Ui, state: &mut AppState) -> (bool, bool) {
    let mut start_install   = false;
    let mut start_uninstall = false;

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        tab_btn(ui, "⬇  Installer",    state.list_mode == ListMode::Install,
            Color32::from_rgb(30, 85, 145), || state.list_mode = ListMode::Install);
        ui.add_space(4.0);
        tab_btn(ui, "🗑  Désinstaller", state.list_mode == ListMode::Uninstall,
            Color32::from_rgb(125, 30, 30), || state.list_mode = ListMode::Uninstall);
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

    // Quick-select toolbar
    ui.horizontal(|ui| {
        small_btn(ui, "Tout activer",    || state.programs.iter_mut().for_each(|p| p.selected = true));
        small_btn(ui, "Tout désactiver", || state.programs.iter_mut().for_each(|p| p.selected = false));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let n = state.programs.iter().filter(|p| p.selected).count();
            ui.label(RichText::new(format!("{}/{} sélectionné(s)", n, state.programs.len()))
                .size(11.0).color(Color32::from_rgb(110, 110, 120)));
        });
    });
    ui.add_space(6.0);

    // Scrollable card list
    let avail_h = ui.available_size().y - 190.0;
    ScrollArea::vertical()
        .id_salt("install_scroll")
        .max_height(avail_h.max(120.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            let n = state.programs.len();
            for idx in 0..n {
                let name      = state.programs[idx].repo.name.clone();
                let desc      = state.programs[idx].repo.description.clone().unwrap_or_default();
                let inst_ver  = state.programs[idx].installed_version.clone();
                let avail_ver = state.programs[idx].release.as_ref().map(|r| r.tag_name.clone());
                let langs     = state.programs[idx].languages.iter()
                    .map(|l| language_label(l)).collect::<Vec<_>>().join("  ·  ");
                let installed  = inst_ver.is_some();
                let needs_upd  = state.programs[idx].needs_update;
                let selected   = state.programs[idx].selected;

                let action = program_card(
                    ui, &name, &desc,
                    inst_ver.as_deref(), avail_ver.as_deref(),
                    &langs, installed, needs_upd, selected, false,
                );
                if action == CardAction::Toggle  { state.programs[idx].selected = !selected; }
                ui.add_space(5.0);
            }
        });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // Options row
    ui.horizontal(|ui| {
        ui.label(RichText::new("Langue").size(12.0).color(Color32::from_rgb(170, 170, 170)));
        ui.add_space(4.0);
        if state.common_languages.is_empty() {
            ui.label(RichText::new("aucune langue disponible")
                .size(12.0).color(Color32::from_rgb(200, 80, 80)));
        } else {
            let langs = state.common_languages.clone();
            ComboBox::from_id_salt("language_select")
                .selected_text(language_label(&state.install_options.selected_language))
                .show_ui(ui, |ui| {
                    for lang in &langs {
                        ui.selectable_value(
                            &mut state.install_options.selected_language,
                            lang.clone(),
                            language_label(lang),
                        );
                    }
                });
        }
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label(RichText::new("Raccourcis").size(12.0).color(Color32::from_rgb(170, 170, 170)));
        ui.add_space(6.0);
        toggle_inline(ui, &mut state.install_options.desktop_shortcut,     "Bureau");
        ui.add_space(10.0);
        toggle_inline(ui, &mut state.install_options.quicklaunch_shortcut, "Démarrer");
    });

    ui.add_space(10.0);

    let selected = state.programs.iter().filter(|p| p.selected).count();
    let can      = selected > 0 && !state.common_languages.is_empty();
    let label    = if selected == 0             { "Aucun programme sélectionné".into() }
                   else if state.common_languages.is_empty() { "Aucune langue disponible".into() }
                   else                         { format!("Installer {} programme(s)", selected) };

    action_btn(ui, &label, if can { Color32::from_rgb(30, 115, 30) } else { Color32::from_rgb(50, 50, 55) }, can,
        || start_install = true);

    start_install
}

// ── Uninstall tab ─────────────────────────────────────────────────────────────

fn show_uninstall_tab(ui: &mut Ui, state: &mut AppState) -> bool {
    let mut start_uninstall = false;

    if !state.programs.iter().any(|p| p.installed_version.is_some()) {
        ui.add_space(50.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Aucun programme Rusty Suite n'est installé.")
                .color(Color32::from_rgb(110, 110, 110)).size(14.0));
        });
        return false;
    }

    ui.horizontal(|ui| {
        small_btn(ui, "Tout activer",    || {
            state.programs.iter_mut().filter(|p| p.installed_version.is_some()).for_each(|p| p.selected = true);
        });
        small_btn(ui, "Tout désactiver", || {
            state.programs.iter_mut().filter(|p| p.installed_version.is_some()).for_each(|p| p.selected = false);
        });
    });
    ui.add_space(6.0);

    let avail_h = ui.available_size().y - 120.0;
    ScrollArea::vertical()
        .id_salt("uninstall_scroll")
        .max_height(avail_h.max(120.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            let n = state.programs.len();
            for idx in 0..n {
                if state.programs[idx].installed_version.is_none() { continue; }

                let name     = state.programs[idx].repo.name.clone();
                let inst_ver = state.programs[idx].installed_version.clone();
                let avail_ver= state.programs[idx].release.as_ref().map(|r| r.tag_name.clone());
                let selected = state.programs[idx].selected;

                let install_dir = crate::install::paths::program_files_dir(&name);
                let appdata_dir = crate::install::paths::appdata_dir(&name);
                let detail = format!(
                    "📁  {}  \n📁  {}",
                    install_dir.display(), appdata_dir.display()
                );

                let action = program_card(
                    ui, &name, &detail,
                    inst_ver.as_deref(), avail_ver.as_deref(),
                    "", true, false, selected, true,
                );
                if action == CardAction::Toggle { state.programs[idx].selected = !selected; }
                ui.add_space(5.0);
            }
        });

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("⚠").color(Color32::from_rgb(220, 160, 60)));
        ui.label(RichText::new("La désinstallation supprime définitivement les fichiers, données et raccourcis.")
            .size(11.0).color(Color32::from_rgb(175, 135, 55)));
    });
    ui.add_space(8.0);

    let selected = state.programs.iter()
        .filter(|p| p.selected && p.installed_version.is_some()).count();
    let label = if selected == 0 { "Aucun programme sélectionné".into() }
                else { format!("Désinstaller {} programme(s)", selected) };

    action_btn(ui, &label,
        if selected > 0 { Color32::from_rgb(140, 30, 30) } else { Color32::from_rgb(50, 50, 55) },
        selected > 0, || start_uninstall = true);

    start_uninstall
}

// ── Card widget ───────────────────────────────────────────────────────────────

#[derive(PartialEq)]
enum CardAction { None, Toggle }

fn program_card(
    ui:           &mut Ui,
    name:         &str,
    subtitle:     &str,
    installed_ver: Option<&str>,
    available_ver: Option<&str>,
    langs:        &str,
    installed:    bool,
    needs_update: bool,
    selected:     bool,
    danger:       bool,
) -> CardAction {
    let card_id    = Id::new("card").with(name);
    let expand_id  = card_id.with("expand");
    let hover_id   = card_id.with("hover");

    // Persistent expanded state
    let mut expanded: bool = ui.data(|d| d.get_temp(expand_id).unwrap_or(false));

    // Hover animation  0.0 → 1.0
    let is_hovered = ui.data(|d| d.get_temp::<bool>(hover_id).unwrap_or(false));
    let hover_t = ui.ctx().animate_bool(hover_id, is_hovered);

    // Interpolate background
    let bg_base = if danger && selected { CARD_BG_DEL }
                  else if selected      { CARD_BG_ON  }
                  else                  { CARD_BG_OFF };
    let bg = lerp_color(bg_base, brighten(bg_base, 18), hover_t);
    let border = if danger && selected  { CARD_BORDER_DEL }
                 else if selected       { CARD_BORDER_ON  }
                 else                   { lerp_color(CARD_BORDER_OFF, Color32::from_rgb(80, 80, 100), hover_t) };

    let mut action = CardAction::None;

    let frame_resp = Frame::none()
        .fill(bg)
        .stroke(Stroke::new(1.0, border))
        .inner_margin(Margin::same(10.0))
        .rounding(Rounding::same(8.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            // ── Header row (always visible) ───────────────────────────────────
            ui.horizontal(|ui| {
                // Left: name + badges
                ui.vertical(|ui| {
                    ui.set_min_width(ui.available_width() - 56.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(name).size(14.0).strong()
                                .color(if selected { Color32::WHITE } else { Color32::from_rgb(165, 165, 175) }),
                        );
                        ui.add_space(5.0);
                        if installed && !danger {
                            if needs_update {
                                badge(ui, "MÀJ",        Color32::from_rgb(220, 160, 55), Color32::from_rgb(55, 40, 8));
                            } else {
                                badge(ui, "✓ installé", Color32::from_rgb(70, 200, 70),  Color32::from_rgb(10, 40, 10));
                            }
                        }
                        if let Some(iv) = installed_ver {
                            if !danger {
                                ui.label(RichText::new(format!("v{iv}")).size(11.0).color(Color32::from_rgb(110, 110, 120)));
                            }
                        }
                        if let Some(av) = available_ver {
                            let lbl = if needs_update && !danger { format!("→ v{av}") }
                                      else if installed_ver.is_none() { format!("v{av}") }
                                      else { String::new() };
                            if !lbl.is_empty() {
                                ui.label(RichText::new(lbl).size(11.0).color(Color32::from_rgb(85, 155, 225)));
                            }
                        }
                        if danger {
                            if let Some(iv) = installed_ver {
                                badge(ui, &format!("v{iv}"), Color32::from_rgb(200, 95, 95), Color32::from_rgb(48, 14, 14));
                            }
                        }
                        // Expand chevron
                        let chevron = if expanded { "▲" } else { "▼" };
                        ui.label(RichText::new(chevron).size(10.0).color(Color32::from_rgb(90, 90, 110)));
                    });
                });

                // Right: pill toggle
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(2.0);
                    if draw_pill(ui, selected, danger) { action = CardAction::Toggle; }
                });
            });

            // ── Expanded details ──────────────────────────────────────────────
            if expanded {
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                if !subtitle.is_empty() {
                    ui.label(RichText::new(subtitle).size(12.0).color(Color32::from_rgb(145, 145, 155)));
                    ui.add_space(4.0);
                }
                if !langs.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🌐").size(11.0));
                        ui.label(RichText::new(langs).size(11.0).color(Color32::from_rgb(85, 110, 155)));
                    });
                }
            }
        });

    // Detect hover over the whole card
    let rect = frame_resp.response.rect;
    let body_resp = ui.interact(rect, card_id.with("body"), Sense::click());
    let now_hovered = body_resp.hovered();
    ui.data_mut(|d| d.insert_temp(hover_id, now_hovered));

    if body_resp.clicked() {
        expanded = !expanded;
        ui.data_mut(|d| d.insert_temp(expand_id, expanded));
    }

    // Request repaint while animation is running
    if hover_t > 0.01 && hover_t < 0.99 {
        ui.ctx().request_repaint();
    }

    action
}

// ── Pill toggle ───────────────────────────────────────────────────────────────

/// Returns true if clicked.
fn draw_pill(ui: &mut Ui, on: bool, danger: bool) -> bool {
    let size = Vec2::new(38.0, 20.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let anim_id = resp.id.with("pill_anim");
        let t = ui.ctx().animate_bool(anim_id, on);

        let track = if on {
            if danger { lerp_color(Color32::from_rgb(60, 60, 72), Color32::from_rgb(170, 45, 45), t) }
            else      { lerp_color(Color32::from_rgb(60, 60, 72), Color32::from_rgb(40, 160, 40), t) }
        } else {
            Color32::from_rgb(60, 60, 72)
        };

        let r = rect.height() / 2.0;
        ui.painter().rect_filled(rect, Rounding::same(r), track);

        let knob_r  = r - 2.5;
        let knob_x  = rect.left() + r + t * (rect.width() - r * 2.0);
        ui.painter().circle_filled(egui::pos2(knob_x, rect.center().y), knob_r, Color32::WHITE);
    }

    resp.clicked()
}

// ── Small helpers ─────────────────────────────────────────────────────────────

fn badge(ui: &mut Ui, text: &str, fg: Color32, bg: Color32) {
    Frame::none()
        .fill(bg)
        .rounding(Rounding::same(4.0))
        .inner_margin(Margin { left: 5.0, right: 5.0, top: 1.0, bottom: 1.0 })
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(10.0).color(fg).strong());
        });
}

fn toggle_inline(ui: &mut Ui, value: &mut bool, label: &str) {
    ui.horizontal(|ui| {
        if draw_pill(ui, *value, false) { *value = !*value; }
        ui.add_space(4.0);
        ui.label(RichText::new(label).size(12.0).color(Color32::from_rgb(175, 175, 175)));
    });
}

fn tab_btn(ui: &mut Ui, label: &str, active: bool, active_color: Color32, on_click: impl FnOnce()) {
    let btn = egui::Button::new(RichText::new(label).color(Color32::WHITE).size(13.0))
        .fill(if active { active_color } else { Color32::from_rgb(36, 36, 48) })
        .stroke(Stroke::new(1.0,
            if active { active_color } else { Color32::from_rgb(58, 58, 72) }))
        .min_size(Vec2::new(130.0, 28.0));
    if ui.add(btn).clicked() { on_click(); }
}

fn small_btn(ui: &mut Ui, label: &str, on_click: impl FnOnce()) {
    let btn = egui::Button::new(RichText::new(label).size(11.0).color(Color32::from_rgb(185, 185, 185)))
        .fill(Color32::from_rgb(42, 42, 55))
        .stroke(Stroke::new(1.0, Color32::from_rgb(62, 62, 76)));
    if ui.add(btn).clicked() { on_click(); }
}

fn action_btn(ui: &mut Ui, label: &str, color: Color32, enabled: bool, on_click: impl FnOnce()) {
    let btn = egui::Button::new(RichText::new(label).color(Color32::WHITE).strong().size(13.0))
        .fill(color)
        .min_size(Vec2::new(220.0, 32.0));
    if ui.add_enabled(enabled, btn).clicked() { on_click(); }
}

// ── Color utilities ───────────────────────────────────────────────────────────

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_premultiplied(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
        (a.a() as f32 + (b.a() as f32 - a.a() as f32) * t) as u8,
    )
}

fn brighten(c: Color32, amount: u8) -> Color32 {
    Color32::from_rgb(
        c.r().saturating_add(amount),
        c.g().saturating_add(amount),
        c.b().saturating_add(amount),
    )
}

fn language_label(file_name: &str) -> String {
    file_name
        .strip_suffix(".default.toml")
        .or_else(|| file_name.strip_suffix(".toml"))
        .unwrap_or(file_name)
        .replace('_', "-")
}
