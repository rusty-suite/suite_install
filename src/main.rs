// Rusty Suite Installer — main entry point
#![windows_subsystem = "windows"]

mod github;
mod install;
mod screens;
mod state;

use eframe::egui::{
    self, Color32, FontData, FontDefinitions, FontFamily, RichText, ViewportBuilder,
};
use install::{paths, runner};
use state::{AppState, InstallStatus, ProgramEntry, Screen};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("Rusty Suite — Installeur")
            .with_inner_size([760.0, 580.0])
            .with_min_inner_size([640.0, 480.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(
                    include_bytes!("../assets/img/Suite-install-1.png"),
                )
                .unwrap_or_default(),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "Rusty Suite Installer",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            setup_visuals(&cc.egui_ctx);
            Ok(Box::new(InstallerApp::new()))
        }),
    )
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    for (name, path) in windows_font_candidates() {
        match std::fs::read(&path) {
            Ok(bytes) => {
                fonts
                    .font_data
                    .insert(name.to_string(), FontData::from_owned(bytes));

                for family in [FontFamily::Proportional, FontFamily::Monospace] {
                    fonts
                        .families
                        .entry(family)
                        .or_default()
                        .push(name.to_string());
                }

                eprintln!("[suite_install][fonts][INFO] Police chargee: {name} ({path})");
            }
            Err(err) => {
                eprintln!(
                    "[suite_install][fonts][WARN] Police indisponible: {name} ({path}): {err}"
                );
            }
        }
    }

    ctx.set_fonts(fonts);
}

fn windows_font_candidates() -> Vec<(&'static str, String)> {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
    let fonts_dir = std::path::PathBuf::from(windir).join("Fonts");

    [
        ("segoe-ui", "segoeui.ttf"),
        ("segoe-ui-symbol", "seguisym.ttf"),
        ("segoe-ui-emoji", "seguiemj.ttf"),
    ]
    .into_iter()
    .map(|(name, file)| (name, fonts_dir.join(file).to_string_lossy().to_string()))
    .collect()
}

fn setup_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(Color32::from_rgb(230, 230, 230));
    visuals.window_fill = Color32::from_rgb(22, 22, 28);
    visuals.panel_fill = Color32::from_rgb(22, 22, 28);
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(30, 30, 38);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(40, 40, 50);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(55, 55, 68);
    visuals.widgets.active.bg_fill = Color32::from_rgb(40, 130, 40);
    ctx.set_visuals(visuals);
}

struct InstallerApp {
    state: AppState,
    log: runner::Log,
    load_handle: Option<std::thread::JoinHandle<Result<LoadedPrograms, String>>>,
}

struct LoadedPrograms {
    programs: Vec<ProgramEntry>,
    common_languages: Vec<String>,
}

impl InstallerApp {
    fn new() -> Self {
        let log: runner::Log = Arc::new(Mutex::new(Vec::new()));
        let mut app = Self {
            state: AppState::default(),
            log,
            load_handle: None,
        };
        app.start_loading();
        app
    }

    fn start_loading(&mut self) {
        self.state.screen = Screen::Loading;
        let handle = std::thread::spawn(|| load_programs());
        self.load_handle = Some(handle);
    }

    fn poll_loading(&mut self) {
        if self
            .load_handle
            .as_ref()
            .map(|h| h.is_finished())
            .unwrap_or(false)
        {
            if let Some(handle) = self.load_handle.take() {
                match handle.join().unwrap_or(Err("Thread paniqué".to_string())) {
                    Ok(loaded) => {
                        self.state.programs = loaded.programs;
                        self.state.common_languages = loaded.common_languages;
                        if let Some(language) = self.state.common_languages.first() {
                            self.state.install_options.selected_language = language.clone();
                        }
                        self.state.screen = Screen::Eula;
                    }
                    Err(e) => {
                        self.state.loading_error = Some(e);
                        self.state.screen = Screen::Loading;
                    }
                }
            }
        }
    }

    fn start_installation(&mut self) {
        self.state.screen = Screen::Installing;
        self.state.is_uninstall = false;

        let to_install: Vec<_> = self
            .state
            .programs
            .iter()
            .filter(|p| p.selected)
            .map(|p| {
                (
                    p.repo.name.clone(),
                    p.release.clone(),
                    p.repo.default_branch.clone(),
                    self.state.install_options.selected_language.clone(),
                    p.lang_folder.clone(),
                )
            })
            .collect();

        {
            let mut l = self.log.lock().unwrap();
            l.clear();
            for (name, _, _, _, _) in &to_install {
                l.push(state::InstallLogEntry {
                    app: name.clone(),
                    status: InstallStatus::Pending,
                    actions: Vec::new(),
                });
            }
        }

        let log_clone = Arc::clone(&self.log);
        runner::install_programs(to_install, self.state.install_options.clone(), log_clone);
    }

    fn start_uninstallation(&mut self) {
        self.state.screen = Screen::Installing;
        self.state.is_uninstall = true;

        let to_uninstall: Vec<String> = self
            .state
            .programs
            .iter()
            .filter(|p| p.selected && p.installed_version.is_some())
            .map(|p| p.repo.name.clone())
            .collect();

        {
            let mut l = self.log.lock().unwrap();
            l.clear();
            for name in &to_uninstall {
                l.push(state::InstallLogEntry {
                    app: name.clone(),
                    status: InstallStatus::Pending,
                    actions: Vec::new(),
                });
            }
        }

        let log_clone = Arc::clone(&self.log);
        runner::uninstall_programs(to_uninstall, log_clone);
    }
}

impl eframe::App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_loading();

        if matches!(self.state.screen, Screen::Loading | Screen::Installing) {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.state.screen.clone() {
            Screen::Loading => {
                show_loading(ui, &self.state.loading_error);
            }
            Screen::Eula => {
                if screens::eula::show(ui, &mut self.state.eula_accepted) {
                    self.state.screen = Screen::ProgramList;
                }
            }
            Screen::ProgramList => {
                let (do_install, do_uninstall) = screens::program_list::show(ui, &mut self.state);
                if do_install {
                    self.start_installation();
                } else if do_uninstall {
                    self.start_uninstallation();
                }
            }
            Screen::Installing => {
                let log = self.log.lock().unwrap().clone();
                let all_done = !log.is_empty()
                    && log.iter().all(|entry| {
                        matches!(
                            entry.status,
                            InstallStatus::Done(_) | InstallStatus::Error(_)
                        )
                    });
                if screens::installing::show(ui, &log, all_done, self.state.is_uninstall) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            Screen::Done => {}
        });
    }
}

fn show_loading(ui: &mut egui::Ui, error: &Option<String>) {
    ui.vertical_centered(|ui| {
        ui.add_space(80.0);
        if let Some(err) = error {
            ui.label(
                RichText::new(format!("Erreur : {}", err))
                    .color(Color32::from_rgb(220, 80, 80))
                    .size(15.0),
            );
        } else {
            ui.label(
                RichText::new("Chargement de la liste des programmes…")
                    .size(16.0)
                    .color(Color32::from_rgb(160, 160, 160)),
            );
            ui.add_space(16.0);
            ui.spinner();
        }
    });
}

fn load_programs() -> Result<LoadedPrograms, String> {
    let repos = github::fetch_org_repos().map_err(|e| e.to_string())?;

    let mut programs = Vec::new();
    let mut all_languages: BTreeSet<String> = BTreeSet::new();

    for repo in repos {
        let release = github::fetch_latest_release(&repo.name).ok().flatten();
        let (languages, lang_folder) = github::fetch_language_files(&repo.name, &repo.default_branch)
            .unwrap_or_else(|e| {
                eprintln!("[suite_install][load][WARN] {}: fetch langue impossible: {e}", repo.name);
                (Vec::new(), "langue".to_string())
            });

        // Union: accumulate every language found across all repos
        for lang in &languages {
            all_languages.insert(lang.clone());
        }

        let installed = paths::read_install_record(&repo.name);
        let installed_version = installed.as_ref().map(|r| r.version.clone());
        let needs_update = match (&installed_version, &release) {
            (Some(iv), Some(rel)) => iv != &rel.tag_name,
            _ => false,
        };

        programs.push(ProgramEntry {
            repo,
            release,
            languages,
            lang_folder,
            selected: true,
            installed_version,
            needs_update,
        });
    }

    // Sort: default language first, then alphabetical
    let mut common_languages: Vec<String> = all_languages.into_iter().collect();
    common_languages.sort_by(|a, b| {
        let a_default = a.contains(".default.");
        let b_default = b.contains(".default.");
        b_default.cmp(&a_default).then(a.cmp(b))
    });

    Ok(LoadedPrograms {
        programs,
        common_languages,
    })
}
