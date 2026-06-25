use serde::{Deserialize, Serialize};
use crate::credentials;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagesProject {
    pub name: String,
    pub subdomain: String,
    pub domains: Vec<String>,
    pub production_branch: String,
}

/// Create a Cloudflare Pages project connected to a GitHub repo
pub async fn create_pages_project(
    project_name: &str,
    github_repo: &str, // "owner/repo"
    account_id: &str,
) -> Result<PagesProject, String> {
    let credential = credentials::get_credential("cloudflare")?
        .ok_or("Cloudflare not connected. Please connect Cloudflare first.")?;

    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "name": project_name,
        "production_branch": "main",
        "source": {
            "type": "github",
            "config": {
                "owner": github_repo.split('/').next().unwrap_or(""),
                "repo_name": github_repo.split('/').nth(1).unwrap_or(""),
                "production_branch": "main",
                "pr_comments_enabled": true,
                "deployments_enabled": true,
            }
        }
    });

    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/pages/projects",
        account_id
    );

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", credential.access_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("Read failed: {}", e))?;

    if !status.is_success() {
        return Err(format!("Cloudflare API error ({}): {}", status, text));
    }

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("Parse failed: {}", e))?;

    let result = &json["result"];

    Ok(PagesProject {
        name: result["name"].as_str().unwrap_or("").to_string(),
        subdomain: result["subdomain"].as_str().unwrap_or("").to_string(),
        domains: result["domains"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
        production_branch: "main".to_string(),
    })
}

/// Verify Cloudflare token and get account info
pub async fn verify_connection() -> Result<(String, String), String> {
    let credential = credentials::get_credential("cloudflare")?
        .ok_or("Cloudflare not connected.")?;

    let client = reqwest::Client::new();

    // Verify token
    let resp = client
        .get("https://api.cloudflare.com/client/v4/user/tokens/verify")
        .header("Authorization", format!("Bearer {}", credential.access_token))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        return Err("Cloudflare token is invalid or expired.".to_string());
    }

    // Get accounts
    let resp = client
        .get("https://api.cloudflare.com/client/v4/accounts")
        .header("Authorization", format!("Bearer {}", credential.access_token))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let text = resp.text().await.map_err(|e| format!("Read failed: {}", e))?;
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("Parse failed: {}", e))?;

    let account_name = json["result"][0]["name"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();
    let account_id = json["result"][0]["id"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok((account_name, account_id))
}

/// Trigger a deployment (push to GitHub triggers Cloudflare Pages automatically)
/// This is a helper to manually trigger if needed
pub async fn trigger_deployment(
    account_id: &str,
    project_name: &str,
) -> Result<String, String> {
    let credential = credentials::get_credential("cloudflare")?
        .ok_or("Cloudflare not connected.")?;

    let client = reqwest::Client::new();

    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/pages/projects/{}/deployments",
        account_id, project_name
    );

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", credential.access_token))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("Read failed: {}", e))?;

    if !status.is_success() {
        return Err(format!("Deploy failed ({}): {}", status, text));
    }

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("Parse failed: {}", e))?;

    let deploy_url = json["result"]["url"]
        .as_str()
        .unwrap_or("(deploying...)")
        .to_string();

    Ok(deploy_url)
}
