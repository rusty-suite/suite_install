use egui::{Color32, RichText, ScrollArea, Ui};
use crate::state::{InstallStatus};

pub fn show(ui: &mut Ui, log: &[(String, InstallStatus)], all_done: bool) -> bool {
    let mut close = false;

    ui.vertical_centered(|ui| {
        ui.add_space(8.0);
        let title = if all_done { "Installation terminée ✓" } else { "Installation en cours…" };
        ui.label(
            RichText::new(title)
                .size(18.0)
                .strong()
                .color(if all_done { Color32::from_rgb(80, 210, 80) } else { Color32::WHITE }),
        );
        ui.add_space(10.0);
    });

    let avail_h = ui.available_size().y - 80.0;
    ScrollArea::vertical()
        .max_height(avail_h)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for (name, status) in log {
                ui.horizontal(|ui| {
                    let (icon, color) = match status {
                        InstallStatus::Pending => ("○", Color32::GRAY),
                        InstallStatus::Downloading(_) => ("⬇", Color32::from_rgb(100, 180, 240)),
                        InstallStatus::Installing(_) => ("⚙", Color32::from_rgb(220, 160, 60)),
                        InstallStatus::Done(_) => ("✓", Color32::from_rgb(80, 210, 80)),
                        InstallStatus::Error(_) => ("✗", Color32::from_rgb(220, 80, 80)),
                    };
                    ui.label(RichText::new(icon).color(color).monospace());

                    let msg = match status {
                        InstallStatus::Pending => name.clone(),
                        InstallStatus::Downloading(s) | InstallStatus::Installing(s) => s.clone(),
                        InstallStatus::Done(s) => format!("{} — OK", s),
                        InstallStatus::Error(e) => format!("ERREUR: {}", e),
                    };
                    ui.label(RichText::new(msg).color(color));
                });
            }
        });

    if all_done {
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui
                .button(RichText::new("Fermer").color(Color32::WHITE).strong())
                .clicked()
            {
                close = true;
            }
        });
    }

    close
}
