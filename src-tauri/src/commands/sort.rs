use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::ipc::Channel;
use tokio::io::AsyncBufReadExt;
use crate::commands::file_parser::{JsonLine, FileFormat};
use crate::commands::search::SearchResult;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SortColumn {
	pub column: String,
	pub direction: String,
}

/// Sortable value representation for comparison
enum SortValue {
	Null,
	Number(f64),
	Date(i64),  // Unix timestamp in seconds
	String(String),
}

/// Extract value from nested JSON by underscore-separated path
fn get_nested_value(json: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
	let parts: Vec<&str> = path.split('_').collect();
	let mut current = json;

	for part in parts {
		match current {
			serde_json::Value::Object(obj) => {
				current = obj.get(part)?;
			}
			_ => return None,
		}
	}

	Some(current.clone())
}

/// Convert JSON value to sortable representation
fn to_sort_value(val: &serde_json::Value) -> SortValue {
	match val {
		serde_json::Value::Null => SortValue::Null,
		serde_json::Value::Bool(b) => SortValue::Number(if *b { 1.0 } else { 0.0 }),
		serde_json::Value::Number(n) => {
			if let Some(f) = n.as_f64() {
				SortValue::Number(f)
			} else if let Some(i) = n.as_i64() {
				SortValue::Number(i as f64)
			} else {
				SortValue::String(n.to_string())
			}
		}
		serde_json::Value::String(s) => {
			// Try to parse as ISO 8601 date first
			if let Ok(timestamp) = parse_iso_date(s) {
				return SortValue::Date(timestamp);
			}
			// Try to parse as number
			if let Ok(n) = s.parse::<f64>() {
				return SortValue::Number(n);
			}
			SortValue::String(s.clone())
		}
		serde_json::Value::Array(_) => SortValue::String("[Array]".to_string()),
		serde_json::Value::Object(_) => SortValue::String("[Object]".to_string()),
	}
}

/// Parse ISO 8601 date string to Unix timestamp
fn parse_iso_date(s: &str) -> Result<i64, ()> {
	// Formats with timezone (Z suffix)
	const TIMEZONE_FORMATS: &[&str] = &[
		"%Y-%m-%dT%H:%M:%S%.fZ",      // 2024-01-15T10:30:00.123Z
		"%Y-%m-%dT%H:%M:%SZ",          // 2024-01-15T10:30:00Z
	];

	// Formats without timezone (treated as UTC)
	const NAIVE_FORMATS: &[&str] = &[
		"%Y-%m-%d %H:%M:%S%.f",        // 2024-01-15 10:30:00.123
		"%Y-%m-%d %H:%M:%S",           // 2024-01-15 10:30:00
		"%Y-%m-%d",                    // 2024-01-15
	];

	// Try timezone-aware formats first
	for fmt in TIMEZONE_FORMATS {
		if let Ok(dt) = chrono::DateTime::parse_from_str(s, fmt) {
			return Ok(dt.timestamp());
		}
	}

	// Try timezone-less formats (treat as UTC)
	for fmt in NAIVE_FORMATS {
		if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, fmt) {
			return Ok(dt.and_utc().timestamp());
		}
	}

	Err(())
}

/// Compare two sort values with direction
fn compare_sort_values(a: &SortValue, b: &SortValue, direction: &str) -> std::cmp::Ordering {
	// Null values always come last
	let a_is_null = matches!(a, SortValue::Null);
	let b_is_null = matches!(b, SortValue::Null);

	if a_is_null && b_is_null {
		return std::cmp::Ordering::Equal;
	}
	if a_is_null {
		return std::cmp::Ordering::Greater;
	}
	if b_is_null {
		return std::cmp::Ordering::Less;
	}

	let cmp = match (a, b) {
		(SortValue::Number(a), SortValue::Number(b)) => {
			a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
		}
		(SortValue::Date(a), SortValue::Date(b)) => a.cmp(b),
		(SortValue::String(a), SortValue::String(b)) => a.to_lowercase().cmp(&b.to_lowercase()),
		(SortValue::String(_), SortValue::Number(_)) => std::cmp::Ordering::Greater,
		(SortValue::Number(_), SortValue::String(_)) => std::cmp::Ordering::Less,
		(SortValue::Date(_), _) => std::cmp::Ordering::Less,
		(_, SortValue::Date(_)) => std::cmp::Ordering::Greater,
		_ => std::cmp::Ordering::Equal,
	};

	if direction == "desc" {
		cmp.reverse()
	} else {
		cmp
	}
}

/// Helper function to sort JsonLine items and stream results
async fn sort_and_stream_json_lines(
	mut items: Vec<(usize, JsonLine, SortValue)>,
	direction: String,
	channel: Channel<Vec<JsonLine>>,
) -> Result<usize, String> {
	// Sort by pre-extracted values
	items.sort_by(|a, b| {
		let cmp = compare_sort_values(&a.2, &b.2, &direction);
		if cmp == std::cmp::Ordering::Equal {
			a.0.cmp(&b.0)  // Stable sort by original index
		} else {
			cmp
		}
	});

	// Extract sorted lines for streaming
	let lines: Vec<JsonLine> = items.into_iter().map(|(_, line, _)| line).collect();
	let lines_len = lines.len();

	// Stream sorted results
	const CHUNK_SIZE: usize = 2000;
	let mut chunk: Vec<JsonLine> = Vec::with_capacity(CHUNK_SIZE);

	for line in &lines {
		chunk.push(line.clone());
		if chunk.len() >= CHUNK_SIZE {
			channel.send(chunk.clone()).map_err(|e| format!("Failed to send: {}", e))?;
			chunk.clear();
		}
	}

	if !chunk.is_empty() {
		channel.send(chunk).map_err(|e| format!("Failed to send: {}", e))?;
	}

	Ok(lines_len)
}

/// Helper function to sort SearchResult items and stream results
async fn sort_and_stream_search_results(
	mut items: Vec<(usize, SearchResult, SortValue)>,
	direction: String,
	channel: Channel<Vec<SearchResult>>,
) -> Result<usize, String> {
	// Sort by pre-extracted values
	items.sort_by(|a, b| {
		let cmp = compare_sort_values(&a.2, &b.2, &direction);
		if cmp == std::cmp::Ordering::Equal {
			a.0.cmp(&b.0)  // Stable sort by original index
		} else {
			cmp
		}
	});

	// Extract sorted results for streaming
	let sorted_results: Vec<SearchResult> = items.into_iter().map(|(_, result, _)| result).collect();
	let sorted_len = sorted_results.len();

	// Stream sorted results
	const CHUNK_SIZE: usize = 100;
	let mut chunk: Vec<SearchResult> = Vec::with_capacity(CHUNK_SIZE);

	for result in &sorted_results {
		chunk.push(result.clone());
		if chunk.len() >= CHUNK_SIZE {
			channel.send(chunk.clone()).map_err(|e| format!("Failed to send: {}", e))?;
			chunk.clear();
		}
	}

	if !chunk.is_empty() {
		channel.send(chunk).map_err(|e| format!("Failed to send: {}", e))?;
	}

	Ok(sorted_len)
}

/// Command to sort all lines in a file by a column
#[tauri::command]
pub async fn sort_file_lines(
	path: String,
	sort_column: SortColumn,
	file_format: FileFormat,
	channel: Channel<Vec<JsonLine>>,
) -> Result<usize, String> {
	let file_path = PathBuf::from(&path);
	let direction = sort_column.direction.clone();
	let column_path = sort_column.column.clone();

	// Handle JsonArray format
	if matches!(file_format, FileFormat::JsonArray) {
		let content = tokio::fs::read_to_string(&file_path)
			.await
			.map_err(|e| format!("Failed to read file: {}", e))?;

		let json: serde_json::Value = serde_json::from_str(&content)
			.map_err(|e| format!("Failed to parse JSON: {}", e))?;

		if let Some(array) = json.as_array() {
			// Extract sort keys once per item for better performance
			let items: Vec<(usize, JsonLine, SortValue)> = array
				.iter()
				.enumerate()
				.map(|(index, item)| {
					let sort_val = get_nested_value(item, &column_path);
					let sort_key = sort_val.as_ref().map(|v| to_sort_value(v)).unwrap_or(SortValue::Null);
					(
						index,
						JsonLine {
							id: index,
							content: serde_json::to_string(item).unwrap_or_default(),
							parsed: item.clone(),
							byte_offset: 0,
						},
						sort_key,
					)
				})
				.collect();

			return sort_and_stream_json_lines(items, direction, channel).await;
		}
	}

	// Default JsonL format
	let file = tokio::fs::File::open(&file_path)
		.await
		.map_err(|e| format!("Failed to open file: {}", e))?;

	let reader = tokio::io::BufReader::new(file);
	let mut lines = reader.lines();

	let mut line_num = 0;
	let mut items: Vec<(usize, JsonLine, SortValue)> = Vec::new();

	while let Ok(Some(line)) = lines.next_line().await {
		if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
			let sort_val = get_nested_value(&json, &column_path);
			let sort_key = sort_val.as_ref().map(|v| to_sort_value(v)).unwrap_or(SortValue::Null);

			items.push((
				line_num,
				JsonLine {
					id: line_num,
					content: line.clone(),
					parsed: json,
					byte_offset: 0,
				},
				sort_key,
			));
		}
		line_num += 1;
	}

	sort_and_stream_json_lines(items, direction, channel).await
}

/// Command to sort search results by a column
#[tauri::command]
pub async fn sort_search_results(
	results: Vec<SearchResult>,
	sort_column: SortColumn,
	channel: Channel<Vec<SearchResult>>,
) -> Result<usize, String> {
	let direction = sort_column.direction.clone();
	let column_path = sort_column.column.clone();

	// Extract sort keys once per item for better performance
	let items: Vec<(usize, SearchResult, SortValue)> = results
		.into_iter()
		.enumerate()
		.map(|(index, result)| {
			let parsed = serde_json::from_str::<serde_json::Value>(&result.context).ok();
			let val = parsed.as_ref().and_then(|v| get_nested_value(v, &column_path));
			let sort_key = val.as_ref().map(|v| to_sort_value(v)).unwrap_or(SortValue::Null);
			(index, result, sort_key)
		})
		.collect();

	sort_and_stream_search_results(items, direction, channel).await
}
