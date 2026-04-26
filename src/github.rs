use serde::{Deserialize, Serialize};

const ORG: &str = "rusty-suite";
const API_BASE: &str = "https://api.github.com";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GithubRepo {
    pub name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub default_branch: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

pub fn fetch_org_repos() -> anyhow::Result<Vec<GithubRepo>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("suite_install/0.1")
        .build()?;

    let url = format!("{}/orgs/{}/repos?per_page=100&type=public", API_BASE, ORG);
    let resp = client.get(&url).send()?;
    let repos: Vec<GithubRepo> = resp.json()?;

    // exclude the installer itself and meta repos
    let filtered = repos
        .into_iter()
        .filter(|r| r.name != "suite_install" && !r.name.starts_with('.'))
        .collect();
    Ok(filtered)
}

pub fn fetch_latest_release(repo: &str) -> anyhow::Result<Option<GithubRelease>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("suite_install/0.1")
        .build()?;

    let url = format!("{}/repos/{}/{}/releases/latest", API_BASE, ORG, repo);
    let resp = client.get(&url).send()?;
    if resp.status() == 404 {
        return Ok(None);
    }
    let release: GithubRelease = resp.json()?;
    Ok(Some(release))
}

/// Returns the raw URL for a file at the root of a repo's default branch.
pub fn raw_url(repo: &str, branch: &str, path: &str) -> String {
    format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{}",
        ORG, repo, branch, path
    )
}

/// Check if a public certificate exists for this repo in the `certificat_public` folder.
pub fn certificate_url(repo: &str, branch: &str) -> String {
    raw_url(repo, branch, &format!("certificat_public/{}.crt", repo))
}
