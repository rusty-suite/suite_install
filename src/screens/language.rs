use crate::i18n::Translations;
use egui::{Color32, RichText, Ui};

/// Base languages compiled into the binary — always available regardless of GitHub content.
/// Format matches the TOML filename convention used by installed apps.
pub const BASE_LANGUAGES: &[(&str, &str, &str)] = &[
    ("EN_en.default.toml", "English",  "🇬🇧"),
    ("CH_fr.toml",          "Français", "🇨🇭"),
    ("CH_de.toml",          "Deutsch",  "🇨🇭"),
    ("CH_it.toml",          "Italiano", "🇨🇭"),
];

/// Returns true when the user clicks the continue button.
pub fn show(ui: &mut Ui, selected: &mut String, extra_languages: &[String], t: &Translations) -> bool {
    let mut proceed = false;

    let dark_mode = ui.visuals().dark_mode;
    let text_primary = if dark_mode {
        Color32::from_rgb(230, 230, 230)
    } else {
        Color32::from_rgb(20, 20, 20)
    };
    let text_secondary = if dark_mode {
        Color32::from_rgb(180, 180, 180)
    } else {
        Color32::from_rgb(80, 80, 80)
    };
    let btn_idle = if dark_mode {
        Color32::from_rgb(40, 40, 52)
    } else {
        Color32::from_rgb(215, 215, 228)
    };
    let btn_selected = Color32::from_rgb(40, 130, 40);

    ui.vertical_centered(|ui| {
        ui.add_space(18.0);
        ui.label(
            RichText::new(t.app_title)
                .size(22.0)
                .strong()
                .color(text_primary),
        );
        ui.add_space(5.0);
        ui.label(
            RichText::new(t.select_lang_subtitle)
                .size(13.0)
                .color(text_secondary),
        );
        ui.add_space(28.0);
    });

    let avail_w = ui.available_width();
    let btn_w = (avail_w - 48.0) / 2.0;
    let btn_h = 80.0;

    for chunk in BASE_LANGUAGES.chunks(2) {
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            for (code, label, flag) in chunk {
                let is_sel = selected.as_str() == *code;
                let fill = if is_sel { btn_selected } else { btn_idle };
                let txt_col = if is_sel { Color32::WHITE } else { text_primary };

                let content = format!("{}\n{}", flag, label);
                let btn = egui::Button::new(RichText::new(content).size(15.0).color(txt_col))
                    .fill(fill)
                    .min_size(egui::vec2(btn_w, btn_h));

                if ui.add(btn).clicked() {
                    *selected = code.to_string();
                }
                ui.add_space(8.0);
            }
        });
        ui.add_space(8.0);
    }

    // Additional languages from GitHub (exclude base languages already shown)
    let base_codes: Vec<&str> = BASE_LANGUAGES.iter().map(|(c, _, _)| *c).collect();
    let extras: Vec<&String> = extra_languages
        .iter()
        .filter(|l| !base_codes.contains(&l.as_str()))
        .collect();

    if !extras.is_empty() {
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(6.0);
        ui.label(
            RichText::new(t.other_languages)
                .size(11.0)
                .color(text_secondary),
        );
        ui.add_space(6.0);

        for chunk in extras.chunks(3) {
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                for code in chunk {
                    let is_sel = selected == *code;
                    let fill = if is_sel { btn_selected } else { btn_idle };
                    let txt_col = if is_sel { Color32::WHITE } else { text_primary };
                    let display = language_label(code);
                    let btn = egui::Button::new(
                        RichText::new(display).size(13.0).color(txt_col),
                    )
                    .fill(fill)
                    .min_size(egui::vec2(120.0, 36.0));

                    if ui.add(btn).clicked() {
                        *selected = code.to_string();
                    }
                    ui.add_space(6.0);
                }
            });
            ui.add_space(6.0);
        }
    }

    ui.add_space(20.0);
    ui.vertical_centered(|ui| {
        let continue_btn = egui::Button::new(
            RichText::new(t.continue_btn)
                .size(15.0)
                .color(Color32::WHITE)
                .strong(),
        )
        .fill(Color32::from_rgb(40, 130, 40))
        .min_size(egui::vec2(200.0, 40.0));

        if ui.add(continue_btn).clicked() {
            proceed = true;
        }
    });

    proceed
}

fn language_label(file_name: &str) -> String {
    file_name
        .strip_suffix(".default.toml")
        .or_else(|| file_name.strip_suffix(".toml"))
        .unwrap_or(file_name)
        .replace('_', "-")
}
