use std::path::PathBuf;

pub fn program_files_dir(app_name: &str) -> PathBuf {
    let base = std::env::var("PROGRAMFILES")
        .unwrap_or_else(|_| "C:\\Program Files".to_string());
    PathBuf::from(base).join("rusty-suite").join(app_name)
}

pub fn appdata_dir(app_name: &str) -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Roaming"))
        .join("rusty-suite")
        .join(app_name)
}

pub fn temp_dir(app_name: &str) -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Roaming"))
        .join("rusty-suite")
        .join(".tmp")
        .join(app_name)
}

pub fn install_record_path(app_name: &str) -> PathBuf {
    appdata_dir(app_name).join("install.json")
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InstallRecord {
    pub version: String,
    pub exe_path: String,
    pub installed_at: String,
}

pub fn read_install_record(app_name: &str) -> Option<InstallRecord> {
    let path = install_record_path(app_name);
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn write_install_record(app_name: &str, record: &InstallRecord) -> anyhow::Result<()> {
    let path = install_record_path(app_name);
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, serde_json::to_string_pretty(record)?)?;
    Ok(())
}
