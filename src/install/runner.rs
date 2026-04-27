use crate::github::{self, GithubRelease, ReleaseAsset};
use crate::install::{certificates, paths, shortcuts};
use crate::state::{InstallLogEntry, InstallOptions, InstallStatus};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

pub type Log = Arc<Mutex<Vec<InstallLogEntry>>>;
type ProgramInstall = (String, Option<GithubRelease>, String, String, String); // (name, release, branch, language, lang_folder)

fn log(log: &Log, app: &str, status: InstallStatus) {
    eprintln!("[suite_install][install][INFO] {app}: {status:?}");
    let mut l = log.lock().unwrap();
    // Update existing entry or push
    if let Some(entry) = l.iter_mut().find(|entry| entry.app == app) {
        entry.status = status;
    } else {
        l.push(InstallLogEntry {
            app: app.to_string(),
            status,
            actions: Vec::new(),
        });
    }
}

fn action(log: &Log, app: &str, message: impl Into<String>) {
    let message = message.into();
    eprintln!("[suite_install][install][ACTION] {app}: {message}");
    let mut l = log.lock().unwrap();
    if let Some(entry) = l.iter_mut().find(|entry| entry.app == app) {
        entry.actions.push(message);
    } else {
        l.push(InstallLogEntry {
            app: app.to_string(),
            status: InstallStatus::Pending,
            actions: vec![message],
        });
    }
}

pub fn install_programs(programs: Vec<ProgramInstall>, options: InstallOptions, log_out: Log) {
    std::thread::spawn(move || {
        for (name, release, branch, language, lang_folder) in programs {
            eprintln!("[suite_install][install][INFO] Debut installation de {name} depuis la branche {branch}");
            action(
                &log_out,
                &name,
                format!("Démarrage de l'installation depuis la branche {branch}"),
            );
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
            action(
                &log_out,
                &name,
                format!(
                    "Création du dossier d'installation: {}",
                    install_dir.display()
                ),
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
            action(
                &log_out,
                &name,
                format!("Création du dossier de données: {}", appdata_dir.display()),
            );
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
            action(
                &log_out,
                &name,
                format!("Vérification du certificat: {cert_url}"),
            );
            if certificates::cert_exists(&cert_url) {
                log(
                    &log_out,
                    &name,
                    InstallStatus::Installing(format!("Certificat {name}")),
                );
                action(&log_out, &name, "Installation du certificat public");
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
            action(
                &log_out,
                &name,
                "Recherche et téléchargement de l'asset Windows",
            );
            let exe_path = match download_binary(&name, release.as_ref(), &install_dir, &log_out) {
                Ok(p) => p,
                Err(e) => {
                    log(&log_out, &name, InstallStatus::Error(e));
                    continue;
                }
            };

            // 4. Copy language files from repo
            action(
                &log_out,
                &name,
                format!("Copie de la langue sélectionnée: {language}"),
            );
            if let Err(e) = copy_lang_file(&name, &branch, &language, &lang_folder, &appdata_dir) {
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
                action(&log_out, &name, "Création du raccourci Bureau");
                if let Err(e) = shortcuts::create_desktop_shortcut(&name, &exe_path) {
                    eprintln!(
                        "[suite_install][install][ERROR] {name}: raccourci bureau impossible: {e}"
                    );
                }
            }
            if options.quicklaunch_shortcut {
                action(&log_out, &name, "Création du raccourci Menu Démarrer");
                if let Err(e) = shortcuts::create_start_menu_shortcut(&name, &exe_path) {
                    eprintln!("[suite_install][install][ERROR] {name}: raccourci menu demarrer impossible: {e}");
                }
            }

            // 6. Write install record
            action(&log_out, &name, "Écriture du fichier install.json");
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
            action(&log_out, &name, "Installation terminée");
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

    // Prefer windows-x64, then any .exe/.zip
    let asset = pick_windows_asset(&release.assets)
        .ok_or_else(|| format!("{name}: pas d'asset Windows trouvé dans la release"))?;

    // Look for a companion .sha256 asset
    let sha256_asset = release.assets.iter().find(|a| {
        a.name == format!("{}.sha256", asset.name)
            || a.name == format!("{}.sha256sum", asset.name)
    });

    eprintln!(
        "[suite_install][install][INFO] {name}: asset='{}' size={} sha256={}",
        asset.name,
        human_size(asset.size),
        sha256_asset.map(|a| a.name.as_str()).unwrap_or("absent")
    );
    action(log, name, format!("Asset: {} ({})", asset.name, human_size(asset.size)));

    let tmp_dir = paths::temp_dir(name);
    std::fs::create_dir_all(&tmp_dir).map_err(|e| format!("{name}: tmp dir: {e}"))?;
    let tmp_path = tmp_dir.join(&asset.name);

    {
        let mut l = log.lock().unwrap();
        if let Some(entry) = l.iter_mut().find(|e| e.app == name) {
            entry.status = InstallStatus::Downloading(
                format!("{} ({})", name, human_size(asset.size))
            );
        }
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent("suite_install/0.1")
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;

    // Download main binary
    let bytes = fetch_bytes(&client, &asset.browser_download_url, name)?;

    // Verify size
    let expected_size = asset.size;
    let actual_size = bytes.len() as u64;
    eprintln!("[suite_install][install][INFO] {name}: taille attendue={expected_size} recue={actual_size}");
    if expected_size > 0 && actual_size != expected_size {
        return Err(format!(
            "{name}: taille incorrecte (attendu {expected_size} o, recu {actual_size} o) — téléchargement corrompu"
        ));
    }
    action(log, name, format!("Taille vérifiée: {}", human_size(actual_size)));

    // Verify SHA-256 if a checksum file exists
    if let Some(sha_asset) = sha256_asset {
        action(log, name, "Vérification SHA-256…");
        let sha_bytes = fetch_bytes(&client, &sha_asset.browser_download_url, name)?;
        let expected_hex = String::from_utf8_lossy(&sha_bytes)
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_lowercase();

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let actual_hex = hex::encode(hasher.finalize());

        eprintln!("[suite_install][install][INFO] {name}: SHA-256 attendu={expected_hex} calcule={actual_hex}");
        if !expected_hex.is_empty() && actual_hex != expected_hex {
            return Err(format!(
                "{name}: SHA-256 invalide — fichier corrompu ou falsifié\n  attendu : {expected_hex}\n  calculé : {actual_hex}"
            ));
        }
        action(log, name, "SHA-256 validé ✓");
    } else {
        action(log, name, "Aucun fichier .sha256 fourni — vérification de taille uniquement");
    }

    // Write to tmp
    std::fs::write(&tmp_path, &bytes).map_err(|e| format!("{name}: écriture tmp: {e}"))?;

    // Extract or copy
    let exe_path = if asset.name.ends_with(".zip") {
        action(log, name, "Extraction de l'archive ZIP");
        extract_zip(&tmp_path, install_dir, name).map_err(|e| format!("{name}: zip: {e}"))?
    } else {
        let dest = install_dir.join(&asset.name);
        action(log, name, format!("Copie vers {}", dest.display()));
        std::fs::copy(&tmp_path, &dest).map_err(|e| format!("{name}: copie: {e}"))?;
        dest
    };

    Ok(exe_path)
}

fn pick_windows_asset(assets: &[ReleaseAsset]) -> Option<&ReleaseAsset> {
    // Priority: explicit windows-x64/amd64, then any .exe or .zip (skip checksums)
    let is_checksum = |n: &str| n.ends_with(".sha256") || n.ends_with(".sha256sum") || n.ends_with(".md5");
    assets.iter()
        .filter(|a| !is_checksum(&a.name))
        .max_by_key(|a| {
            let n = a.name.to_lowercase();
            let score =
                (if n.contains("x86_64") || n.contains("x64") || n.contains("amd64") { 4 } else { 0 })
              + (if n.contains("windows") || n.contains("win") { 2 } else { 0 })
              + (if n.ends_with(".exe") || n.ends_with(".zip") { 1 } else { 0 });
            score
        })
        .filter(|a| {
            let n = a.name.to_lowercase();
            n.ends_with(".exe") || n.ends_with(".zip") || n.contains("windows") || n.contains("win")
        })
}

fn fetch_bytes(client: &reqwest::blocking::Client, url: &str, name: &str) -> Result<Vec<u8>, String> {
    let resp = client.get(url).send()
        .map_err(|e| format!("{name}: connexion impossible ({url}): {e}"))?;
    let status = resp.status();
    eprintln!("[suite_install][install][INFO] {name}: GET {url} -> {status}");
    if !status.is_success() {
        return Err(format!("{name}: HTTP {status} pour {url}"));
    }
    resp.bytes()
        .map(|b| b.to_vec())
        .map_err(|e| format!("{name}: lecture corps ({url}): {e}"))
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
    lang_folder: &str,
    appdata_dir: &std::path::Path,
) -> anyhow::Result<()> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("suite_install/0.1")
        .build()?;

    // Try the requested language file, fall back to the .default.toml if not found
    let candidates: Vec<String> = {
        let mut v = vec![format!("{lang_folder}/{language}")];
        // If the requested file is not the default, also queue the default as fallback
        if !language.contains(".default.") {
            v.push(format!("{lang_folder}/EN_en.default.toml"));
        }
        v
    };

    for path in &candidates {
        let url = github::raw_url(name, branch, path);
        eprintln!("[suite_install][install][INFO] {name}: tentative langue {url}");
        let resp = client.get(&url).send()?;
        if resp.status().is_success() {
            let lang_dir = appdata_dir.join("lang");
            std::fs::create_dir_all(&lang_dir)?;
            let bytes = resp.bytes()?;
            let file_name = std::path::Path::new(path).file_name().unwrap_or_default();
            std::fs::write(lang_dir.join(file_name), &bytes)?;
            eprintln!(
                "[suite_install][install][INFO] {name}: langue '{}' copiee dans '{}'",
                file_name.to_string_lossy(),
                lang_dir.display()
            );
            return Ok(());
        }
        eprintln!("[suite_install][install][WARN] {name}: {path} absent (HTTP {}), essai suivant", resp.status());
    }

    anyhow::bail!("aucun fichier de langue disponible pour {name} (cherche: {})", candidates.join(", "))
}

pub fn uninstall_programs(names: Vec<String>, log_out: Log) {
    std::thread::spawn(move || {
        for name in names {
            eprintln!("[suite_install][uninstall][INFO] Debut desinstallation de {name}");
            action(&log_out, &name, format!("Démarrage de la désinstallation de {name}"));
            log(&log_out, &name, InstallStatus::Installing(format!("Désinstallation {name}")));

            let install_dir = paths::program_files_dir(&name);
            let appdata_dir = paths::appdata_dir(&name);
            let tmp_dir = paths::temp_dir(&name);

            // 1. Remove executable directory
            action(&log_out, &name, format!("Suppression de {}", install_dir.display()));
            if install_dir.exists() {
                match std::fs::remove_dir_all(&install_dir) {
                    Ok(_) => eprintln!("[suite_install][uninstall][INFO] {name}: dossier install supprime"),
                    Err(e) => {
                        eprintln!("[suite_install][uninstall][ERROR] {name}: suppression install impossible: {e}");
                        action(&log_out, &name, format!("⚠ Impossible de supprimer {}: {e}", install_dir.display()));
                    }
                }
            } else {
                action(&log_out, &name, format!("Dossier install absent (déjà supprimé): {}", install_dir.display()));
            }

            // 2. Remove shortcuts
            action(&log_out, &name, "Suppression des raccourcis");
            remove_shortcut_desktop(&name);
            remove_shortcut_start_menu(&name);

            // 3. Remove tmp
            action(&log_out, &name, format!("Suppression de {}", tmp_dir.display()));
            if tmp_dir.exists() {
                let _ = std::fs::remove_dir_all(&tmp_dir);
            }

            // 4. Remove AppData (last, contains install.json)
            action(&log_out, &name, format!("Suppression de {}", appdata_dir.display()));
            if appdata_dir.exists() {
                match std::fs::remove_dir_all(&appdata_dir) {
                    Ok(_) => eprintln!("[suite_install][uninstall][INFO] {name}: dossier appdata supprime"),
                    Err(e) => {
                        eprintln!("[suite_install][uninstall][ERROR] {name}: suppression appdata impossible: {e}");
                        action(&log_out, &name, format!("⚠ Impossible de supprimer AppData: {e}"));
                    }
                }
            }

            log(&log_out, &name, InstallStatus::Done(name.clone()));
            action(&log_out, &name, "Désinstallation terminée");
            eprintln!("[suite_install][uninstall][INFO] {name}: desinstallation terminee");
        }

        // Clean up empty rusty-suite root dirs
        cleanup_empty_roots();
    });
}

fn remove_shortcut_desktop(app_name: &str) {
    if let Some(desktop) = dirs::desktop_dir() {
        let lnk = desktop.join(format!("{}.lnk", app_name));
        if lnk.exists() {
            let _ = std::fs::remove_file(&lnk);
            eprintln!("[suite_install][uninstall][INFO] {app_name}: raccourci bureau supprime");
        }
    }
}

fn remove_shortcut_start_menu(app_name: &str) {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders",
        ) {
            if let Ok(programs_path) = hkcu.get_value::<String, _>("Programs") {
                let lnk = std::path::PathBuf::from(&programs_path)
                    .join("Rusty Suite")
                    .join(format!("{}.lnk", app_name));
                if lnk.exists() {
                    let _ = std::fs::remove_file(&lnk);
                    eprintln!("[suite_install][uninstall][INFO] {app_name}: raccourci menu supprime");
                }
            }
        }
    }
}

fn cleanup_empty_roots() {
    let base = dirs::data_dir().unwrap_or_default().join("rusty-suite");
    // Remove .tmp if empty
    let tmp = base.join(".tmp");
    if tmp.exists() {
        let is_empty = std::fs::read_dir(&tmp).map(|mut d| d.next().is_none()).unwrap_or(false);
        if is_empty {
            let _ = std::fs::remove_dir(&tmp);
        }
    }
    // Remove rusty-suite root if empty
    if base.exists() {
        let is_empty = std::fs::read_dir(&base).map(|mut d| d.next().is_none()).unwrap_or(false);
        if is_empty {
            let _ = std::fs::remove_dir(&base);
            eprintln!("[suite_install][uninstall][INFO] Dossier rusty-suite vide supprime");
        }
    }
    // Try to remove empty Program Files\rusty-suite
    let pf_base = paths::program_files_dir("__sentinel__")
        .parent().unwrap_or(std::path::Path::new("")).parent().unwrap_or(std::path::Path::new("")).to_path_buf();
    if pf_base.exists() {
        let is_empty = std::fs::read_dir(&pf_base).map(|mut d| d.next().is_none()).unwrap_or(false);
        if is_empty {
            let _ = std::fs::remove_dir(&pf_base);
        }
    }
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
