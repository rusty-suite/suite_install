use std::path::{Path, PathBuf};
use anyhow::Result;

pub fn create_desktop_shortcut(app_name: &str, exe_path: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        let desktop = dirs::desktop_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find desktop folder"))?;
        create_lnk(&desktop.join(format!("{}.lnk", app_name)), exe_path)?;
    }
    Ok(())
}

pub fn create_start_menu_shortcut(app_name: &str, exe_path: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        let start = start_menu_programs_dir()?;
        let folder = start.join("Rusty Suite");
        std::fs::create_dir_all(&folder)?;
        create_lnk(&folder.join(format!("{}.lnk", app_name)), exe_path)?;
    }
    Ok(())
}

#[cfg(windows)]
fn start_menu_programs_dir() -> Result<PathBuf> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders",
    )?;
    let path: String = key.get_value("Programs")?;
    Ok(PathBuf::from(path))
}

#[cfg(windows)]
fn create_lnk(lnk_path: &Path, target: &Path) -> Result<()> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let script = format!(
        r#"$ws = New-Object -ComObject WScript.Shell; $sc = $ws.CreateShortcut('{}'); $sc.TargetPath = '{}'; $sc.Save()"#,
        lnk_path.display(),
        target.display()
    );
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(CREATE_NO_WINDOW)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("PowerShell shortcut creation failed"))
    }
}
