use serde::{Deserialize, Serialize};
use crate::{credentials, github, cloudflare, llm, scraper};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupStatus {
    pub github_connected: bool,
    pub github_username: Option<String>,
    pub cloudflare_connected: bool,
    pub cloudflare_account: Option<String>,
    pub cloudflare_account_id: Option<String>,
    pub llm_connected: bool,
    pub llm_provider: Option<String>,
}

#[tauri::command]
pub async fn get_setup_status() -> Result<SetupStatus, String> {
    let github_connected = credentials::has_credential("github");
    let cloudflare_connected = credentials::has_credential("cloudflare");
    let llm_connected = credentials::has_credential("chatgpt")
        || credentials::has_credential("claude")
        || credentials::has_credential("gemini")
        || credentials::has_credential("openai_compatible");

    let llm_provider = if credentials::has_credential("chatgpt") {
        Some("chatgpt".to_string())
    } else if credentials::has_credential("claude") {
        Some("claude".to_string())
    } else if credentials::has_credential("gemini") {
        Some("gemini".to_string())
    } else if credentials::has_credential("openai_compatible") {
        Some("openai_compatible".to_string())
    } else {
        None
    };

    Ok(SetupStatus {
        github_connected,
        github_username: None, // Lazy-loaded on verify
        cloudflare_connected,
        cloudflare_account: None,
        cloudflare_account_id: None,
        llm_connected,
        llm_provider,
    })
}

#[derive(Debug, Deserialize)]
pub struct SaveCredentialRequest {
    pub provider: String,
    pub token: String,
    pub token_type: String, // "oauth" | "api_key"
    pub refresh_token: Option<String>,
}

#[tauri::command]
pub async fn save_credential(request: SaveCredentialRequest) -> Result<(), String> {
    let cred = credentials::Credential {
        provider: request.provider.clone(),
        token_type: request.token_type,
        access_token: request.token,
        refresh_token: request.refresh_token,
        expires_at: None,
    };
    credentials::store_credential(&request.provider, &cred)
}

#[tauri::command]
pub async fn remove_credential(provider: String) -> Result<(), String> {
    credentials::remove_credential(&provider)
}

#[derive(Debug, Serialize)]
pub struct OAuthUrl {
    pub url: String,
    pub state: String,
}

#[tauri::command]
pub async fn generate_oauth_url(provider: String, client_id: String) -> Result<OAuthUrl, String> {
    let state = format!("youai_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis());

    let redirect_uri = "https://youai.dev/oauth/callback"; // User copies this URL back

    let url = match provider.as_str() {
        "chatgpt" => {
            format!(
                "https://auth.openai.com/authorize?client_id={}&redirect_uri={}&response_type=code&scope=openid%20profile&state={}",
                client_id, redirect_uri, state
            )
        }
        "github" => {
            format!(
                "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=repo&state={}",
                client_id, redirect_uri, state
            )
        }
        "gemini" => {
            format!(
                "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=https://www.googleapis.com/auth/generative-language&state={}&access_type=offline",
                client_id, redirect_uri, state
            )
        }
        _ => return Err(format!("OAuth not supported for provider: {}", provider)),
    };

    Ok(OAuthUrl { url, state })
}

#[derive(Debug, Deserialize)]
pub struct ExchangeRequest {
    pub provider: String,
    pub redirect_url: String, // The full URL they paste back
    pub client_id: String,
    pub client_secret: String,
}

#[tauri::command]
pub async fn exchange_oauth_code(request: ExchangeRequest) -> Result<(), String> {
    // Parse the authorization code from the redirect URL
    let parsed = url::Url::parse(&request.redirect_url)
        .map_err(|e| format!("Invalid URL: {}. Make sure you copied the entire URL from your browser.", e))?;

    let code = parsed
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.to_string())
        .ok_or("No authorization code found in the URL. Make sure you copied the complete URL.")?;

    let client = reqwest::Client::new();
    let redirect_uri = "https://youai.dev/oauth/callback";

    let (token_url, body) = match request.provider.as_str() {
        "chatgpt" => (
            "https://auth.openai.com/oauth/token",
            serde_json::json!({
                "grant_type": "authorization_code",
                "code": code,
                "redirect_uri": redirect_uri,
                "client_id": request.client_id,
                "client_secret": request.client_secret,
            }),
        ),
        "github" => (
            "https://github.com/login/oauth/access_token",
            serde_json::json!({
                "client_id": request.client_id,
                "client_secret": request.client_secret,
                "code": code,
                "redirect_uri": redirect_uri,
            }),
        ),
        "gemini" => (
            "https://oauth2.googleapis.com/token",
            serde_json::json!({
                "grant_type": "authorization_code",
                "code": code,
                "redirect_uri": redirect_uri,
                "client_id": request.client_id,
                "client_secret": request.client_secret,
            }),
        ),
        _ => return Err(format!("OAuth exchange not supported for: {}", request.provider)),
    };

    let resp = client
        .post(token_url)
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Token exchange failed: {}", e))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("Read failed: {}", e))?;

    if !status.is_success() {
        return Err(format!("Authorization failed ({}): {}. Try connecting again.", status, text));
    }

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("Parse failed: {}", e))?;

    let access_token = json["access_token"]
        .as_str()
        .ok_or("No access token in response. The authorization may have expired — try again.")?
        .to_string();

    let refresh_token = json["refresh_token"].as_str().map(|s| s.to_string());

    let cred = credentials::Credential {
        provider: request.provider.clone(),
        token_type: "oauth".to_string(),
        access_token,
        refresh_token,
        expires_at: None,
    };

    credentials::store_credential(&request.provider, &cred)
}

#[tauri::command]
pub async fn test_connection(provider: String) -> Result<String, String> {
    match provider.as_str() {
        "github" => {
            let username = github::verify_connection().await?;
            Ok(format!("Connected as {}", username))
        }
        "cloudflare" => {
            let (name, _id) = cloudflare::verify_connection().await?;
            Ok(format!("Connected to account: {}", name))
        }
        "chatgpt" | "claude" | "gemini" | "openai_compatible" => {
            // Test with a simple message
            let request = llm::ChatRequest {
                provider: provider.clone(),
                messages: vec![llm::ChatMessage {
                    role: "user".to_string(),
                    content: "Say hello in one word.".to_string(),
                }],
                system_prompt: None,
            };
            let response = llm::send_chat(request).await?;
            Ok(format!("Working! AI says: {}", response.content.trim()))
        }
        _ => Err(format!("Unknown provider: {}", provider)),
    }
}

#[tauri::command]
pub async fn get_llm_providers() -> Result<Vec<llm::LlmProvider>, String> {
    Ok(llm::get_available_providers())
}

#[derive(Debug, Deserialize)]
pub struct ChatWithLlmRequest {
    pub provider: String,
    pub messages: Vec<llm::ChatMessage>,
    pub project_context: Option<String>, // JSON with inspirations, current site state, etc.
}

#[tauri::command]
pub async fn chat_with_llm(request: ChatWithLlmRequest) -> Result<llm::ChatResponse, String> {
    let system_prompt = build_designer_system_prompt(request.project_context.as_deref());

    let chat_request = llm::ChatRequest {
        provider: request.provider,
        messages: request.messages,
        system_prompt: Some(system_prompt),
    };

    llm::send_chat(chat_request).await
}

fn build_designer_system_prompt(project_context: Option<&str>) -> String {
    let mut prompt = String::from(
r#"You are a friendly, expert web designer helping someone build their website. Your job is to:

1. Understand what they want through natural conversation
2. Ask the RIGHT questions — they may not know design terms, so keep it simple
3. Generate clean, modern HTML and CSS based on what they describe
4. Iterate based on their feedback

IMPORTANT RULES:
- Never use jargon. Say "the big text at the top" not "the hero heading"
- Ask one question at a time, not a list of 10 things
- When they share a website they like, ask what SPECIFICALLY appeals to them
- Be encouraging — there are no wrong answers
- When generating code, output ONLY valid HTML and CSS
- Use modern CSS (flexbox, grid, custom properties) but keep it readable
- Make sites responsive by default
- Use placeholder images from https://placehold.co/

When you generate HTML/CSS, wrap it in:
```html
(full HTML document here)
```

Start by understanding their goal, not jumping to code. Ask about:
- What's the website for?
- Who will visit it?
- What feeling should visitors get?

Build rapport first, then build the site.
"#);

    if let Some(context) = project_context {
        prompt.push_str("\n\nPROJECT CONTEXT:\n");
        prompt.push_str(context);
    }

    prompt
}

#[tauri::command]
pub async fn scrape_site(url: String) -> Result<scraper::SiteInspiration, String> {
    scraper::scrape_site_inspiration(&url).await
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub github_repo: Option<String>,
    pub cloudflare_project: Option<String>,
    pub live_url: Option<String>,
    pub created_at: u64,
}

#[tauri::command]
pub async fn create_project(request: CreateProjectRequest) -> Result<Project, String> {
    let projects_dir = get_projects_dir()?;
    let project_path = projects_dir.join(&request.name);

    std::fs::create_dir_all(&project_path)
        .map_err(|e| format!("Failed to create project folder: {}", e))?;

    // Create .you-ai metadata folder
    let meta_dir = project_path.join(".you-ai");
    std::fs::create_dir_all(&meta_dir)
        .map_err(|e| format!("Failed to create metadata folder: {}", e))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let project = Project {
        name: request.name.clone(),
        path: project_path.to_string_lossy().to_string(),
        github_repo: None,
        cloudflare_project: None,
        live_url: None,
        created_at: now,
    };

    // Save project metadata
    let meta = serde_json::json!({
        "name": project.name,
        "description": request.description,
        "created_at": now,
        "github_repo": null,
        "cloudflare_project": null,
        "conversation": [],
        "inspirations": [],
    });

    let meta_path = meta_dir.join("project.json");
    std::fs::write(&meta_path, serde_json::to_string_pretty(&meta).unwrap())
        .map_err(|e| format!("Failed to write project metadata: {}", e))?;

    // Create starter index.html
    let starter_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>My Website</title>
    <link rel="stylesheet" href="css/style.css">
</head>
<body>
    <h1>Welcome to your new website!</h1>
    <p>Let's build something together.</p>
</body>
</html>
"#;

    std::fs::write(project_path.join("index.html"), starter_html)
        .map_err(|e| format!("Failed to write index.html: {}", e))?;

    std::fs::create_dir_all(project_path.join("css"))
        .map_err(|e| format!("Failed to create css folder: {}", e))?;

    let starter_css = r#"* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    color: #333;
}
"#;

    std::fs::write(project_path.join("css/style.css"), starter_css)
        .map_err(|e| format!("Failed to write style.css: {}", e))?;

    std::fs::create_dir_all(project_path.join("images"))
        .map_err(|e| format!("Failed to create images folder: {}", e))?;

    Ok(project)
}

#[tauri::command]
pub async fn list_projects() -> Result<Vec<Project>, String> {
    let projects_dir = get_projects_dir()?;

    if !projects_dir.exists() {
        return Ok(Vec::new());
    }

    let mut projects = Vec::new();

    let entries = std::fs::read_dir(&projects_dir)
        .map_err(|e| format!("Failed to read projects directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() { continue; }

        let meta_path = path.join(".you-ai/project.json");
        if meta_path.exists() {
            let content = std::fs::read_to_string(&meta_path).unwrap_or_default();
            let meta: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

            projects.push(Project {
                name: meta["name"].as_str().unwrap_or("Unknown").to_string(),
                path: path.to_string_lossy().to_string(),
                github_repo: meta["github_repo"].as_str().map(|s| s.to_string()),
                cloudflare_project: meta["cloudflare_project"].as_str().map(|s| s.to_string()),
                live_url: None,
                created_at: meta["created_at"].as_u64().unwrap_or(0),
            });
        }
    }

    projects.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(projects)
}

#[tauri::command]
pub async fn open_project(name: String) -> Result<Project, String> {
    let projects_dir = get_projects_dir()?;
    let project_path = projects_dir.join(&name);

    let meta_path = project_path.join(".you-ai/project.json");
    if !meta_path.exists() {
        return Err(format!("Project '{}' not found.", name));
    }

    let content = std::fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read project: {}", e))?;
    let meta: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse project: {}", e))?;

    Ok(Project {
        name: meta["name"].as_str().unwrap_or("Unknown").to_string(),
        path: project_path.to_string_lossy().to_string(),
        github_repo: meta["github_repo"].as_str().map(|s| s.to_string()),
        cloudflare_project: meta["cloudflare_project"].as_str().map(|s| s.to_string()),
        live_url: meta["live_url"].as_str().map(|s| s.to_string()),
        created_at: meta["created_at"].as_u64().unwrap_or(0),
    })
}

#[derive(Debug, Deserialize)]
pub struct PublishRequest {
    pub project_name: String,
    pub commit_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublishResult {
    pub github_url: String,
    pub live_url: Option<String>,
    pub status: String,
}

#[tauri::command]
pub async fn publish_site(request: PublishRequest) -> Result<PublishResult, String> {
    let projects_dir = get_projects_dir()?;
    let project_path = projects_dir.join(&request.project_name);
    let meta_path = project_path.join(".you-ai/project.json");

    let content = std::fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read project: {}", e))?;
    let mut meta: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse project: {}", e))?;

    // Create GitHub repo if it doesn't exist
    let repo_name = if let Some(repo) = meta["github_repo"].as_str() {
        repo.to_string()
    } else {
        let slug = request.project_name.to_lowercase().replace(' ', "-");
        let repo = github::create_repository(
            &slug,
            &format!("Website: {}", request.project_name),
        ).await?;
        meta["github_repo"] = serde_json::json!(repo.full_name);
        std::fs::write(&meta_path, serde_json::to_string_pretty(&meta).unwrap()).ok();
        repo.full_name
    };

    // Collect all site files to push
    let mut files: Vec<(String, String)> = Vec::new();
    collect_site_files(&project_path, &project_path, &mut files)?;

    let commit_msg = request.commit_message
        .unwrap_or_else(|| "Update site via You AI Website Builder".to_string());

    github::push_files(&repo_name, files, &commit_msg).await?;

    // Check if Cloudflare Pages project exists, create if not
    if meta["cloudflare_project"].is_null() {
        if credentials::has_credential("cloudflare") {
            let (_, account_id) = cloudflare::verify_connection().await?;
            let slug = request.project_name.to_lowercase().replace(' ', "-");
            match cloudflare::create_pages_project(&slug, &repo_name, &account_id).await {
                Ok(pages) => {
                    meta["cloudflare_project"] = serde_json::json!(pages.name);
                    meta["live_url"] = serde_json::json!(format!("https://{}.pages.dev", pages.subdomain));
                    std::fs::write(&meta_path, serde_json::to_string_pretty(&meta).unwrap()).ok();
                }
                Err(e) => {
                    // Non-fatal — GitHub push still succeeded
                    eprintln!("Cloudflare Pages setup note: {}", e);
                }
            }
        }
    }

    let live_url = meta["live_url"].as_str().map(|s| s.to_string());

    Ok(PublishResult {
        github_url: format!("https://github.com/{}", repo_name),
        live_url,
        status: "Published! Your site will be live in a few seconds.".to_string(),
    })
}

#[tauri::command]
pub async fn get_site_preview(project_name: String) -> Result<String, String> {
    let projects_dir = get_projects_dir()?;
    let project_path = projects_dir.join(&project_name);
    let index_path = project_path.join("index.html");

    std::fs::read_to_string(&index_path)
        .map_err(|e| format!("Failed to read site preview: {}", e))
}

fn get_projects_dir() -> Result<PathBuf, String> {
    let home = dirs_next::home_dir()
        .ok_or("Cannot determine home directory")?;
    let dir = home.join("You AI Sites");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create sites directory: {}", e))?;
    Ok(dir)
}

fn collect_site_files(
    base: &std::path::Path,
    current: &std::path::Path,
    files: &mut Vec<(String, String)>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(current)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

        // Skip hidden dirs and metadata
        if name.starts_with('.') { continue; }

        if path.is_dir() {
            collect_site_files(base, &path, files)?;
        } else {
            let relative = path.strip_prefix(base)
                .map_err(|_| "Path error")?
                .to_string_lossy()
                .to_string();

            if let Ok(content) = std::fs::read_to_string(&path) {
                files.push((relative, content));
            }
        }
    }

    Ok(())
}
