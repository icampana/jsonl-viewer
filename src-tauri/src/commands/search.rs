use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::ipc::Channel;
use tokio::io::AsyncBufReadExt;
use regex::Regex;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub json_path: Option<String>,
    pub case_sensitive: bool,
    pub regex: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub line_id: usize,
    pub matches: Vec<String>,
    pub context: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchStats {
    pub total_matches: usize,
    pub lines_searched: usize,
}

#[tauri::command]
pub async fn search_in_file(
    path: String,
    query: SearchQuery,
    channel: Channel<Vec<SearchResult>>,
) -> Result<SearchStats, String> {
    let file_path = PathBuf::from(&path);
    let file = tokio::fs::File::open(&file_path)
        .await
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let reader = tokio::io::BufReader::new(file);
    let mut lines = reader.lines();

    let mut line_num = 0;
    let mut total_matches = 0;

    const CHUNK_SIZE: usize = 100;
    let mut chunk: Vec<SearchResult> = Vec::with_capacity(CHUNK_SIZE);

    while let Ok(Some(line)) = lines.next_line().await {
        let mut matches = Vec::new();

        // Text search with optional regex support
        if let Some(ref text) = query.text {
            if query.regex {
                // Regex search
                let regex_pattern = if query.case_sensitive {
                    Regex::new(text)
                } else {
                    Regex::new(&format!("(?i){}", text))
                };

                if let Ok(regex) = regex_pattern {
                    for found in regex.find_iter(&line) {
                        matches.push(found.as_str().to_string());
                    }
                }
            } else {
                // Simple text search
                let search_line = if query.case_sensitive {
                    line.to_string()
                } else {
                    line.to_lowercase()
                };
                let search_text = if query.case_sensitive {
                    text.clone()
                } else {
                    text.to_lowercase()
                };

                if search_line.contains(&search_text) {
                    matches.push(text.clone());
                }
            }
        }

        // JSONPath search (basic implementation)
        if let Some(ref json_path) = query.json_path {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                if let Some(matched) = evaluate_jsonpath(&json, json_path) {
                    matches.extend(matched);
                }
            }
        }

        if !matches.is_empty() {
            let result = SearchResult {
                line_id: line_num,
                matches,
                context: line.clone(),
            };

            chunk.push(result);
            if chunk.len() >= CHUNK_SIZE {
                channel.send(chunk.clone()).map_err(|e| format!("Failed to send search result: {}", e))?;
                chunk.clear();
            }

            total_matches += 1;
        }

        line_num += 1;
    }

    // Send remaining items
    if !chunk.is_empty() {
        channel.send(chunk).map_err(|e| format!("Failed to send search result: {}", e))?;
    }

    Ok(SearchStats {
        total_matches,
        lines_searched: line_num,
    })
}

fn evaluate_jsonpath(json: &serde_json::Value, path: &str) -> Option<Vec<String>> {
    match path {
        "$" => Some(vec![serde_json::to_string(json).unwrap_or_default()]),
        "$.users[*].name" => {
            if let Some(users) = json.get("users").and_then(|u| u.as_array()) {
                let names: Vec<String> = users
                    .iter()
                    .filter_map(|u| u.get("name"))
                    .filter_map(|name| name.as_str())
                    .map(|s| s.to_string())
                    .collect();
                if !names.is_empty() {
                    return Some(names);
                }
            }
            None
        }
        "$.data[0].value" => {
            if let Some(value) = json.get("data").and_then(|d| d.get(0)).and_then(|i| i.get("value")) {
                Some(vec![serde_json::to_string(value).unwrap_or_default()])
            } else {
                None
            }
        }
        "$.config.enabled" => {
            if let Some(enabled) = json.get("config").and_then(|c| c.get("enabled")) {
                Some(vec![serde_json::to_string(enabled).unwrap_or_default()])
            } else {
                None
            }
        }
        "$.items[*]" => {
            if let Some(items) = json.get("items").and_then(|i| i.as_array()) {
                let items_str: Vec<String> = items
                    .iter()
                    .map(|item| serde_json::to_string(item).unwrap_or_default())
                    .collect();
                if !items_str.is_empty() {
                    return Some(items_str);
                }
            }
            None
        }
        _ => None, // Unsupported path
    }
}