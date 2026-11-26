use serde::{Deserialize, Serialize};

pub const ORG: &str = "rusty-suite";
const API_BASE: &str = "https://api.github.com";
const USER_AGENT: &str = concat!("suite_install/", env!("CARGO_PKG_VERSION"));

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
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct GithubContent {
    name: String,
    #[serde(rename = "type")]
    kind: String,
}

pub fn fetch_org_repos() -> anyhow::Result<Vec<GithubRepo>> {
    let client = github_client()?;

    let org_url = format!("{}/orgs/{}/repos?per_page=100&type=public", API_BASE, ORG);
    crate::logger::write("github", "INFO", &format!("GET {org_url}"));
    let mut resp = client.get(&org_url).send()?;

    let (url, source_kind) = if resp.status() == 404 {
        let user_url = format!("{}/users/{}/repos?per_page=100&type=public", API_BASE, ORG);
        crate::logger::write("github", "INFO", &format!(
            "{ORG} introuvable comme organisation, tentative comme utilisateur: GET {user_url}"
        ));
        resp = client.get(&user_url).send()?;
        (user_url, "utilisateur")
    } else {
        (org_url, "organisation")
    };

    let repos: Vec<GithubRepo> = decode_json_response(resp, &url)?;
    crate::logger::write("github", "INFO", &format!(
        "{} depot(s) public(s) recus pour le compte {source_kind} {ORG}",
        repos.len()
    ));

    // exclude the installer itself, the org meta-repo, and infrastructure repos
    let excluded = ["suite_install", "rusty-suite"];
    let filtered = repos
        .into_iter()
        .filter(|r| !excluded.contains(&r.name.as_str()) && !r.name.starts_with('.'))
        .collect();
    Ok(filtered)
}

pub fn fetch_latest_release(repo: &str) -> anyhow::Result<Option<GithubRelease>> {
    let client = github_client()?;

    // Try /releases/latest first
    let url = format!("{}/repos/{}/{}/releases/latest", API_BASE, ORG, repo);
    crate::logger::write("github", "INFO", &format!("GET {url}"));
    let resp = client.get(&url).send()?;
    let status = resp.status();

    if status.is_success() {
        let release: GithubRelease = decode_json_response(resp, &url)?;
        crate::logger::write("github", "INFO", &format!(
            "Release latest pour {repo}: {} avec {} asset(s)",
            release.tag_name, release.assets.len()
        ));
        return Ok(Some(release));
    }

    // Fallback: list API — picks first non-draft release
    drop(resp);
    crate::logger::write("github", "INFO", &format!(
        "Pas de release 'latest' pour {repo} ({status}), fallback liste"
    ));
    let list_url = format!("{}/repos/{}/{}/releases?per_page=20", API_BASE, ORG, repo);
    crate::logger::write("github", "INFO", &format!("GET {list_url}"));
    let resp2 = client.get(&list_url).send()?;

    if !resp2.status().is_success() {
        crate::logger::write("github", "WARN", &format!(
            "Aucune release pour {repo} ({})", resp2.status()
        ));
        return Ok(None);
    }

    let releases: Vec<GithubRelease> = decode_json_response(resp2, &list_url)?;
    // Prefer a stable release; fall back to a pre-release only if nothing else exists.
    let mut candidates: Vec<GithubRelease> = releases.into_iter().filter(|r| !r.draft).collect();
    let release = match candidates.iter().position(|r| !r.prerelease) {
        Some(i) => Some(candidates.swap_remove(i)),
        None => candidates.into_iter().next(),
    };

    match &release {
        Some(r) => crate::logger::write("github", "INFO", &format!(
            "Release fallback pour {repo}: {} avec {} asset(s)", r.tag_name, r.assets.len()
        )),
        None => crate::logger::write("github", "WARN", &format!(
            "Aucune release non-draft pour {repo}"
        )),
    }

    Ok(release)
}

/// Picks the most suitable Windows binary among the release assets.
pub fn pick_windows_asset(assets: &[ReleaseAsset]) -> Option<&ReleaseAsset> {
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

/// Checks that a release really offers an installable Windows binary:
/// a matching asset exists, is not empty, and its download URL answers.
/// Returns `Err(reason)` describing why the release is not installable.
pub fn verify_release_installable(repo: &str, release: &GithubRelease) -> Result<(), String> {
    let asset = pick_windows_asset(&release.assets)
        .ok_or_else(|| format!("aucun asset Windows dans la release {}", release.tag_name))?;

    if asset.size == 0 {
        return Err(format!("asset '{}' vide", asset.name));
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    crate::logger::write("github", "INFO", &format!("HEAD {}", asset.browser_download_url));
    let resp = client
        .head(&asset.browser_download_url)
        .send()
        .map_err(|e| format!("asset '{}' injoignable: {e}", asset.name))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("asset '{}' indisponible (HTTP {status})", asset.name));
    }

    crate::logger::write("github", "INFO", &format!(
        "{repo}: release {} verifiee, asset '{}' disponible", release.tag_name, asset.name
    ));
    Ok(())
}

/// HEAD request to api.github.com to verify connectivity. Returns latency in ms.
pub fn check_connectivity() -> Result<u64, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("{}/orgs/{}", API_BASE, ORG);
    let start = std::time::Instant::now();
    let resp = client.get(&url).send()
        .map_err(|e| format!("connexion échouée: {e}"))?;
    let ms = start.elapsed().as_millis() as u64;

    // 200 or 404 (unauthenticated rate-limit) both confirm reachability
    let s = resp.status();
    if s.is_success() || s == 404 {
        Ok(ms)
    } else {
        Err(format!("HTTP {s}"))
    }
}

/// Returns (language_files, folder_name).
/// Tries "langue" first (Rusty Suite convention), then "lang" as fallback.
pub fn fetch_language_files(repo: &str, branch: &str) -> anyhow::Result<(Vec<String>, String)> {
    let client = github_client()?;

    for folder in ["langue", "lang"] {
        let url = format!(
            "{}/repos/{}/{}/contents/{}?ref={}",
            API_BASE, ORG, repo, folder, branch
        );
        crate::logger::write("github", "INFO", &format!("GET {url}"));
        let resp = client.get(&url).send()?;

        if resp.status() == 404 {
            crate::logger::write("github", "INFO", &format!(
                "Dossier '{folder}' absent pour {repo}, essai suivant"
            ));
            continue;
        }

        let contents: Vec<GithubContent> = decode_json_response(resp, &url)?;
        let mut languages: Vec<String> = contents
            .into_iter()
            .filter(|entry| entry.kind == "file" && entry.name.ends_with(".toml"))
            .map(|entry| entry.name)
            .collect();
        languages.sort();

        crate::logger::write("github", "INFO", &format!(
            "{} langue(s) trouvee(s) pour {repo} dans '{folder}': {}",
            languages.len(),
            languages.join(", ")
        ));
        return Ok((languages, folder.to_string()));
    }

    crate::logger::write("github", "INFO", &format!("Aucun dossier de langue trouve pour {repo}"));
    Ok((Vec::new(), "langue".to_string()))
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

fn github_client() -> anyhow::Result<reqwest::blocking::Client> {
    Ok(reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .build()?)
}

fn decode_json_response<T>(resp: reqwest::blocking::Response, url: &str) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let status = resp.status();
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("inconnu")
        .to_string();

    let body = resp.text().map_err(|err| {
        crate::logger::write("github", "ERROR", &format!("Lecture du corps impossible pour {url}: {err}"));
        anyhow::anyhow!("lecture de la reponse GitHub impossible ({status}) pour {url}: {err}")
    })?;

    crate::logger::write("github", "INFO", &format!(
        "Reponse {status} depuis {url} (content-type: {content_type}, {} octet(s))",
        body.len()
    ));

    if !status.is_success() {
        let excerpt = body_excerpt(&body);
        crate::logger::write("github", "ERROR", &format!(
            "Statut HTTP inattendu pour {url}: {status}; corps: {excerpt}"
        ));
        anyhow::bail!("GitHub a retourne {status} pour {url}: {excerpt}");
    }

    serde_json::from_str(&body).map_err(|err| {
        let excerpt = body_excerpt(&body);
        crate::logger::write("github", "ERROR", &format!(
            "JSON invalide pour {url}: {err}; content-type: {content_type}; corps: {excerpt}"
        ));
        anyhow::anyhow!("decodage JSON impossible pour {url}: {err}. Extrait: {excerpt}")
    })
}

fn body_excerpt(body: &str) -> String {
    let normalized = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut excerpt: String = normalized.chars().take(300).collect();
    if normalized.chars().count() > 300 {
        excerpt.push_str("...");
    }
    excerpt
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(name: &str, size: u64) -> ReleaseAsset {
        ReleaseAsset { name: name.into(), browser_download_url: String::new(), size }
    }

    #[test]
    fn picks_windows_binary_over_other_platforms() {
        let assets = vec![
            asset("app-linux-x86_64", 10),
            asset("app-macos.zip", 10),
            asset("app-windows-x86_64.exe", 10),
            asset("app-windows-x86_64.exe.sha256", 1),
        ];
        assert_eq!(pick_windows_asset(&assets).unwrap().name, "app-windows-x86_64.exe");
    }

    #[test]
    fn no_windows_binary_means_none() {
        let assets = vec![asset("app-linux-x86_64", 10), asset("app.sha256", 1)];
        assert!(pick_windows_asset(&assets).is_none());
        assert!(pick_windows_asset(&[]).is_none());
    }

    #[test]
    fn release_without_windows_asset_is_not_installable() {
        let rel = GithubRelease { tag_name: "v1".into(), assets: vec![asset("app-linux", 5)], draft: false, prerelease: false };
        assert!(verify_release_installable("x", &rel).is_err());
    }

    #[test]
    fn empty_asset_is_not_installable() {
        let rel = GithubRelease { tag_name: "v1".into(), assets: vec![asset("app.exe", 0)], draft: false, prerelease: false };
        assert!(verify_release_installable("x", &rel).is_err());
    }

    /// Network test against the real organisation: run with `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn live_check_all_repos() {
        let repos = fetch_org_repos().expect("repo list");
        assert!(!repos.is_empty());
        for repo in repos {
            let rel = fetch_latest_release(&repo.name).expect("release request");
            let verdict = match &rel {
                Some(r) => verify_release_installable(&repo.name, r).map(|_| format!("OK ({})", r.tag_name)),
                None => Err("aucune release".to_string()),
            };
            println!("[live] {:<24} -> {:?}", repo.name, verdict);
        }
    }

    /// Network test: a missing download URL must be rejected.
    #[test]
    #[ignore]
    fn live_missing_asset_is_rejected() {
        let rel = GithubRelease {
            tag_name: "v0.0.0".into(),
            assets: vec![ReleaseAsset {
                name: "app-windows-x86_64.exe".into(),
                browser_download_url: format!("https://github.com/{ORG}/suite_install/releases/download/does-not-exist/none.exe"),
                size: 10,
            }],
            draft: false,
            prerelease: false,
        };
        assert!(verify_release_installable("suite_install", &rel).is_err());
    }
}
