use crate::github::{self, GithubRelease, ReleaseAsset};
use crate::install::{certificates, paths, shortcuts};
use crate::state::{InstallLogEntry, InstallOptions, InstallStatus};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

pub type Log = Arc<Mutex<Vec<InstallLogEntry>>>;
type ProgramInstall = (String, Option<GithubRelease>, String, String, String);
// (name, release, branch, language_file, lang_folder)

// ── Log helpers ───────────────────────────────────────────────────────────────

/// Update the status of a program entry in the shared log and write to file.
fn set_status(log: &Log, app: &str, status: InstallStatus) {
    crate::logger::write("runner", "STATUS", &format!("{app}: {status:?}"));
    let mut l = log.lock().unwrap();
    if let Some(entry) = l.iter_mut().find(|e| e.app == app) {
        entry.status = status;
    } else {
        l.push(InstallLogEntry { app: app.to_string(), status, actions: Vec::new() });
    }
}

/// Append an action message to a program entry and write to file.
fn action(log: &Log, app: &str, message: impl Into<String>) {
    let message = message.into();
    crate::logger::write("runner", "ACTION", &format!("{app}: {message}"));
    let mut l = log.lock().unwrap();
    if let Some(entry) = l.iter_mut().find(|e| e.app == app) {
        entry.actions.push(message);
    } else {
        l.push(InstallLogEntry {
            app: app.to_string(),
            status: InstallStatus::Pending,
            actions: vec![message],
        });
    }
}

// ── Install ───────────────────────────────────────────────────────────────────

pub fn install_programs(
    programs: Vec<ProgramInstall>,
    options: InstallOptions,
    log_out: Log,
    lang: String,
) {
    std::thread::spawn(move || {
        let t = crate::i18n::get(&lang);
        crate::logger::write("runner", "INFO", &format!(
            "=== Installation démarrée — {} programme(s) — langue: {lang} ===", programs.len()
        ));

        run_precheck(&programs, &log_out, t);

        for (name, release, branch, language, lang_folder) in programs {
            // ── Start ─────────────────────────────────────────────────────────
            action(&log_out, &name,
                t.log_starting_install.replace("{branch}", &branch));
            set_status(&log_out, &name, InstallStatus::Downloading(name.clone()));

            // ── 1. Prepare directories ────────────────────────────────────────
            let install_dir = paths::program_files_dir(&name);
            let appdata_dir = paths::appdata_dir(&name);
            let tmp_dir     = paths::temp_dir(&name);

            crate::logger::write("runner", "INFO", &format!(
                "{name}: install='{}' appdata='{}' tmp='{}'",
                install_dir.display(), appdata_dir.display(), tmp_dir.display()
            ));
            action(&log_out, &name,
                t.log_creating_install_dir.replace("{path}", &install_dir.display().to_string()));
            if let Err(e) = std::fs::create_dir_all(&install_dir) {
                crate::logger::write("runner", "ERROR", &format!("{name}: mkdir install: {e}"));
                set_status(&log_out, &name, InstallStatus::Error(format!("mkdir install: {e}")));
                continue;
            }

            action(&log_out, &name,
                t.log_creating_data_dir.replace("{path}", &appdata_dir.display().to_string()));
            if let Err(e) = std::fs::create_dir_all(&appdata_dir) {
                crate::logger::write("runner", "ERROR", &format!("{name}: mkdir appdata: {e}"));
                set_status(&log_out, &name, InstallStatus::Error(format!("mkdir appdata: {e}")));
                continue;
            }

            // ── 2. Certificate ────────────────────────────────────────────────
            let cert_url = github::certificate_url(&name, &branch);
            action(&log_out, &name,
                t.log_checking_cert.replace("{url}", &cert_url));
            if certificates::cert_exists(&cert_url) {
                set_status(&log_out, &name,
                    InstallStatus::Installing(format!("Certificat {name}")));
                action(&log_out, &name, t.log_installing_cert);
                if let Err(e) = certificates::install_certificate(&cert_url, &name, &tmp_dir) {
                    crate::logger::write("runner", "ERROR", &format!("{name}: certificat: {e}"));
                    set_status(&log_out, &name, InstallStatus::Error(format!("Certificat: {e}")));
                    continue;
                }
            }

            // ── 3. Download binary ────────────────────────────────────────────
            action(&log_out, &name, t.log_searching_asset);
            let exe_path = match download_binary(&name, release.as_ref(), &install_dir, &log_out, t) {
                Ok(p) => p,
                Err(e) => {
                    crate::logger::write("runner", "ERROR", &format!("{name}: {e}"));
                    set_status(&log_out, &name, InstallStatus::Error(e));
                    continue;
                }
            };

            // ── 4. Language file ──────────────────────────────────────────────
            action(&log_out, &name,
                t.log_copying_lang.replace("{lang}", &language));
            if let Err(e) = copy_lang_file(&name, &branch, &language, &lang_folder, &appdata_dir) {
                crate::logger::write("runner", "WARN",
                    &format!("{name}: copie langue '{language}' impossible: {e}"));
            }

            // ── 5. Shortcuts ──────────────────────────────────────────────────
            set_status(&log_out, &name,
                InstallStatus::Installing(format!("Raccourcis {name}")));
            if options.desktop_shortcut {
                action(&log_out, &name, t.log_creating_desktop);
                if let Err(e) = shortcuts::create_desktop_shortcut(&name, &exe_path) {
                    crate::logger::write("runner", "WARN",
                        &format!("{name}: raccourci bureau: {e}"));
                }
            }
            if options.quicklaunch_shortcut {
                action(&log_out, &name, t.log_creating_start);
                if let Err(e) = shortcuts::create_start_menu_shortcut(&name, &exe_path) {
                    crate::logger::write("runner", "WARN",
                        &format!("{name}: raccourci démarrer: {e}"));
                }
            }

            // ── 6. Install record ─────────────────────────────────────────────
            action(&log_out, &name, t.log_writing_record);
            let version = release.as_ref()
                .map(|r| r.tag_name.clone())
                .unwrap_or_else(|| "unknown".to_string());
            let record = paths::InstallRecord {
                version,
                exe_path: exe_path.to_string_lossy().to_string(),
                installed_at: chrono_now(),
            };
            if let Err(e) = paths::write_install_record(&name, &record) {
                crate::logger::write("runner", "WARN",
                    &format!("{name}: install.json: {e}"));
            }

            set_status(&log_out, &name, InstallStatus::Done(name.clone()));
            action(&log_out, &name, t.log_install_done);
        }

        crate::logger::write("runner", "INFO", "=== Installation terminée ===");
    });
}

// ── Binary download ───────────────────────────────────────────────────────────

fn download_binary(
    name: &str,
    release: Option<&GithubRelease>,
    install_dir: &std::path::Path,
    log: &Log,
    t: &crate::i18n::Translations,
) -> Result<std::path::PathBuf, String> {
    let release = release.ok_or_else(|| {
        format!("{name}: {}", t.log_no_release)
    })?;

    let asset = pick_windows_asset(&release.assets)
        .ok_or_else(|| format!("{name}: {}", t.log_no_windows_asset))?;

    let sha256_asset = release.assets.iter().find(|a| {
        a.name == format!("{}.sha256", asset.name)
            || a.name == format!("{}.sha256sum", asset.name)
    });

    let asset_msg = t.log_asset_info
        .replace("{asset}", &asset.name)
        .replace("{size}", &human_size(asset.size));
    action(log, name, &asset_msg);
    crate::logger::write("runner", "INFO", &format!(
        "{name}: asset='{}' size={} sha256={}",
        asset.name, human_size(asset.size),
        sha256_asset.map(|a| a.name.as_str()).unwrap_or("absent")
    ));

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

    let bytes = fetch_bytes(&client, &asset.browser_download_url, name)?;

    // Size check
    let expected_size = asset.size;
    let actual_size   = bytes.len() as u64;
    crate::logger::write("runner", "INFO", &format!(
        "{name}: taille attendue={expected_size} reçue={actual_size}"
    ));
    if expected_size > 0 && actual_size != expected_size {
        return Err(format!(
            "{name}: taille incorrecte (attendu {expected_size} o, reçu {actual_size} o)"
        ));
    }
    action(log, name, t.log_size_verified.replace("{size}", &human_size(actual_size)));

    // SHA-256 check
    if let Some(sha_asset) = sha256_asset {
        action(log, name, t.log_sha256_checking);
        let sha_bytes = fetch_bytes(&client, &sha_asset.browser_download_url, name)?;
        let expected_hex = String::from_utf8_lossy(&sha_bytes)
            .split_whitespace().next().unwrap_or("").to_lowercase();

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let actual_hex = hex::encode(hasher.finalize());

        crate::logger::write("runner", "INFO", &format!(
            "{name}: SHA-256 attendu={expected_hex} calculé={actual_hex}"
        ));
        if !expected_hex.is_empty() && actual_hex != expected_hex {
            return Err(format!(
                "{name}: SHA-256 invalide\n  attendu : {expected_hex}\n  calculé : {actual_hex}"
            ));
        }
        action(log, name, t.log_sha256_ok);
    } else {
        action(log, name, t.log_no_sha256);
    }

    // Write tmp
    std::fs::write(&tmp_path, &bytes).map_err(|e| format!("{name}: écriture tmp: {e}"))?;

    // Extract or copy
    let exe_path = if asset.name.ends_with(".zip") {
        action(log, name, t.log_extracting_zip);
        extract_zip(&tmp_path, install_dir, name).map_err(|e| format!("{name}: zip: {e}"))?
    } else {
        let dest = install_dir.join(&asset.name);
        action(log, name, t.log_copying_to.replace("{path}", &dest.display().to_string()));
        std::fs::copy(&tmp_path, &dest).map_err(|e| format!("{name}: copie: {e}"))?;
        dest
    };

    Ok(exe_path)
}

fn pick_windows_asset(assets: &[ReleaseAsset]) -> Option<&ReleaseAsset> {
    let is_checksum =
        |n: &str| n.ends_with(".sha256") || n.ends_with(".sha256sum") || n.ends_with(".md5");

    assets.iter()
        .filter(|a| !is_checksum(&a.name))
        .filter_map(|a| {
            let n = a.name.to_lowercase();
            let mut score: i32 = 0;

            if n.ends_with(".exe")      { score += 4; }
            else if n.ends_with(".msi") { score += 3; }
            else if n.ends_with(".zip") { score += 2; }

            if n.contains("windows") || n.contains("win") { score += 6; }
            if n.contains("x86_64") || n.contains("x64") || n.contains("amd64") { score += 8; }

            if n.contains("linux") || n.contains("darwin") || n.contains("macos") || n.contains("mac-") {
                score -= 20;
            }

            if score > -10 { Some((a, score)) } else { None }
        })
        .max_by_key(|(_, s)| *s)
        .map(|(a, _)| a)
}

// ── Pre-check phase ───────────────────────────────────────────────────────────

fn run_precheck(programs: &[ProgramInstall], log: &Log, t: &crate::i18n::Translations) {
    let key = t.precheck_title;

    {
        let mut l = log.lock().unwrap();
        l.insert(0, crate::state::InstallLogEntry {
            app: key.to_string(),
            status: crate::state::InstallStatus::Installing(key.to_string()),
            actions: Vec::new(),
        });
    }

    // 1. Connectivity
    action(log, key, t.precheck_checking_conn);
    let conn_ms = match crate::github::check_connectivity() {
        Ok(ms) => {
            action(log, key, t.precheck_conn_ok.replace("{ms}", &ms.to_string()));
            crate::logger::write("runner", "INFO", &format!("GitHub joignable en {ms}ms"));
            Some(ms)
        }
        Err(e) => {
            action(log, key, t.precheck_conn_failed.replace("{error}", &e));
            crate::logger::write("runner", "WARN", &format!("GitHub inaccessible: {e}"));
            None
        }
    };

    // 2. Total download size from selected assets
    let total_bytes: u64 = programs.iter()
        .filter_map(|(_, release, _, _, _)| release.as_ref())
        .filter_map(|r| pick_windows_asset(&r.assets))
        .map(|a| a.size)
        .sum();

    if total_bytes > 0 {
        let msg = t.precheck_total_size.replace("{size}", &human_size(total_bytes));
        action(log, key, &msg);
        crate::logger::write("runner", "INFO", &msg);
    }

    // 3. Speed test + ETA (only if connectivity confirmed)
    if conn_ms.is_some() {
        action(log, key, t.precheck_speed_test);
        match estimate_speed_bps() {
            Ok(bps) if bps > 0 => {
                let msg = t.precheck_speed.replace("{speed}", &human_speed(bps));
                action(log, key, &msg);
                crate::logger::write("runner", "INFO", &msg);

                if total_bytes > 0 {
                    let eta_secs = total_bytes / bps.max(1);
                    let eta_msg = t.precheck_eta.replace("{eta}", &format_eta(eta_secs));
                    action(log, key, &eta_msg);
                    crate::logger::write("runner", "INFO", &eta_msg);
                }
            }
            _ => {}
        }
    }

    set_status(log, key, crate::state::InstallStatus::Done(t.precheck_done.to_string()));
    crate::logger::write("runner", "INFO", "Pré-vérification terminée");
}

fn estimate_speed_bps() -> Result<u64, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("suite_install/0.1")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("https://api.github.com/orgs/{}", crate::github::ORG);
    let start = std::time::Instant::now();
    let resp = client.get(&url).send().map_err(|e| e.to_string())?;
    let bytes = resp.bytes().map_err(|e| e.to_string())?;
    let elapsed = start.elapsed();

    let secs = elapsed.as_secs_f64().max(0.001);
    Ok((bytes.len() as f64 / secs) as u64)
}

fn human_speed(bps: u64) -> String {
    if bps < 1024 { format!("{bps} B/s") }
    else if bps < 1024 * 1024 { format!("{:.1} KB/s", bps as f64 / 1024.0) }
    else { format!("{:.1} MB/s", bps as f64 / 1024.0 / 1024.0) }
}

fn format_eta(secs: u64) -> String {
    if secs < 60 { format!("{secs}s") }
    else if secs < 3600 { format!("{}m {:02}s", secs / 60, secs % 60) }
    else { format!("{}h {:02}m", secs / 3600, (secs % 3600) / 60) }
}

fn fetch_bytes(
    client: &reqwest::blocking::Client,
    url: &str,
    name: &str,
) -> Result<Vec<u8>, String> {
    let resp = client.get(url).send()
        .map_err(|e| format!("{name}: connexion ({url}): {e}"))?;
    let status = resp.status();
    crate::logger::write("runner", "HTTP", &format!("{name}: GET {url} -> {status}"));
    if !status.is_success() {
        return Err(format!("{name}: HTTP {status} pour {url}"));
    }
    resp.bytes().map(|b| b.to_vec())
        .map_err(|e| format!("{name}: lecture ({url}): {e}"))
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
            if let Some(p) = out.parent() { std::fs::create_dir_all(p)?; }
            let mut outfile = std::fs::File::create(&out)?;
            std::io::copy(&mut zf, &mut outfile)?;
            if zf.name().ends_with(".exe") { exe_path = out.clone(); }
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

    let candidates: Vec<String> = {
        let mut v = vec![format!("{lang_folder}/{language}")];
        if !language.contains(".default.") {
            v.push(format!("{lang_folder}/EN_en.default.toml"));
        }
        v
    };

    for path in &candidates {
        let url = github::raw_url(name, branch, path);
        crate::logger::write("runner", "INFO", &format!("{name}: langue {url}"));
        let resp = client.get(&url).send()?;
        if resp.status().is_success() {
            let lang_dir = appdata_dir.join("lang");
            std::fs::create_dir_all(&lang_dir)?;
            let bytes = resp.bytes()?;
            let file_name = std::path::Path::new(path).file_name().unwrap_or_default();
            std::fs::write(lang_dir.join(file_name), &bytes)?;
            crate::logger::write("runner", "INFO", &format!(
                "{name}: langue '{}' copiée dans '{}'",
                file_name.to_string_lossy(), lang_dir.display()
            ));
            return Ok(());
        }
        crate::logger::write("runner", "WARN",
            &format!("{name}: {path} absent (HTTP {})", resp.status()));
    }

    anyhow::bail!("aucun fichier de langue pour {name} (cherche: {})", candidates.join(", "))
}

// ── Uninstall ─────────────────────────────────────────────────────────────────

pub fn uninstall_programs(names: Vec<String>, log_out: Log, lang: String) {
    std::thread::spawn(move || {
        let t = crate::i18n::get(&lang);
        crate::logger::write("runner", "INFO", &format!(
            "=== Désinstallation démarrée — {} programme(s) ===", names.len()
        ));

        for name in names {
            action(&log_out, &name,
                t.log_starting_uninstall.replace("{name}", &name));
            set_status(&log_out, &name,
                InstallStatus::Installing(format!("Désinstallation {name}")));

            let install_dir = paths::program_files_dir(&name);
            let appdata_dir = paths::appdata_dir(&name);
            let tmp_dir     = paths::temp_dir(&name);

            // Remove install dir
            action(&log_out, &name,
                t.log_removing.replace("{path}", &install_dir.display().to_string()));
            if install_dir.exists() {
                match std::fs::remove_dir_all(&install_dir) {
                    Ok(_) => crate::logger::write("runner", "INFO",
                        &format!("{name}: dossier install supprimé")),
                    Err(e) => {
                        let msg = t.log_cannot_remove
                            .replace("{path}", &install_dir.display().to_string())
                            .replace("{error}", &e.to_string());
                        crate::logger::write("runner", "ERROR", &format!("{name}: {msg}"));
                        action(&log_out, &name, msg);
                    }
                }
            } else {
                action(&log_out, &name,
                    t.log_dir_already_removed.replace("{path}", &install_dir.display().to_string()));
            }

            // Shortcuts
            action(&log_out, &name, t.log_removing_shortcuts);
            remove_shortcut_desktop(&name);
            remove_shortcut_start_menu(&name);

            // Tmp
            action(&log_out, &name,
                t.log_removing.replace("{path}", &tmp_dir.display().to_string()));
            if tmp_dir.exists() { let _ = std::fs::remove_dir_all(&tmp_dir); }

            // AppData
            action(&log_out, &name,
                t.log_removing.replace("{path}", &appdata_dir.display().to_string()));
            if appdata_dir.exists() {
                match std::fs::remove_dir_all(&appdata_dir) {
                    Ok(_) => crate::logger::write("runner", "INFO",
                        &format!("{name}: appdata supprimé")),
                    Err(e) => {
                        let msg = t.log_cannot_remove
                            .replace("{path}", &appdata_dir.display().to_string())
                            .replace("{error}", &e.to_string());
                        crate::logger::write("runner", "ERROR", &format!("{name}: {msg}"));
                        action(&log_out, &name, msg);
                    }
                }
            }

            set_status(&log_out, &name, InstallStatus::Done(name.clone()));
            action(&log_out, &name, t.log_uninstall_done);
        }

        cleanup_empty_roots();
        crate::logger::write("runner", "INFO", "=== Désinstallation terminée ===");
    });
}

fn remove_shortcut_desktop(app_name: &str) {
    if let Some(desktop) = dirs::desktop_dir() {
        let lnk = desktop.join(format!("{}.lnk", app_name));
        if lnk.exists() {
            let _ = std::fs::remove_file(&lnk);
            crate::logger::write("runner", "INFO",
                &format!("{app_name}: raccourci bureau supprimé"));
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
                    crate::logger::write("runner", "INFO",
                        &format!("{app_name}: raccourci menu supprimé"));
                }
            }
        }
    }
}

fn cleanup_empty_roots() {
    let base = dirs::data_dir().unwrap_or_default().join("rusty-suite");
    let tmp = base.join(".tmp");
    if tmp.exists() {
        let is_empty = std::fs::read_dir(&tmp)
            .map(|mut d| d.next().is_none()).unwrap_or(false);
        if is_empty { let _ = std::fs::remove_dir(&tmp); }
    }
    if base.exists() {
        let is_empty = std::fs::read_dir(&base)
            .map(|mut d| d.next().is_none()).unwrap_or(false);
        if is_empty {
            let _ = std::fs::remove_dir(&base);
            crate::logger::write("runner", "INFO", "Dossier rusty-suite vide supprimé");
        }
    }
    let pf_base = paths::program_files_dir("__sentinel__")
        .parent().unwrap_or(std::path::Path::new(""))
        .parent().unwrap_or(std::path::Path::new(""))
        .to_path_buf();
    if pf_base.exists() {
        let is_empty = std::fs::read_dir(&pf_base)
            .map(|mut d| d.next().is_none()).unwrap_or(false);
        if is_empty { let _ = std::fs::remove_dir(&pf_base); }
    }
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn human_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{bytes} B") }
    else if bytes < 1024 * 1024 { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else { format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0) }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
