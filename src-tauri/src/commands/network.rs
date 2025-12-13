use std::io::Write;
use reqwest::Url;
use uuid::Uuid;

#[tauri::command]
pub async fn download_url_to_temp(url: String) -> Result<String, String> {
    // Validate URL
    let parsed_url = Url::parse(&url).map_err(|e| format!("Invalid URL: {}", e))?;

    // Validate scheme
    if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
        return Err("Only HTTP and HTTPS URLs are supported".to_string());
    }

    // Perform request
    let response = reqwest::get(parsed_url)
        .await
        .map_err(|e| format!("Failed to download file: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Server returned error: {}", response.status()));
    }

    let content = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    // Create temp file
    let temp_dir = std::env::temp_dir();
    let file_name = format!("jsonl-viewer-{}.json", Uuid::new_v4());
    let temp_path = temp_dir.join(file_name);

    let mut file = std::fs::File::create(&temp_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    file.write_all(&content)
        .map_err(|e| format!("Failed to write to temp file: {}", e))?;

    Ok(temp_path.to_string_lossy().to_string())
}
