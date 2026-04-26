use serde::{Deserialize, Serialize};

const ORG: &str = "rusty-suite";
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
    log_info(format!("GET {org_url}"));
    let mut resp = client.get(&org_url).send()?;

    let (url, source_kind) = if resp.status() == 404 {
        let user_url = format!("{}/users/{}/repos?per_page=100&type=public", API_BASE, ORG);
        log_info(format!(
            "{ORG} introuvable comme organisation, tentative comme utilisateur: GET {user_url}"
        ));
        resp = client.get(&user_url).send()?;
        (user_url, "utilisateur")
    } else {
        (org_url, "organisation")
    };

    let repos: Vec<GithubRepo> = decode_json_response(resp, &url)?;
    log_info(format!(
        "{} depot(s) public(s) recus pour le compte {source_kind} {ORG}",
        repos.len()
    ));

    // exclude the installer itself and meta repos
    let filtered = repos
        .into_iter()
        .filter(|r| r.name != "suite_install" && !r.name.starts_with('.'))
        .collect();
    Ok(filtered)
}

pub fn fetch_latest_release(repo: &str) -> anyhow::Result<Option<GithubRelease>> {
    let client = github_client()?;

    let url = format!("{}/repos/{}/{}/releases/latest", API_BASE, ORG, repo);
    log_info(format!("GET {url}"));
    let resp = client.get(&url).send()?;
    if resp.status() == 404 {
        log_info(format!("Aucune release latest pour {repo} (404)"));
        return Ok(None);
    }
    let release: GithubRelease = decode_json_response(resp, &url)?;
    log_info(format!(
        "Release latest recue pour {repo}: {} avec {} asset(s)",
        release.tag_name,
        release.assets.len()
    ));
    Ok(Some(release))
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
        log_info(format!("GET {url}"));
        let resp = client.get(&url).send()?;

        if resp.status() == 404 {
            log_info(format!("Dossier '{folder}' absent pour {repo}, essai suivant"));
            continue;
        }

        let contents: Vec<GithubContent> = decode_json_response(resp, &url)?;
        let mut languages: Vec<String> = contents
            .into_iter()
            .filter(|entry| entry.kind == "file" && entry.name.ends_with(".toml"))
            .map(|entry| entry.name)
            .collect();
        languages.sort();

        log_info(format!(
            "{} langue(s) trouvee(s) pour {repo} dans '{folder}': {}",
            languages.len(),
            languages.join(", ")
        ));
        return Ok((languages, folder.to_string()));
    }

    log_info(format!("Aucun dossier de langue trouve pour {repo}"));
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
        log_error(format!("Lecture du corps impossible pour {url}: {err}"));
        anyhow::anyhow!("lecture de la reponse GitHub impossible ({status}) pour {url}: {err}")
    })?;

    log_info(format!(
        "Reponse {status} depuis {url} (content-type: {content_type}, {} octet(s))",
        body.len()
    ));

    if !status.is_success() {
        let excerpt = body_excerpt(&body);
        log_error(format!(
            "Statut HTTP inattendu pour {url}: {status}; corps: {excerpt}"
        ));
        anyhow::bail!("GitHub a retourne {status} pour {url}: {excerpt}");
    }

    serde_json::from_str(&body).map_err(|err| {
        let excerpt = body_excerpt(&body);
        log_error(format!(
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

fn log_info(message: impl AsRef<str>) {
    eprintln!("[suite_install][github][INFO] {}", message.as_ref());
}

fn log_error(message: impl AsRef<str>) {
    eprintln!("[suite_install][github][ERROR] {}", message.as_ref());
}
