use keyring::Entry;
use serde::{Deserialize, Serialize};

const SERVICE_NAME: &str = "you-ai-website-builder";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub provider: String,
    pub token_type: String, // "oauth" | "api_key"
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
}

pub fn store_credential(provider: &str, credential: &Credential) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, provider)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
    let json = serde_json::to_string(credential)
        .map_err(|e| format!("Failed to serialize credential: {}", e))?;
    entry.set_password(&json)
        .map_err(|e| format!("Failed to store credential: {}", e))?;
    Ok(())
}

pub fn get_credential(provider: &str) -> Result<Option<Credential>, String> {
    let entry = Entry::new(SERVICE_NAME, provider)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
    match entry.get_password() {
        Ok(json) => {
            let cred: Credential = serde_json::from_str(&json)
                .map_err(|e| format!("Failed to deserialize credential: {}", e))?;
            Ok(Some(cred))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to get credential: {}", e)),
    }
}

pub fn remove_credential(provider: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, provider)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Failed to remove credential: {}", e)),
    }
}

pub fn has_credential(provider: &str) -> bool {
    get_credential(provider).ok().flatten().is_some()
}
