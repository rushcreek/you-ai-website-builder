use crate::credentials;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub default_branch: String,
}

/// Create a new GitHub repository for a site project
pub async fn create_repository(name: &str, description: &str) -> Result<GitHubRepo, String> {
    let credential = credentials::get_credential("github")?
        .ok_or("GitHub not connected. Please connect GitHub first.")?;

    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "name": name,
        "description": description,
        "private": false,
        "auto_init": true,
    });

    let resp = client
        .post("https://api.github.com/user/repos")
        .header(
            "Authorization",
            format!("Bearer {}", credential.access_token),
        )
        .header("User-Agent", "You-AI-Website-Builder/0.1")
        .header("Accept", "application/vnd.github+json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Read failed: {}", e))?;

    if !status.is_success() {
        return Err(format!("GitHub API error ({}): {}", status, text));
    }

    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("Parse failed: {}", e))?;

    Ok(GitHubRepo {
        name: json["name"].as_str().unwrap_or("").to_string(),
        full_name: json["full_name"].as_str().unwrap_or("").to_string(),
        html_url: json["html_url"].as_str().unwrap_or("").to_string(),
        default_branch: json["default_branch"]
            .as_str()
            .unwrap_or("main")
            .to_string(),
    })
}

/// Push files to a GitHub repository (using the Contents API for simplicity)
pub async fn push_files(
    repo_full_name: &str,
    files: Vec<(String, String)>, // (path, content)
    commit_message: &str,
) -> Result<(), String> {
    let credential = credentials::get_credential("github")?.ok_or("GitHub not connected.")?;

    let client = reqwest::Client::new();

    for (path, content) in files {
        let encoded = base64_encode(&content);

        // Check if file exists (to get SHA for updates)
        let get_url = format!(
            "https://api.github.com/repos/{}/contents/{}",
            repo_full_name, path
        );

        let existing = client
            .get(&get_url)
            .header(
                "Authorization",
                format!("Bearer {}", credential.access_token),
            )
            .header("User-Agent", "You-AI-Website-Builder/0.1")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let sha = if existing.status().is_success() {
            let text = existing.text().await.unwrap_or_default();
            let json: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
            json["sha"].as_str().map(|s| s.to_string())
        } else {
            None
        };

        let mut body = serde_json::json!({
            "message": commit_message,
            "content": encoded,
        });

        if let Some(sha) = sha {
            body["sha"] = serde_json::json!(sha);
        }

        let put_url = format!(
            "https://api.github.com/repos/{}/contents/{}",
            repo_full_name, path
        );

        let resp = client
            .put(&put_url)
            .header(
                "Authorization",
                format!("Bearer {}", credential.access_token),
            )
            .header("User-Agent", "You-AI-Website-Builder/0.1")
            .header("Accept", "application/vnd.github+json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Push failed for {}: {}", path, e))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Failed to push {}: {}", path, text));
        }
    }

    Ok(())
}

/// Verify GitHub token works
pub async fn verify_connection() -> Result<String, String> {
    let credential = credentials::get_credential("github")?.ok_or("GitHub not connected.")?;

    let client = reqwest::Client::new();

    let resp = client
        .get("https://api.github.com/user")
        .header(
            "Authorization",
            format!("Bearer {}", credential.access_token),
        )
        .header("User-Agent", "You-AI-Website-Builder/0.1")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Read failed: {}", e))?;

    if !status.is_success() {
        return Err(format!("GitHub connection failed ({}): {}", status, text));
    }

    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("Parse failed: {}", e))?;

    Ok(json["login"].as_str().unwrap_or("unknown").to_string())
}

fn base64_encode(input: &str) -> String {
    use std::io::Write;
    let mut buf = Vec::new();
    let mut encoder = Base64Encoder::new(&mut buf);
    encoder.write_all(input.as_bytes()).unwrap();
    drop(encoder);
    String::from_utf8(buf).unwrap()
}

// Simple base64 encoder (no external dep needed)
struct Base64Encoder<'a> {
    output: &'a mut Vec<u8>,
    buffer: [u8; 3],
    buffer_len: usize,
}

impl<'a> Base64Encoder<'a> {
    fn new(output: &'a mut Vec<u8>) -> Self {
        Self {
            output,
            buffer: [0; 3],
            buffer_len: 0,
        }
    }

    fn flush_buffer(&mut self) {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        if self.buffer_len == 0 {
            return;
        }

        let b0 = self.buffer[0] as usize;
        let b1 = if self.buffer_len > 1 {
            self.buffer[1] as usize
        } else {
            0
        };
        let b2 = if self.buffer_len > 2 {
            self.buffer[2] as usize
        } else {
            0
        };

        self.output.push(CHARS[b0 >> 2]);
        self.output.push(CHARS[((b0 & 0x03) << 4) | (b1 >> 4)]);

        if self.buffer_len > 1 {
            self.output.push(CHARS[((b1 & 0x0f) << 2) | (b2 >> 6)]);
        } else {
            self.output.push(b'=');
        }

        if self.buffer_len > 2 {
            self.output.push(CHARS[b2 & 0x3f]);
        } else {
            self.output.push(b'=');
        }

        self.buffer_len = 0;
    }
}

impl<'a> std::io::Write for Base64Encoder<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for &byte in buf {
            self.buffer[self.buffer_len] = byte;
            self.buffer_len += 1;
            if self.buffer_len == 3 {
                self.flush_buffer();
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> Drop for Base64Encoder<'a> {
    fn drop(&mut self) {
        if self.buffer_len > 0 {
            self.flush_buffer();
        }
    }
}
