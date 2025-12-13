use crate::commands::search::SearchQuery;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, HashMap};
use std::path::PathBuf;
use tokio::io::AsyncBufReadExt;
use rust_xlsxwriter::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportFilter {
    pub line_ids: Option<Vec<usize>>,
    pub search_query: Option<SearchQuery>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportStats {
    pub lines_exported: usize,
    pub file_size: u64,
}

#[tauri::command]
pub async fn export_to_csv(
    path: String,
    _filter: ExportFilter,
    output_path: String,
) -> Result<ExportStats, String> {
    let file_path = PathBuf::from(&path);
    let file = tokio::fs::File::open(&file_path)
        .await
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let reader = tokio::io::BufReader::new(file);
    let mut lines = reader.lines();

    // Collect headers (scan first 1000 lines for better coverage)
    let mut headers_set = HashSet::new();
    let mut sample_lines = Vec::new();

    // Buffer first 1000 lines for header detection
    for _ in 0..1000 {
        match lines.next_line().await {
            Ok(Some(line)) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                    collect_headers(&json, "", &mut headers_set);
                    sample_lines.push((line, json));
                }
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }

    let mut headers: Vec<String> = headers_set.into_iter().collect();
    headers.sort();

    // Use CSV crate for valid output
    let mut wtr = csv::Writer::from_path(&output_path)
        .map_err(|e| format!("Failed to create CSV writer: {}", e))?;

    // Write header
    wtr.write_record(&headers)
        .map_err(|e| format!("Failed to write CSV headers: {}", e))?;

    let mut lines_exported = 0;

    // Process sample lines
    for (_raw, json) in &sample_lines {
        let record: Vec<String> = headers.iter()
            .map(|h| get_flat_value(json, h))
            .collect();
        wtr.write_record(&record)
            .map_err(|e| format!("Failed to write CSV record: {}", e))?;
        lines_exported += 1;
    }

    // Process remaining
    while let Ok(Some(line)) = lines.next_line().await {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
            let record: Vec<String> = headers.iter()
                .map(|h| get_flat_value(&json, h))
                .collect();
            wtr.write_record(&record)
                .map_err(|e| format!("Failed to write CSV record: {}", e))?;
            lines_exported += 1;
        }
    }

    wtr.flush().map_err(|e| format!("Failed to flush CSV: {}", e))?;

    let file_size = tokio::fs::metadata(&output_path)
        .await
        .map_err(|e| format!("Failed to get metadata: {}", e))?
        .len();

    Ok(ExportStats {
        lines_exported,
        file_size
    })
}

#[tauri::command]
pub async fn export_to_excel(
    path: String,
    _filter: ExportFilter,
    output_path: String,
) -> Result<ExportStats, String> {
    let file_path = PathBuf::from(&path);
    let file = tokio::fs::File::open(&file_path)
        .await
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let reader = tokio::io::BufReader::new(file);
    let mut lines = reader.lines();

    // Collect headers (scan first 1000 lines)
    let mut headers_set = HashSet::new();
    let mut sample_lines = Vec::new();

    for _ in 0..1000 {
        match lines.next_line().await {
            Ok(Some(line)) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                    collect_headers(&json, "", &mut headers_set);
                    sample_lines.push((line, json));
                }
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }

    let mut headers: Vec<String> = headers_set.into_iter().collect();
    headers.sort();

    // EXCEL Setup
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Formats
    let header_format = Format::new()
        .set_bold()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_background_color(Color::Silver);

    let subheader_format = Format::new()
        .set_bold()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Left)
        .set_background_color(Color::Gray);

    // Group headers logic
    let mut current_top_key: Option<String> = None;
    let mut start_col: u16 = 0;

    for (i, header) in headers.iter().enumerate() {
        let parts: Vec<&str> = header.splitn(2, '_').collect();
        let top_key = if parts.len() > 1 { parts[0].to_string() } else { "".to_string() };

        let is_last = i == headers.len() - 1;
        let col_idx = i as u16;

        if let Some(ref current) = current_top_key {
            if top_key != *current {
                // End of previous group
                let end_col = col_idx - 1;

                // Write Header for previous group
                if current.is_empty() {
                    // No grouping, just write full headers in row 1??
                    // Or merge vertically? Let's merge vertically R1:R2 for root keys
                     for c in start_col..=end_col {
                        worksheet.merge_range(0, c, 1, c, &headers[c as usize], &header_format)
                             .map_err(|e| e.to_string())?;
                     }
                } else {
                    // Group header
                    worksheet.merge_range(0, start_col, 0, end_col, current, &header_format)
                        .map_err(|e| e.to_string())?;
                    // Sub headers
                    for c in start_col..=end_col {
                        let sub = headers[c as usize].trim_start_matches(&format!("{}_", current));
                        worksheet.write_string_with_format(1, c, sub, &subheader_format).map_err(|e| e.to_string())?;
                    }
                }

                start_col = col_idx;
                current_top_key = Some(top_key.clone());
            }
        } else {
            current_top_key = Some(top_key.clone());
        }

        if is_last {
            // Write last group
            if let Some(ref current) = current_top_key {
                let end_col = col_idx;
                if current.is_empty() {
                    for c in start_col..=end_col {
                        worksheet.merge_range(0, c, 1, c, &headers[c as usize], &header_format)
                             .map_err(|e| e.to_string())?;
                    }
                } else {
                     worksheet.merge_range(0, start_col, 0, end_col, current, &header_format)
                        .map_err(|e| e.to_string())?;
                     for c in start_col..=end_col {
                        let sub = headers[c as usize].trim_start_matches(&format!("{}_", current));
                        worksheet.write_string_with_format(1, c, sub, &subheader_format).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }

    // Write Data (Row index starts at 2)
    let mut row_idx = 2;

    for (_raw, json) in &sample_lines {
        for (col_idx, header) in headers.iter().enumerate() {
            let val = get_flat_value(json, header);
            worksheet.write_string(row_idx, col_idx as u16, &val)
                .map_err(|e| e.to_string())?;
        }
        row_idx += 1;
    }

    while let Ok(Some(line)) = lines.next_line().await {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
             for (col_idx, header) in headers.iter().enumerate() {
                let val = get_flat_value(&json, header);
                worksheet.write_string(row_idx, col_idx as u16, &val)
                    .map_err(|e| e.to_string())?;
            }
            row_idx += 1;
        }
    }

    workbook.save(&output_path).map_err(|e| e.to_string())?;

    let file_size = tokio::fs::metadata(&output_path)
        .await
        .map_err(|e| format!("Failed to get metadata: {}", e))?
        .len();

    Ok(ExportStats {
        lines_exported: row_idx as usize - 2,
        file_size
    })
}


// Shared Utils
#[allow(dead_code)]
fn collect_headers(json: &serde_json::Value, prefix: &str, headers: &mut HashSet<String>) {
    match json {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}_{}", prefix, key)
                };
                match value {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        collect_headers(value, &new_prefix, headers);
                    }
                    _ => {
                        headers.insert(new_prefix);
                    }
                }
            }
        }
        serde_json::Value::Array(arr) => {
             for (index, item) in arr.iter().enumerate() {
                let new_prefix = format!("{}_{}", prefix, index);
                match item {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        collect_headers(item, &new_prefix, headers);
                    }
                    _ => {
                        headers.insert(new_prefix);
                    }
                }
            }
        }
        _ => {
             if !prefix.is_empty() { headers.insert(prefix.to_string()); }
        }
    }
}

#[allow(dead_code)]
fn get_flat_value(json: &serde_json::Value, path: &str) -> String {
    let parts: Vec<&str> = path.split('_').collect();
    let mut current = json;

    for part in parts {
        if let Ok(index) = part.parse::<usize>() {
            if let Some(arr) = current.as_array() {
                if let Some(item) = arr.get(index) {
                    current = item;
                } else { return "".to_string(); }
            } else { return "".to_string(); }
        } else {
             if let Some(obj) = current.as_object() {
                if let Some(val) = obj.get(part) {
                    current = val;
                } else { return "".to_string(); }
            } else { return "".to_string(); }
        }
    }

    match current {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => "".to_string(),
        v => v.to_string()
    }
}