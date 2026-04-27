use crate::state::{InstallLogEntry, InstallStatus};
use egui::{Color32, RichText, ScrollArea, TextureHandle, Ui};
use image::AnimationDecoder;
use std::io::{BufReader, Cursor};
use std::sync::{Mutex, OnceLock};

const INSTALL_IMAGE: &[u8] = include_bytes!("../../assets/img/Rusty_suite_install_1.webp");

struct AnimFrame {
    handle: TextureHandle,
    delay_s: f64,
}

/// Cached GPU texture handles — initialised once on first render.
static ANIM_FRAMES: OnceLock<Mutex<Vec<AnimFrame>>> = OnceLock::new();

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
    // Initialise GPU handles once — decode + upload happen here on first call only.
    let mutex = ANIM_FRAMES.get_or_init(|| Mutex::new(load_anim_frames(ui.ctx())));
    let frames = mutex.lock().unwrap();

    if frames.is_empty() {
        return;
    }

    let total_s: f64 = frames.iter().map(|f| f.delay_s).sum();
    let time = ui.ctx().input(|i| i.time);
    let mut t = if total_s > 0.0 { time % total_s } else { 0.0 };

    let mut idx = frames.len() - 1;
    for (i, f) in frames.iter().enumerate() {
        if t < f.delay_s { idx = i; break; }
        t -= f.delay_s;
    }

    let next_frame_in = frames[idx].delay_s - t;
    ui.ctx().request_repaint_after(
        std::time::Duration::from_secs_f64(next_frame_in.max(0.016))
    );

    ui.add(
        egui::Image::from_texture(&frames[idx].handle)
            .max_height(150.0)
            .fit_to_original_size(1.0),
    );
}

fn load_anim_frames(ctx: &egui::Context) -> Vec<AnimFrame> {
    let cursor = Cursor::new(INSTALL_IMAGE);
    let decoder = match image::codecs::webp::WebPDecoder::new(BufReader::new(cursor)) {
        Ok(d) => d,
        Err(e) => { eprintln!("[anim] WebP decode: {e}"); return Vec::new(); }
    };
    let raw_frames = match decoder.into_frames().collect_frames() {
        Ok(f) => f,
        Err(e) => { eprintln!("[anim] frames: {e}"); return Vec::new(); }
    };

    raw_frames.into_iter().enumerate().map(|(i, frame)| {
        let (num, den) = frame.delay().numer_denom_ms();
        let delay_s = if num == 0 || den == 0 { 0.08 }
                      else { (num as f64 / den as f64 / 1000.0).max(0.016) };

        let buf  = frame.into_buffer();
        let size = [buf.width() as usize, buf.height() as usize];
        let img  = egui::ColorImage::from_rgba_unmultiplied(size, buf.as_raw());

        // Upload to GPU immediately — no clone needed at render time.
        let handle = ctx.load_texture(format!("anim_{i}"), img, egui::TextureOptions::LINEAR);
        AnimFrame { handle, delay_s }
    }).collect()
}
