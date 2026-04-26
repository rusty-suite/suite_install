use crate::github::{GithubRelease, GithubRepo};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Loading,
    Eula,
    ProgramList,
    Installing,
    Done,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListMode {
    Install,
    Uninstall,
}

#[derive(Debug, Clone)]
pub struct ProgramEntry {
    pub repo: GithubRepo,
    pub release: Option<GithubRelease>,
    pub languages: Vec<String>,
    /// "langue" or "lang" — whichever folder was found in the repo
    pub lang_folder: String,
    pub selected: bool,
    pub installed_version: Option<String>,
    pub needs_update: bool,
}

#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub desktop_shortcut: bool,
    pub quicklaunch_shortcut: bool,
    pub selected_language: String,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            desktop_shortcut: true,
            quicklaunch_shortcut: false,
            selected_language: "EN_en.default.toml".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstallLogEntry {
    pub app: String,
    pub status: InstallStatus,
    pub actions: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum InstallStatus {
    Pending,
    Downloading(String),
    Installing(String),
    Done(String),
    Error(String),
}

pub struct AppState {
    pub screen: Screen,
    pub list_mode: ListMode,
    pub eula_accepted: bool,
    pub programs: Vec<ProgramEntry>,
    pub common_languages: Vec<String>,
    pub install_options: InstallOptions,
    pub loading_error: Option<String>,
    /// true when the active operation is an uninstall (vs install)
    pub is_uninstall: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::Loading,
            list_mode: ListMode::Install,
            eula_accepted: false,
            programs: Vec::new(),
            common_languages: Vec::new(),
            install_options: InstallOptions::default(),
            loading_error: None,
            is_uninstall: false,
        }
    }
}
