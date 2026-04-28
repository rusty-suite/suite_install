use crate::i18n::Translations;
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

/// Call this early (e.g. during the Loading screen) so the WebP decoding
/// happens on the UI thread before the user ever clicks Install.
pub fn preload_frames(ctx: &egui::Context) {
    ANIM_FRAMES.get_or_init(|| Mutex::new(load_anim_frames(ctx)));
}

pub fn show(
    ui: &mut Ui,
    log: &[InstallLogEntry],
    all_done: bool,
    is_uninstall: bool,
    t: &Translations,
) -> bool {
    let mut close = false;

    ui.vertical_centered(|ui| {
        ui.add_space(6.0);
        show_install_image(ui);
        ui.add_space(8.0);
        let title = if all_done {
            if is_uninstall { t.uninstall_done } else { t.install_done }
        } else if is_uninstall {
            t.uninstalling
        } else {
            t.installing
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
        .filter(|entry| matches!(entry.status, InstallStatus::Done(_) | InstallStatus::Error(_)))
        .count();
    let total = log.len();

    let total_bytes: u64 = log.iter().map(|e| e.bytes_total).sum();
    let done_bytes: u64  = log.iter().map(|e| e.bytes_done).sum();
    let progress = if total_bytes > 0 {
        (done_bytes as f32 / total_bytes as f32).clamp(0.0, 1.0)
    } else if total == 0 {
        0.0
    } else {
        completed as f32 / total as f32
    };

    let progress_label = t.n_programs_done
        .replace("{n}",     &completed.to_string())
        .replace("{total}", &total.to_string());

    ui.add(
        egui::ProgressBar::new(progress)
            .desired_width(f32::INFINITY)
            .text(progress_label),
    );
    ui.add_space(8.0);

    egui::CollapsingHeader::new(t.active_actions_header)
        .default_open(!all_done)
        .show(ui, |ui| {
            let current_actions = log.iter().filter(|entry| {
                matches!(entry.status, InstallStatus::Downloading(_) | InstallStatus::Installing(_))
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
                    RichText::new(t.no_active_action)
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
                        InstallStatus::Pending        => ("○", Color32::GRAY),
                        InstallStatus::Downloading(_) => ("⬇", Color32::from_rgb(100, 180, 240)),
                        InstallStatus::Installing(_)  => ("⚙", Color32::from_rgb(220, 160, 60)),
                        InstallStatus::Done(_)        => ("✓", Color32::from_rgb(80, 210, 80)),
                        InstallStatus::Error(_)       => ("✗", Color32::from_rgb(220, 80, 80)),
                    };
                    ui.label(RichText::new(icon).color(color).monospace());

                    let msg = match &entry.status {
                        InstallStatus::Pending => entry.app.clone(),
                        InstallStatus::Downloading(s) | InstallStatus::Installing(s) => s.clone(),
                        InstallStatus::Done(s)  => format!("{} — OK", s),
                        InstallStatus::Error(e) => t.status_error.replace("{msg}", e),
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
            let close_btn = egui::Button::new(
                RichText::new(t.close_btn).color(Color32::WHITE).strong(),
            )
            .fill(Color32::from_rgb(50, 50, 65))
            .min_size(egui::vec2(100.0, 28.0));
            if ui.add(close_btn).clicked() {
                eprintln!("[suite_install][installing][INFO] Fermeture demandée par l'utilisateur");
                close = true;
            }
        });
    }

    close
}

fn show_install_image(ui: &mut Ui) {
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

    if raw_frames.is_empty() { return Vec::new(); }

    // Canvas dimensions from the first frame.
    let canvas_w = raw_frames[0].buffer().width();
    let canvas_h = raw_frames[0].buffer().height();

    // Pre-composite delta-encoded WebP frames onto a persistent canvas so each
    // AnimFrame texture always contains the full, correct image for that moment.
    let mut canvas = image::RgbaImage::new(canvas_w, canvas_h);

    raw_frames.into_iter().enumerate().map(|(i, frame)| {
        let (num, den) = frame.delay().numer_denom_ms();
        let delay_s = if num == 0 || den == 0 { 0.08 }
                      else { (num as f64 / den as f64 / 1000.0).max(0.016) };

        // Frame offset within the canvas (WebP supports partial-frame updates).
        let left = frame.left();
        let top  = frame.top();
        let buf  = frame.into_buffer();

        for (fx, fy, src) in buf.enumerate_pixels() {
            let cx = left + fx;
            let cy = top  + fy;
            if cx >= canvas_w || cy >= canvas_h { continue; }
            let sa = src[3];
            if sa == 0 { continue; } // transparent pixel — keep canvas
            if sa == 255 {
                canvas.put_pixel(cx, cy, *src);
            } else {
                // src-over alpha compositing for semi-transparent pixels
                let dst   = canvas.get_pixel(cx, cy);
                let sa_f  = sa as f32 / 255.0;
                let da_f  = dst[3] as f32 / 255.0;
                let out_a = sa_f + da_f * (1.0 - sa_f);
                let blend = |s: u8, d: u8| -> u8 {
                    if out_a < 0.001 { return 0; }
                    ((s as f32 * sa_f + d as f32 * da_f * (1.0 - sa_f)) / out_a) as u8
                };
                canvas.put_pixel(cx, cy, image::Rgba([
                    blend(src[0], dst[0]),
                    blend(src[1], dst[1]),
                    blend(src[2], dst[2]),
                    (out_a * 255.0).min(255.0) as u8,
                ]));
            }
        }

        let size = [canvas_w as usize, canvas_h as usize];
        let img  = egui::ColorImage::from_rgba_unmultiplied(size, canvas.as_raw());
        let handle = ctx.load_texture(format!("anim_{i}"), img, egui::TextureOptions::LINEAR);
        AnimFrame { handle, delay_s }
    }).collect()
}
