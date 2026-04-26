use crate::github::{self, GithubRelease};
use crate::install::{certificates, paths, shortcuts};
use crate::state::{InstallOptions, InstallStatus};
use std::sync::{Arc, Mutex};

pub type Log = Arc<Mutex<Vec<(String, InstallStatus)>>>;
type ProgramInstall = (String, Option<GithubRelease>, String, String); // (name, release, branch, language)

fn log(log: &Log, app: &str, status: InstallStatus) {
    eprintln!("[suite_install][install][INFO] {app}: {status:?}");
    let mut l = log.lock().unwrap();
    // Update existing entry or push
    if let Some(entry) = l.iter_mut().find(|(n, _)| n == app) {
        entry.1 = status;
    } else {
        l.push((app.to_string(), status));
    }
}

pub fn install_programs(programs: Vec<ProgramInstall>, options: InstallOptions, log_out: Log) {
    std::thread::spawn(move || {
        for (name, release, branch, language) in programs {
            eprintln!("[suite_install][install][INFO] Debut installation de {name} depuis la branche {branch}");
            log(&log_out, &name, InstallStatus::Downloading(name.clone()));

            // 1. Prepare directories
            let install_dir = paths::program_files_dir(&name);
            let appdata_dir = paths::appdata_dir(&name);
            let tmp_dir = paths::temp_dir(&name);
            eprintln!(
                "[suite_install][install][INFO] {name}: dossiers install='{}', appdata='{}', tmp='{}'",
                install_dir.display(),
                appdata_dir.display(),
                tmp_dir.display()
            );
            if let Err(e) = std::fs::create_dir_all(&install_dir) {
                eprintln!("[suite_install][install][ERROR] {name}: creation dossier install impossible: {e}");
                log(
                    &log_out,
                    &name,
                    InstallStatus::Error(format!("mkdir install: {e}")),
                );
                continue;
            }
            if let Err(e) = std::fs::create_dir_all(&appdata_dir) {
                eprintln!("[suite_install][install][ERROR] {name}: creation dossier appdata impossible: {e}");
                log(
                    &log_out,
                    &name,
                    InstallStatus::Error(format!("mkdir appdata: {e}")),
                );
                continue;
            }

            // 2. Install certificate if present
            let cert_url = github::certificate_url(&name, &branch);
            if certificates::cert_exists(&cert_url) {
                log(
                    &log_out,
                    &name,
                    InstallStatus::Installing(format!("Certificat {name}")),
                );
                if let Err(e) = certificates::install_certificate(&cert_url, &name, &tmp_dir) {
                    log(
                        &log_out,
                        &name,
                        InstallStatus::Error(format!("Certificat: {e}")),
                    );
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
            if let Err(e) = copy_lang_file(&name, &branch, &language, &appdata_dir) {
                eprintln!(
                    "[suite_install][install][ERROR] {name}: copie langue '{language}' impossible: {e}"
                );
            }

            // 5. Shortcuts
            log(
                &log_out,
                &name,
                InstallStatus::Installing(format!("Raccourcis {name}")),
            );
            if options.desktop_shortcut {
                if let Err(e) = shortcuts::create_desktop_shortcut(&name, &exe_path) {
                    eprintln!(
                        "[suite_install][install][ERROR] {name}: raccourci bureau impossible: {e}"
                    );
                }
            }
            if options.quicklaunch_shortcut {
                if let Err(e) = shortcuts::create_start_menu_shortcut(&name, &exe_path) {
                    eprintln!("[suite_install][install][ERROR] {name}: raccourci menu demarrer impossible: {e}");
                }
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
            if let Err(e) = paths::write_install_record(&name, &record) {
                eprintln!(
                    "[suite_install][install][ERROR] {name}: ecriture install.json impossible: {e}"
                );
            }

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
    eprintln!(
        "[suite_install][install][INFO] {name}: asset selectionne '{}' ({}, {})",
        asset.name,
        human_size(asset.size),
        asset.browser_download_url
    );

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
    let response = client
        .get(&asset.browser_download_url)
        .send()
        .map_err(|e| format!("{name}: download send: {e}"))?;
    let status = response.status();
    eprintln!(
        "[suite_install][install][INFO] {name}: telechargement '{}' -> {status}",
        asset.browser_download_url
    );
    if !status.is_success() {
        return Err(format!("{name}: download HTTP {status}"));
    }
    let bytes = response
        .bytes()
        .map_err(|e| format!("{name}: download body: {e}"))?;
    std::fs::write(&tmp_path, &bytes).map_err(|e| format!("{name}: write tmp: {e}"))?;
    eprintln!(
        "[suite_install][install][INFO] {name}: {} octet(s) ecrits dans '{}'",
        bytes.len(),
        tmp_path.display()
    );

    // Extract zip or copy exe
    let exe_path = if asset.name.ends_with(".zip") {
        extract_zip(&tmp_path, install_dir, name).map_err(|e| format!("{name}: zip: {e}"))?
    } else {
        let dest = install_dir.join(&asset.name);
        std::fs::copy(&tmp_path, &dest).map_err(|e| format!("{name}: copy: {e}"))?;
        dest
    };

    Ok(exe_path)
}

fn extract_zip(
    zip_path: &std::path::Path,
    dest_dir: &std::path::Path,
    app_name: &str,
) -> anyhow::Result<std::path::PathBuf> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut exe_path = dest_dir.join(format!("{}.exe", app_name));

    for i in 0..archive.len() {
        let mut zf = archive.by_index(i)?;
        let out = dest_dir.join(zf.name());
        if zf.name().ends_with('/') {
            std::fs::create_dir_all(&out)?;
        } else {
            if let Some(p) = out.parent() {
                std::fs::create_dir_all(p)?;
            }
            let mut outfile = std::fs::File::create(&out)?;
            std::io::copy(&mut zf, &mut outfile)?;
            if zf.name().ends_with(".exe") {
                exe_path = out.clone();
            }
        }
    }
    Ok(exe_path)
}

fn copy_lang_file(
    name: &str,
    branch: &str,
    language: &str,
    appdata_dir: &std::path::Path,
) -> anyhow::Result<()> {
    let url = github::raw_url(name, branch, &format!("lang/{language}"));
    eprintln!("[suite_install][install][INFO] {name}: verification langue {url}");
    let client = reqwest::blocking::Client::builder()
        .user_agent("suite_install/0.1")
        .build()?;
    let resp = client.get(&url).send()?;
    if resp.status().is_success() {
        let lang_dir = appdata_dir.join("lang");
        std::fs::create_dir_all(&lang_dir)?;
        let bytes = resp.bytes()?;
        std::fs::write(lang_dir.join(language), &bytes)?;
        eprintln!(
            "[suite_install][install][INFO] {name}: fichier langue '{language}' copie dans '{}'",
            lang_dir.display()
        );
    } else {
        anyhow::bail!("lang/{language}: HTTP {}", resp.status());
    }
    Ok(())
}

fn human_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0)
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{secs}")
}
