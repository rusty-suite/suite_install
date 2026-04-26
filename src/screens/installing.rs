use crate::state::{InstallLogEntry, InstallStatus};
use egui::{Color32, RichText, ScrollArea, Ui};
use image::AnimationDecoder;
use std::io::{BufReader, Cursor};
use std::sync::OnceLock;

const INSTALL_IMAGE: &[u8] = include_bytes!("../../assets/img/Rusty_suite_install_1.webp");

struct InstallImageFrame {
    image: egui::ColorImage,
    delay_seconds: f64,
}

static INSTALL_FRAMES: OnceLock<Vec<InstallImageFrame>> = OnceLock::new();

pub fn show(ui: &mut Ui, log: &[InstallLogEntry], all_done: bool, is_uninstall: bool) -> bool {
    let mut close = false;

    ui.vertical_centered(|ui| {
        ui.add_space(6.0);
        show_install_image(ui);
        ui.add_space(8.0);
        let title = if all_done {
            if is_uninstall { "Désinstallation terminée ✓" } else { "Installation terminée ✓" }
        } else if is_uninstall {
            "Désinstallation en cours…"
        } else {
            "Installation en cours…"
        };
        ui.label(RichText::new(title).size(18.0).strong().color(if all_done {
            Color32::from_rgb(80, 210, 80)
        } else {
            Color32::WHITE
        }));
        ui.add_space(8.0);
    });

    let completed = log
        .iter()
        .filter(|entry| {
            matches!(
                entry.status,
                InstallStatus::Done(_) | InstallStatus::Error(_)
            )
        })
        .count();
    let total = log.len();
    let progress = if total == 0 {
        0.0
    } else {
        completed as f32 / total as f32
    };

    ui.add(
        egui::ProgressBar::new(progress)
            .desired_width(f32::INFINITY)
            .text(format!("{completed}/{total} programme(s) traités")),
    );
    ui.add_space(8.0);

    egui::CollapsingHeader::new("Actions exactes en cours")
        .default_open(!all_done)
        .show(ui, |ui| {
            let current_actions = log.iter().filter(|entry| {
                matches!(
                    entry.status,
                    InstallStatus::Downloading(_) | InstallStatus::Installing(_)
                )
            });

            let mut has_current = false;
            for entry in current_actions {
                has_current = true;
                ui.label(RichText::new(&entry.app).strong().color(Color32::WHITE));
                if let Some(action) = entry.actions.last() {
                    ui.label(
                        RichText::new(action)
                            .size(12.0)
                            .color(Color32::from_rgb(160, 190, 230)),
                    );
                }
                for action in entry.actions.iter().rev().skip(1).take(6) {
                    ui.label(
                        RichText::new(format!("• {action}"))
                            .size(11.0)
                            .color(Color32::from_rgb(140, 140, 140)),
                    );
                }
                ui.add_space(6.0);
            }

            if !has_current {
                ui.label(
                    RichText::new("Aucune action active.")
                        .size(12.0)
                        .color(Color32::from_rgb(140, 140, 140)),
                );
            }
        });
    ui.add_space(8.0);

    let avail_h = ui.available_size().y - 80.0;
    ScrollArea::vertical()
        .max_height(avail_h)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for entry in log {
                ui.horizontal(|ui| {
                    let (icon, color) = match &entry.status {
                        InstallStatus::Pending => ("○", Color32::GRAY),
                        InstallStatus::Downloading(_) => ("⬇", Color32::from_rgb(100, 180, 240)),
                        InstallStatus::Installing(_) => ("⚙", Color32::from_rgb(220, 160, 60)),
                        InstallStatus::Done(_) => ("✓", Color32::from_rgb(80, 210, 80)),
                        InstallStatus::Error(_) => ("✗", Color32::from_rgb(220, 80, 80)),
                    };
                    ui.label(RichText::new(icon).color(color).monospace());

                    let msg = match &entry.status {
                        InstallStatus::Pending => entry.app.clone(),
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

fn show_install_image(ui: &mut Ui) {
    let frames = INSTALL_FRAMES.get_or_init(decode_install_frames);
    if frames.is_empty() {
        return;
    }
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(40));

    let total_duration: f64 = frames.iter().map(|frame| frame.delay_seconds).sum();
    let time = ui.ctx().input(|input| input.time);
    let mut elapsed = if total_duration > 0.0 {
        time % total_duration
    } else {
        0.0
    };

    let mut frame_index = 0;
    for (index, frame) in frames.iter().enumerate() {
        if elapsed <= frame.delay_seconds {
            frame_index = index;
            break;
        }
        elapsed -= frame.delay_seconds;
    }

    let frame = &frames[frame_index];
    let texture = ui.ctx().load_texture(
        format!("rusty_suite_install_frame_{frame_index}"),
        frame.image.clone(),
        egui::TextureOptions::LINEAR,
    );

    ui.add(
        egui::Image::from_texture(&texture)
            .max_height(150.0)
            .fit_to_original_size(1.0),
    );
}

fn decode_install_frames() -> Vec<InstallImageFrame> {
    let cursor = Cursor::new(INSTALL_IMAGE);
    let reader = BufReader::new(cursor);
    let decoder = match image::codecs::webp::WebPDecoder::new(reader) {
        Ok(decoder) => decoder,
        Err(err) => {
            eprintln!("[suite_install][install_ui][ERROR] WebP decode impossible: {err}");
            return Vec::new();
        }
    };

    match decoder.into_frames().collect_frames() {
        Ok(frames) => frames.into_iter().map(frame_to_install_image).collect(),
        Err(err) => {
            eprintln!("[suite_install][install_ui][ERROR] Frames WebP impossibles: {err}");
            Vec::new()
        }
    }
}

fn frame_to_install_image(frame: image::Frame) -> InstallImageFrame {
    let (numerator, denominator) = frame.delay().numer_denom_ms();
    let delay_seconds = if numerator == 0 || denominator == 0 {
        0.08
    } else {
        (numerator as f64 / denominator as f64 / 1000.0).max(0.02)
    };

    let buffer = frame.into_buffer();
    let size = [buffer.width() as usize, buffer.height() as usize];
    let image = egui::ColorImage::from_rgba_unmultiplied(size, buffer.as_raw());

    InstallImageFrame {
        image,
        delay_seconds,
    }
}
