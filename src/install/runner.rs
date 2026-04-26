use std::sync::{Arc, Mutex};
use crate::install::{certificates, paths, shortcuts};
use crate::github::{self, GithubRelease};
use crate::state::{InstallOptions, InstallStatus};

pub type Log = Arc<Mutex<Vec<(String, InstallStatus)>>>;

fn log(log: &Log, app: &str, status: InstallStatus) {
    let mut l = log.lock().unwrap();
    // Update existing entry or push
    if let Some(entry) = l.iter_mut().find(|(n, _)| n == app) {
        entry.1 = status;
    } else {
        l.push((app.to_string(), status));
    }
}

pub fn install_programs(
    programs: Vec<(String, Option<GithubRelease>, String)>, // (name, release, branch)
    options: InstallOptions,
    log_out: Log,
) {
    std::thread::spawn(move || {
        for (name, release, branch) in programs {
            log(&log_out, &name, InstallStatus::Downloading(name.clone()));

            // 1. Prepare directories
            let install_dir = paths::program_files_dir(&name);
            let appdata_dir = paths::appdata_dir(&name);
            let tmp_dir = paths::temp_dir(&name);
            if let Err(e) = std::fs::create_dir_all(&install_dir) {
                log(&log_out, &name, InstallStatus::Error(format!("mkdir install: {e}")));
                continue;
            }
            if let Err(e) = std::fs::create_dir_all(&appdata_dir) {
                log(&log_out, &name, InstallStatus::Error(format!("mkdir appdata: {e}")));
                continue;
            }

            // 2. Install certificate if present
            let cert_url = github::certificate_url(&name, &branch);
            if certificates::cert_exists(&cert_url) {
                log(&log_out, &name, InstallStatus::Installing(format!("Certificat {name}")));
                if let Err(e) = certificates::install_certificate(&cert_url, &name, &tmp_dir) {
                    log(&log_out, &name, InstallStatus::Error(format!("Certificat: {e}")));
                    continue;
                }
            }

            // 3. Download binary from latest release
            let exe_path = match download_binary(&name, release.as_ref(), &install_dir, &log_out) {
                Ok(p) => p,
                Err(e) => {
                    log(&log_out, &name, InstallStatus::Error(e));
                    continue;
                }
            };

            // 4. Copy language files from repo
            let _ = copy_lang_files(&name, &branch, &appdata_dir);

            // 5. Shortcuts
            log(&log_out, &name, InstallStatus::Installing(format!("Raccourcis {name}")));
            if options.desktop_shortcut {
                let _ = shortcuts::create_desktop_shortcut(&name, &exe_path);
            }
            if options.quicklaunch_shortcut {
                let _ = shortcuts::create_start_menu_shortcut(&name, &exe_path);
            }

            // 6. Write install record
            let version = release
                .as_ref()
                .map(|r| r.tag_name.clone())
                .unwrap_or_else(|| "unknown".to_string());
            let record = paths::InstallRecord {
                version,
                exe_path: exe_path.to_string_lossy().to_string(),
                installed_at: chrono_now(),
            };
            let _ = paths::write_install_record(&name, &record);

            log(&log_out, &name, InstallStatus::Done(name.clone()));
        }
    });
}

fn download_binary(
    name: &str,
    release: Option<&GithubRelease>,
    install_dir: &std::path::Path,
    log: &Log,
) -> Result<std::path::PathBuf, String> {
    let release = release.ok_or_else(|| format!("{name}: aucune release disponible"))?;

    // Find a Windows x64 asset
    let asset = release
        .assets
        .iter()
        .find(|a| {
            let n = a.name.to_lowercase();
            n.contains("windows") || n.ends_with(".exe") || n.ends_with(".zip")
        })
        .ok_or_else(|| format!("{name}: pas d'asset Windows trouvé dans la release"))?;

    let tmp_path = paths::temp_dir(name).join(&asset.name);
    std::fs::create_dir_all(tmp_path.parent().unwrap())
        .map_err(|e| format!("{name}: tmp dir: {e}"))?;

    {
        let mut l = log.lock().unwrap();
        if let Some(entry) = l.iter_mut().find(|(n, _)| n == name) {
            entry.1 = InstallStatus::Downloading(format!("{} ({})", name, human_size(asset.size)));
        }
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent("suite_install/0.1")
        .build()
        .map_err(|e| e.to_string())?;
    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .and_then(|r| r.bytes())
        .map_err(|e| format!("{name}: download: {e}"))?;
    std::fs::write(&tmp_path, &bytes).map_err(|e| format!("{name}: write tmp: {e}"))?;

    // Extract zip or copy exe
    let exe_path = if asset.name.ends_with(".zip") {
        extract_zip(&tmp_path, install_dir, name)
            .map_err(|e| format!("{name}: zip: {e}"))?
    } else {
        let dest = install_dir.join(&asset.name);
        std::fs::copy(&tmp_path, &dest).map_err(|e| format!("{name}: copy: {e}"))?;
        dest
    };

    Ok(exe_path)
}

fn extract_zip(zip_path: &std::path::Path, dest_dir: &std::path::Path, app_name: &str) -> anyhow::Result<std::path::PathBuf> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut exe_path = dest_dir.join(format!("{}.exe", app_name));

    for i in 0..archive.len() {
        let mut zf = archive.by_index(i)?;
        let out = dest_dir.join(zf.name());
        if zf.name().ends_with('/') {
            std::fs::create_dir_all(&out)?;
        } else {
            if let Some(p) = out.parent() { std::fs::create_dir_all(p)?; }
            let mut outfile = std::fs::File::create(&out)?;
            std::io::copy(&mut zf, &mut outfile)?;
            if zf.name().ends_with(".exe") {
                exe_path = out.clone();
            }
        }
    }
    Ok(exe_path)
}

fn copy_lang_files(name: &str, branch: &str, appdata_dir: &std::path::Path) -> anyhow::Result<()> {
    // Download EN_en.default.toml from the app repo
    let url = github::raw_url(name, branch, "lang/EN_en.default.toml");
    let client = reqwest::blocking::Client::builder()
        .user_agent("suite_install/0.1")
        .build()?;
    let resp = client.get(&url).send()?;
    if resp.status().is_success() {
        let lang_dir = appdata_dir.join("lang");
        std::fs::create_dir_all(&lang_dir)?;
        let bytes = resp.bytes()?;
        std::fs::write(lang_dir.join("EN_en.default.toml"), &bytes)?;
    }
    Ok(())
}

fn human_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{bytes} B") }
    else if bytes < 1024 * 1024 { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else { format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0) }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    format!("{secs}")
}
