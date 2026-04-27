use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};

static LOG_FILE: OnceLock<Mutex<Option<File>>> = OnceLock::new();

/// Open the log file and write the session header.
/// Must be called once at application startup.
pub fn init() {
    LOG_FILE.get_or_init(|| {
        let path = log_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut file = OpenOptions::new().create(true).append(true).open(&path).ok();
        if let Some(ref mut f) = file {
            let _ = writeln!(f, "=== Rusty Suite Installer — {} UTC ===", timestamp_str());
            let _ = writeln!(f, "Log file: {}", path.display());
            let _ = writeln!(f);
            let _ = f.flush();
            eprintln!("[logger] Log: {}", path.display());
        } else {
            eprintln!("[logger][WARN] Impossible d'ouvrir le fichier de log: {}", path.display());
        }
        Mutex::new(file)
    });
}

/// Write a log entry both to the file and to stderr.
pub fn write(module: &str, level: &str, msg: &str) {
    let line = format!("[{}][{}][{}] {}\n", timestamp_str(), level, module, msg);
    eprint!("{line}");
    if let Some(mutex) = LOG_FILE.get() {
        if let Ok(mut guard) = mutex.lock() {
            if let Some(ref mut f) = *guard {
                let _ = f.write_all(line.as_bytes());
                let _ = f.flush();
            }
        }
    }
}

// ── Path ──────────────────────────────────────────────────────────────────────

fn log_path() -> std::path::PathBuf {
    let base = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("C:\\Users\\Default\\AppData\\Roaming"));
    base.join("rusty-suite")
        .join("suite-install-logs")
        .join(format!("suite-install-{}.log", file_timestamp()))
}

// ── Timestamp helpers ─────────────────────────────────────────────────────────

/// Human-readable timestamp for log lines: "2026-04-27 15:30:45"
fn timestamp_str() -> String {
    let (y, mo, d, h, mi, s) = date_parts(current_secs());
    format!("{y}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}")
}

/// Filename-safe timestamp: "20260427_153045"
fn file_timestamp() -> String {
    let (y, mo, d, h, mi, s) = date_parts(current_secs());
    format!("{y}{mo:02}{d:02}_{h:02}{mi:02}{s:02}")
}

fn current_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn date_parts(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let time = secs % 86400;
    let h  = time / 3600;
    let mi = (time % 3600) / 60;
    let s  = time % 60;

    let mut days = secs / 86400;
    let mut year = 1970u64;
    loop {
        let dy = if is_leap(year) { 366 } else { 365 };
        if days < dy { break; }
        days -= dy;
        year += 1;
    }
    let leap = is_leap(year);
    let mdays: [u64; 12] = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let (mut month, mut dom) = (1u64, days);
    for &md in &mdays {
        if dom < md { break; }
        dom -= md;
        month += 1;
    }
    (year, month, dom + 1, h, mi, s)
}

fn is_leap(y: u64) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}
