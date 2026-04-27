use std::path::Path;
use anyhow::Result;

/// Download the certificate for a program and install it in the user's trusted store.
pub fn install_certificate(cert_url: &str, app_name: &str, temp_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(temp_dir)?;
    let cert_path = temp_dir.join(format!("{}.crt", app_name));

    // Download certificate
    let client = reqwest::blocking::Client::builder()
        .user_agent("suite_install/0.1")
        .build()?;
    let bytes = client.get(cert_url).send()?.bytes()?;
    std::fs::write(&cert_path, &bytes)?;

    install_cert_windows(&cert_path)?;
    Ok(())
}

#[cfg(windows)]
fn install_cert_windows(cert_path: &Path) -> anyhow::Result<()> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let status = std::process::Command::new("certutil")
        .args(["-addstore", "-user", "Root", cert_path.to_str().unwrap()])
        .creation_flags(CREATE_NO_WINDOW)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("certutil failed with status {}", status))
    }
}

#[cfg(not(windows))]
fn install_cert_windows(_cert_path: &Path) -> anyhow::Result<()> {
    Ok(()) // no-op on non-Windows
}

/// Returns true if a certificate exists for this app in the repo.
pub fn cert_exists(cert_url: &str) -> bool {
    let client = reqwest::blocking::Client::builder()
        .user_agent("suite_install/0.1")
        .build();
    if let Ok(c) = client {
        if let Ok(resp) = c.head(cert_url).send() {
            return resp.status().is_success();
        }
    }
    false
}
