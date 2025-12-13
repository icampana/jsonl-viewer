use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::ipc::Channel;
use tokio::io::AsyncBufReadExt;
use regex::Regex;
use jsonpath_rust::JsonPathFinder;
// use std::str::FromStr;
use crate::commands::file_parser::FileFormat;
use std::str::FromStr;

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
    file_format: FileFormat,
    channel: Channel<Vec<SearchResult>>,
) -> Result<SearchStats, String> {
    let file_path = PathBuf::from(&path);

    // Common search logic helper
    let check_match = |line_str: &str, json_val: Option<&serde_json::Value>| -> Option<Vec<String>> {
        let mut collected_matches = Vec::new();

        // Scenario A: Text Search ONLY
        if query.text.is_some() && query.json_path.is_none() {
            let text = query.text.as_ref().unwrap();
            let found = if query.regex {
                 let regex_pattern = if query.case_sensitive {
                    Regex::new(text)
                } else {
                    Regex::new(&format!("(?i){}", text))
                };
                 if let Ok(regex) = regex_pattern {
                    regex.find_iter(line_str).map(|m| m.as_str().to_string()).collect::<Vec<_>>()
                 } else {
                     vec![]
                 }
            } else {
                let search_line = if query.case_sensitive { line_str.to_string() } else { line_str.to_lowercase() };
                let search_text = if query.case_sensitive { text.clone() } else { text.to_lowercase() };
                if search_line.contains(&search_text) {
                    vec![text.clone()]
                } else {
                    vec![]
                }
            };

            if !found.is_empty() {
                return Some(found);
            }
            return None;
        }

        // Scenario B: JSONPath Search (with optional Text Filter on results)
        if let Some(ref json_path) = query.json_path {
            // We need a JSON value to query against
             let val_to_check = if let Some(v) = json_val {
                 Some(v.clone())
             } else {
                 serde_json::from_str::<serde_json::Value>(line_str).ok()
             };

            if let Some(v) = val_to_check {
                if let Some(mut json_matches) = evaluate_jsonpath(&v, json_path) {
                    // Start with all JSONPath matches
                    // If there is NO text query, we accept all these matches.
                    // If there IS a text query, we filter these matches.

                    if let Some(ref text) = query.text {
                         // Filter the JSONPath results: keep only those containing the text
                         let filtered_matches: Vec<String> = json_matches.into_iter().filter(|result_str| {
                             if query.regex {
                                 let regex_pattern = if query.case_sensitive {
                                    Regex::new(text)
                                } else {
                                    Regex::new(&format!("(?i){}", text))
                                };
                                 if let Ok(regex) = regex_pattern {
                                    regex.is_match(result_str)
                                 } else {
                                     false
                                 }
                             } else {
                                let target = if query.case_sensitive { result_str.clone() } else { result_str.to_lowercase() };
                                let query_text = if query.case_sensitive { text.clone() } else { text.to_lowercase() };
                                target.contains(&query_text)
                             }
                         }).collect();

                         if !filtered_matches.is_empty() {
                             // Return the text matches found within the JSONPath results?
                             // Or return the JSONPath values themselves?
                             // Usually highlighting the "match" (the found text) is correct for search.
                             // But the user might want to see the *values*.
                             // Let's return the full field value (filtered_matches) as the "match".
                             collected_matches.extend(filtered_matches);
                         }
                    } else {
                         // No text filter, just return the JSONPath matches
                         collected_matches.extend(json_matches);
                    }
                }
            }
        }

        if !collected_matches.is_empty() {
            Some(collected_matches)
        } else {
            None
        }
    };

    // Handle JsonArray (pretty printed or single line) separately
    if matches!(file_format, FileFormat::JsonArray) {
        let content = tokio::fs::read_to_string(&file_path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let mut total_matches = 0;
        let mut lines_searched = 0;
        const CHUNK_SIZE: usize = 100;
        let mut chunk: Vec<SearchResult> = Vec::with_capacity(CHUNK_SIZE);

        if let Some(array) = json.as_array() {
            for (index, item) in array.iter().enumerate() {
                lines_searched += 1;
                let line_str = serde_json::to_string(item).unwrap_or_default();

                if let Some(matches) = check_match(&line_str, Some(item)) {
                     let result = SearchResult {
                        line_id: index,
                        matches,
                        context: line_str,
                    };
                    chunk.push(result);
                    if chunk.len() >= CHUNK_SIZE {
                         channel.send(chunk.clone()).map_err(|e| format!("Failed to send: {}", e))?;
                         chunk.clear();
                    }
                    total_matches += 1;
                }
            }
        }

        if !chunk.is_empty() {
            channel.send(chunk).map_err(|e| format!("Failed to send: {}", e))?;
        }

        return Ok(SearchStats { total_matches, lines_searched });
    }

    // Default JsonL
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
         if let Some(matches) = check_match(&line, None) {
             let result = SearchResult {
                line_id: line_num,
                matches,
                context: line.clone(),
            };
            chunk.push(result);
            if chunk.len() >= CHUNK_SIZE {
                channel.send(chunk.clone()).map_err(|e| format!("Failed to send: {}", e))?;
                chunk.clear();
            }
            total_matches += 1;
        }
        line_num += 1;
    }

    if !chunk.is_empty() {
        channel.send(chunk).map_err(|e| format!("Failed to send: {}", e))?;
    }

    Ok(SearchStats { total_matches, lines_searched: line_num })
}

fn evaluate_jsonpath(json: &serde_json::Value, path: &str) -> Option<Vec<String>> {
    let json_str = serde_json::to_string(json).ok()?;

    match JsonPathFinder::from_str(&json_str, path) {
        Ok(finder) => {
            let matches = finder.find();
             if let Some(array) = matches.as_array() {
                 let results: Vec<String> = array.iter()
                    .map(|v| {
                        if let Some(s) = v.as_str() {
                            s.to_string()
                        } else {
                            v.to_string()
                        }
                    })
                    .collect();

                 if !results.is_empty() {
                     Some(results)
                 } else {
                     None
                 }
             } else {
                 None
             }
        }
        Err(_) => None
    }
}