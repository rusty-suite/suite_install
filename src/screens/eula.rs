use crate::i18n::Translations;
use egui::{Color32, RichText, ScrollArea, Ui};

pub fn show(ui: &mut Ui, accepted: &mut bool, t: &Translations) -> bool {
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
    let btn_disabled_fill = if dark_mode {
        Color32::from_rgb(60, 60, 60)
    } else {
        Color32::from_rgb(180, 180, 180)
    };

    ui.vertical_centered(|ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new(t.app_title)
                .size(22.0)
                .strong()
                .color(text_primary),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(t.eula_title)
                .size(14.0)
                .color(text_secondary),
        );
        ui.add_space(12.0);
    });

    let avail = ui.available_size();
    ScrollArea::vertical()
        .max_height(avail.y - 110.0)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut t.eula_text.to_string())
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
            RichText::new(t.eula_accept_label).color(text_primary),
        );
    });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let btn = egui::Button::new(
            RichText::new(t.accept_btn)
                .color(Color32::WHITE)
                .strong(),
        )
        .fill(if *accepted {
            Color32::from_rgb(40, 140, 40)
        } else {
            btn_disabled_fill
        });

        if ui.add_enabled(*accepted, btn).clicked() {
            proceed = true;
        }

        ui.add_space(16.0);
        if ui
            .button(RichText::new(t.cancel_btn).color(Color32::from_rgb(220, 80, 80)))
            .clicked()
        {
            std::process::exit(0);
        }
    });

    proceed
}
