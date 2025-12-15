import { writable } from 'svelte/store';
import type { JsonLine, FileMetadata } from '$lib/types';

interface FileState {
	filePath: string | null;
	lines: JsonLine[];
	totalLines: number;
	fileSize: number;
	format: 'JsonL' | 'JsonArray';
	isLoading: boolean;
	error: string | null;
	metadata: FileMetadata | null;
	selectedLineId: number | null;
	columns: string[];
}

function createFileStore() {
	const { subscribe, set, update } = writable<FileState>({
		filePath: null,
		lines: [],
		totalLines: 0,
		fileSize: 0,
		format: 'JsonL',
		isLoading: false,
		error: null,
		metadata: null,
		selectedLineId: null,
		columns: []
	});

	return {
		subscribe,
		setFilePath: (path: string | null) =>
			update(state => ({ ...state, filePath: path })),
		setLoading: (loading: boolean) =>
			update(state => ({ ...state, isLoading: loading })),
		setError: (error: string | null) =>
			update(state => ({ ...state, error })),
		setMetadata: (metadata: FileMetadata | null) =>
			update(state => ({
				...state,
				metadata,
				filePath: metadata?.path || null,
				totalLines: metadata?.total_lines || 0,
				fileSize: metadata?.file_size || 0,
				format: metadata?.format || 'JsonL'
			})),
		setSelectedLine: (id: number | null) =>
			update(state => ({ ...state, selectedLineId: id })),
		addLines: (lines: JsonLine[]) =>
			update(state => {
				const newLines = state.lines.concat(lines);
				let columns = state.columns;

				// Extract columns from the first batch if not yet set
				if (columns.length === 0 && newLines.length > 0) {
					const sampleSize = Math.min(newLines.length, 50);
					const keyCounts = new Map<string, number>();

					// Recursive helper to find flat keys
					const collectKeys = (obj: any, prefix: string = '', depth: number = 0) => {
						if (depth > 2 || !obj || typeof obj !== 'object' || Array.isArray(obj)) return;

						Object.keys(obj).forEach(key => {
							const val = obj[key];
							const newKey = prefix ? `${prefix}_${key}` : key;

							// If value is simple or we hit max depth, count this key
							if (typeof val !== 'object' || val === null || Array.isArray(val) || depth === 2) {
								keyCounts.set(newKey, (keyCounts.get(newKey) || 0) + 1);
							} else {
								// Recurse into object
								collectKeys(val, newKey, depth + 1);
							}
						});
					};

					for (let i = 0; i < sampleSize; i++) {
						collectKeys(newLines[i].parsed);
					}

					// Priority keys (include prefixes)
					const priorityKeys = ['id', 'timestamp', 'time', 'date', 'level', 'severity', 'message', 'msg', 'name', 'type', 'status', 'user', 'meta'];

					// Sort keys by priority and then by frequency
					columns = Array.from(keyCounts.keys()).sort((a, b) => {
						const aBase = a.split('_')[0].toLowerCase();
						const bBase = b.split('_')[0].toLowerCase();

						const aPrio = priorityKeys.indexOf(aBase);
						const bPrio = priorityKeys.indexOf(bBase);

						if (aPrio !== -1 && bPrio !== -1) {
							if (aPrio === bPrio) return a.localeCompare(b);
							return aPrio - bPrio;
						}
						if (aPrio !== -1) return -1;
						if (bPrio !== -1) return 1;

						return (keyCounts.get(b) || 0) - (keyCounts.get(a) || 0);
					}).slice(0, 100); // Increase cap to allow more columns
				}

				return {
					...state,
					lines: newLines,
					totalLines: state.lines.length + lines.length,
					columns
				};
			}),
		clearLines: () =>
			update(state => ({
				...state,
				lines: [],
				totalLines: 0,
				columns: [],
				selectedLineId: null
			})),
		reset: () => set({
			filePath: null,
			lines: [],
			totalLines: 0,
			fileSize: 0,
			format: 'JsonL',
			isLoading: false,
			error: null,
			metadata: null,
			selectedLineId: null,
			columns: []
		})
	};
}

export const fileStore = createFileStore();