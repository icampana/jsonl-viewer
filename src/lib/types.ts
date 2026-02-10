export interface JsonLine {
	id: number;
	content: string;
	parsed: Record<string, unknown>;
	byte_offset: number;
}

export interface FileMetadata {
	path: string;
	total_lines: number;
	file_size: number;
	format: 'JsonL' | 'JsonArray';
}

export interface SearchQuery {
	text?: string;
	json_path?: string;
	case_sensitive: boolean;
	regex: boolean;
}

export interface SearchResult {
	line_id: number;
	matches: string[];
	context: string;
}

export interface SearchStats {
	total_matches: number;
	lines_searched: number;
}

export interface ExportFilter {
	line_ids?: number[];
	search_query?: SearchQuery;
}

export interface ExportStats {
	lines_exported: number;
	file_size: number;
}

export type ExportFormat = 'JsonL' | 'JsonArray' | 'Csv' | 'Excel';

export interface SortState {
	column: string | null;
	direction: 'asc' | 'desc';
}

export interface SortColumn {
	column: string;
	direction: string;
}

export interface ColumnInfo {
	path: string;
	isSortable: boolean;
	displayName: string;
}