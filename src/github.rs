use serde::{Deserialize, Serialize};

pub const ORG: &str = "rusty-suite";
const API_BASE: &str = "https://api.github.com";
const USER_AGENT: &str = "suite_install/0.1";

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
    let release = releases.into_iter().find(|r| !r.draft);

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
