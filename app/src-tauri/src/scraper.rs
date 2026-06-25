use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteInspiration {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub colors: Vec<String>,
    pub fonts: Vec<String>,
    pub layout_type: Option<String>, // "minimal", "dense", "magazine", "portfolio", etc.
    pub screenshot_path: Option<String>,
}

/// Fetch a website and extract design signals (colors, fonts, layout hints)
pub async fn scrape_site_inspiration(url: &str) -> Result<SiteInspiration, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; YouAI/0.1; +https://youai.dev)")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Client error: {}", e))?;

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch {}: {}", url, e))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("Site returned status {}", status));
    }

    let html = resp.text().await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Extract title
    let title = extract_between(&html, "<title>", "</title>")
        .map(|s| s.trim().to_string());

    // Extract meta description
    let description = extract_meta_content(&html, "description");

    // Extract colors from inline styles and CSS
    let colors = extract_colors(&html);

    // Extract font families
    let fonts = extract_fonts(&html);

    // Guess layout type from structure
    let layout_type = guess_layout_type(&html);

    Ok(SiteInspiration {
        url: url.to_string(),
        title,
        description,
        colors,
        fonts,
        layout_type,
        screenshot_path: None, // Screenshots handled by frontend/browser tool later
    })
}

fn extract_between(html: &str, start: &str, end: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start_lower = start.to_lowercase();
    let end_lower = end.to_lowercase();

    let start_idx = lower.find(&start_lower)? + start.len();
    let end_idx = lower[start_idx..].find(&end_lower)? + start_idx;

    Some(html[start_idx..end_idx].to_string())
}

fn extract_meta_content(html: &str, name: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let pattern = format!("name=\"{}\"", name);
    let idx = lower.find(&pattern)?;

    // Look for content= near this meta tag
    let region = &html[idx.saturating_sub(100)..std::cmp::min(idx + 300, html.len())];
    let content_idx = region.to_lowercase().find("content=\"")?;
    let start = content_idx + 9;
    let end = region[start..].find('"')? + start;

    Some(region[start..end].to_string())
}

fn extract_colors(html: &str) -> Vec<String> {
    let mut colors = Vec::new();
    let hex_pattern = regex_lite_find_hex(html);
    for color in hex_pattern.into_iter().take(8) {
        if !colors.contains(&color) {
            colors.push(color);
        }
    }
    colors
}

fn regex_lite_find_hex(text: &str) -> Vec<String> {
    let mut results = Vec::new();
    let bytes = text.as_bytes();

    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'#' {
            // Check for 6-digit hex
            if i + 7 <= bytes.len() {
                let candidate = &text[i..i + 7];
                if candidate[1..].chars().all(|c| c.is_ascii_hexdigit()) {
                    results.push(candidate.to_lowercase());
                    i += 7;
                    continue;
                }
            }
            // Check for 3-digit hex
            if i + 4 <= bytes.len() {
                let candidate = &text[i..i + 4];
                if candidate[1..].chars().all(|c| c.is_ascii_hexdigit()) {
                    results.push(candidate.to_lowercase());
                    i += 4;
                    continue;
                }
            }
        }
        i += 1;
    }

    results
}

fn extract_fonts(html: &str) -> Vec<String> {
    let mut fonts = Vec::new();
    let lower = html.to_lowercase();

    // Look for font-family declarations
    let mut search_from = 0;
    while let Some(idx) = lower[search_from..].find("font-family") {
        let abs_idx = search_from + idx;
        let region = &html[abs_idx..std::cmp::min(abs_idx + 200, html.len())];

        if let Some(colon) = region.find(':') {
            let after_colon = &region[colon + 1..];
            if let Some(end) = after_colon.find(|c: char| c == ';' || c == '}' || c == '"') {
                let font_str = after_colon[..end].trim();
                // Take first font in the stack
                let first_font = font_str.split(',').next().unwrap_or("")
                    .trim()
                    .trim_matches('\'')
                    .trim_matches('"')
                    .to_string();

                if !first_font.is_empty()
                    && !fonts.contains(&first_font)
                    && !["inherit", "initial", "unset", "sans-serif", "serif", "monospace"]
                        .contains(&first_font.to_lowercase().as_str())
                {
                    fonts.push(first_font);
                }
            }
        }

        search_from = abs_idx + 12;
        if fonts.len() >= 5 { break; }
    }

    fonts
}

fn guess_layout_type(html: &str) -> Option<String> {
    let lower = html.to_lowercase();

    let has_grid = lower.contains("display: grid") || lower.contains("display:grid");
    let has_hero = lower.contains("hero") || lower.contains("banner");
    let has_cards = lower.contains("card") && lower.matches("card").count() > 3;
    let has_sidebar = lower.contains("sidebar") || lower.contains("aside");
    let has_gallery = lower.contains("gallery") || lower.contains("portfolio");

    if has_gallery {
        Some("portfolio".to_string())
    } else if has_cards && has_grid {
        Some("grid-cards".to_string())
    } else if has_hero && !has_sidebar {
        Some("minimal-hero".to_string())
    } else if has_sidebar {
        Some("content-sidebar".to_string())
    } else if has_hero {
        Some("standard".to_string())
    } else {
        Some("simple".to_string())
    }
}
