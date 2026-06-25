use crate::credentials;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProvider {
    pub id: String,
    pub name: String,
    pub auth_type: String, // "oauth" | "api_key"
    pub oauth_url: Option<String>,
    pub description: String,
}

pub fn get_available_providers() -> Vec<LlmProvider> {
    vec![
        LlmProvider {
            id: "chatgpt".to_string(),
            name: "ChatGPT".to_string(),
            auth_type: "oauth".to_string(),
            oauth_url: Some("https://auth.openai.com/authorize".to_string()),
            description: "OpenAI's ChatGPT — great for conversational website building".to_string(),
        },
        LlmProvider {
            id: "claude".to_string(),
            name: "Claude".to_string(),
            auth_type: "api_key".to_string(),
            oauth_url: None,
            description: "Anthropic's Claude — excellent at following design instructions"
                .to_string(),
        },
        LlmProvider {
            id: "gemini".to_string(),
            name: "Gemini".to_string(),
            auth_type: "oauth".to_string(),
            oauth_url: Some("https://accounts.google.com/o/oauth2/v2/auth".to_string()),
            description: "Google's Gemini — strong at understanding visual references".to_string(),
        },
        LlmProvider {
            id: "openai_compatible".to_string(),
            name: "Custom (OpenAI-compatible)".to_string(),
            auth_type: "api_key".to_string(),
            oauth_url: None,
            description: "Any OpenAI-compatible API — local models, custom endpoints".to_string(),
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" | "assistant" | "system"
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub provider: String,
    pub messages: Vec<ChatMessage>,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub finish_reason: Option<String>,
}

pub async fn send_chat(request: ChatRequest) -> Result<ChatResponse, String> {
    let credential = credentials::get_credential(&request.provider)?
        .ok_or_else(|| format!("No credential found for provider: {}", request.provider))?;

    match request.provider.as_str() {
        "chatgpt" => send_openai_chat(&credential.access_token, &request).await,
        "claude" => send_claude_chat(&credential.access_token, &request).await,
        "gemini" => send_gemini_chat(&credential.access_token, &request).await,
        "openai_compatible" => send_openai_chat(&credential.access_token, &request).await,
        _ => Err(format!("Unknown provider: {}", request.provider)),
    }
}

async fn send_openai_chat(token: &str, request: &ChatRequest) -> Result<ChatResponse, String> {
    let client = reqwest::Client::new();

    let mut messages: Vec<serde_json::Value> = Vec::new();

    if let Some(system) = &request.system_prompt {
        messages.push(serde_json::json!({
            "role": "system",
            "content": system
        }));
    }

    for msg in &request.messages {
        messages.push(serde_json::json!({
            "role": msg.role,
            "content": msg.content
        }));
    }

    let body = serde_json::json!({
        "model": "gpt-4o",
        "messages": messages,
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    if !status.is_success() {
        return Err(format!("API error ({}): {}", status, text));
    }

    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("Failed to parse response: {}", e))?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let finish_reason = json["choices"][0]["finish_reason"]
        .as_str()
        .map(|s| s.to_string());

    Ok(ChatResponse {
        content,
        finish_reason,
    })
}

async fn send_claude_chat(token: &str, request: &ChatRequest) -> Result<ChatResponse, String> {
    let client = reqwest::Client::new();

    let messages: Vec<serde_json::Value> = request
        .messages
        .iter()
        .map(|msg| {
            serde_json::json!({
                "role": msg.role,
                "content": msg.content
            })
        })
        .collect();

    let mut body = serde_json::json!({
        "model": "claude-sonnet-4-20250514",
        "max_tokens": 4096,
        "messages": messages,
    });

    if let Some(system) = &request.system_prompt {
        body["system"] = serde_json::json!(system);
    }

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", token)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    if !status.is_success() {
        return Err(format!("API error ({}): {}", status, text));
    }

    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("Failed to parse response: {}", e))?;

    let content = json["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let finish_reason = json["stop_reason"].as_str().map(|s| s.to_string());

    Ok(ChatResponse {
        content,
        finish_reason,
    })
}

async fn send_gemini_chat(token: &str, request: &ChatRequest) -> Result<ChatResponse, String> {
    let client = reqwest::Client::new();

    let contents: Vec<serde_json::Value> = request
        .messages
        .iter()
        .map(|msg| {
            let role = if msg.role == "assistant" {
                "model"
            } else {
                "user"
            };
            serde_json::json!({
                "role": role,
                "parts": [{"text": msg.content}]
            })
        })
        .collect();

    let mut body = serde_json::json!({
        "contents": contents,
    });

    if let Some(system) = &request.system_prompt {
        body["systemInstruction"] = serde_json::json!({
            "parts": [{"text": system}]
        });
    }

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
        token
    );

    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    if !status.is_success() {
        return Err(format!("API error ({}): {}", status, text));
    }

    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("Failed to parse response: {}", e))?;

    let content = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(ChatResponse {
        content,
        finish_reason: Some("stop".to_string()),
    })
}
