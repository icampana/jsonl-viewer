use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::ipc::Channel;
use tokio::io::AsyncBufReadExt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonLine {
    pub id: usize,
    pub content: String,
    pub parsed: serde_json::Value,
    pub byte_offset: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub total_lines: usize,
    pub file_size: u64,
    pub format: FileFormat,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FileFormat {
    JsonL,
    JsonArray,
}



#[tauri::command]
pub async fn parse_file_streaming(
    path: String,
    channel: Channel<Vec<JsonLine>>,
) -> Result<FileMetadata, String> {
    let file_path = PathBuf::from(&path);
    let file_size = tokio::fs::metadata(&file_path)
        .await
        .map_err(|e| format!("Failed to read file metadata: {}", e))?
        .len();

    let file = tokio::fs::File::open(&file_path)
        .await
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let reader = tokio::io::BufReader::new(file);
    let mut lines = tokio::io::BufReader::new(reader).lines();

    let mut line_num = 0;
    let mut byte_offset = 0;
    // Default format
    let mut format = FileFormat::JsonL;

    // Determine strict mode based on extension
    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let is_strict_jsonl = extension == "jsonl" || extension == "ndjson";

    const CHUNK_SIZE: usize = 2000;
    let mut chunk: Vec<JsonLine> = Vec::with_capacity(CHUNK_SIZE);

    // Read first line to determine format / content check
    if let Ok(Some(first_line)) = lines.next_line().await {
        let trimmed = first_line.trim();

        // Check if we should treat this as a JSON Array (Explode Mode)
        // Only if NOT strict jsonl AND starts with [
        if !is_strict_jsonl && trimmed.starts_with("[") {
             // Handle JSON array format - Legacy "Explode" Behavior for standard .json files
             // We attempt to parse the FIRST line as a full array.
             // Note: correctly supporting multi-line JSON arrays streaming is complex,
             // this existing logic assumes the array is on the first line or we parse the first valid line as an array.
             // If the file is a massive multi-line JSON array, this simple `from_str` on `first_line` might fail if it's not all on one line.
             // However, for the scope of "fixing the bug", we are preserving existing behavior for .json
             // while fixing it for .jsonl.
            if let Ok(json_array) = serde_json::from_str::<serde_json::Value>(&first_line) {
                if let Some(array) = json_array.as_array() {
                    format = FileFormat::JsonArray;
                    for (index, item) in array.iter().enumerate() {
                        let json_line = JsonLine {
                            id: index,
                            content: serde_json::to_string(item).unwrap_or_default(),
                            parsed: item.clone(),
                            byte_offset: byte_offset,
                        };

                        chunk.push(json_line);
                        if chunk.len() >= CHUNK_SIZE {
                            channel.send(chunk.clone()).map_err(|e| format!("Failed to send data: {}", e))?;
                            chunk.clear();
                        }
                        line_num += 1;
                    }
                    // After successfully processing the array on the first line, we assume the file logic ends here
                    // for "JsonArray" type files as per previous logic.
                }
            } else {
                 // It started with [, but wasn't a valid single-line array.
                 // Fallback to strict line processing (could be a parser error later on individual lines).
                 // We treat it as a normal line.
                 process_single_line(&first_line, line_num, byte_offset, &mut chunk, &channel)?;
                 line_num += 1;
            }
        } else {
            // Strict JSONL or generic object handling
            // Process first line
            process_single_line(&first_line, line_num, byte_offset, &mut chunk, &channel)?;
            line_num += 1;
        }

        // Always continue reading remaining lines UNLESS we successfully processed a generic JsonArray above
        // (which we can detect if format changed to JsonArray)
        // If format is JsonArray from the block above, we stop (as it was a single atomic unit)
        // If format is still JsonL, we continue.

        if matches!(format, FileFormat::JsonL) {
             while let Ok(Some(line)) = lines.next_line().await {
                byte_offset += (line.len() as u64) + 1; // +1 for newline

                process_single_line(&line, line_num, byte_offset, &mut chunk, &channel)?;
                line_num += 1;
            }
        }
    }

    // Send remaining items
    if !chunk.is_empty() {
        channel.send(chunk).map_err(|e| format!("Failed to send data: {}", e))?;
    }

    Ok(FileMetadata {
        path,
        total_lines: line_num,
        file_size,
        format,
    })
}

// Helper to deduce duplicate logic
fn process_single_line(
    line: &str,
    id: usize,
    byte_offset: u64,
    chunk: &mut Vec<JsonLine>,
    channel: &Channel<Vec<JsonLine>>
) -> Result<(), String> {
    if line.trim().is_empty() {
        return Ok(());
    }

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
        let json_line = JsonLine {
            id,
            content: line.to_string(),
            parsed: json,
            byte_offset,
        };

        chunk.push(json_line);
        if chunk.len() >= 2000 { // CHUNK_SIZE
            channel.send(chunk.clone()).map_err(|e| format!("Failed to send data: {}", e))?;
            chunk.clear();
        }
    }
    Ok(())
}