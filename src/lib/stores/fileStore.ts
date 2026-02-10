import { writable } from 'svelte/store';
import type { JsonLine, FileMetadata, ColumnInfo } from '$lib/types';
import { getValue } from '$lib/utils/valueFormat';
import { sortStore } from './sortStore';

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
	columnInfo: ColumnInfo[];
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
		columnInfo: []
	});

	return {
		subscribe,
		setFilePath: (path: string | null) =>
			update((state) => {
				// When path changes (new file), reset sort
				if (path !== state.filePath) {
					sortStore.reset();
				}
				return { ...state, filePath: path };
			}),
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
				let columnInfo = state.columnInfo;

				// Extract columns from the first batch if not yet set
				if (columnInfo.length === 0 && newLines.length > 0) {
					const sampleSize = Math.min(newLines.length, 50);
					const keyCounts = new Map<string, number>();
					const columnComplexity = new Map<string, boolean>();

					// Recursive helper to find flat keys
					const collectKeys = (obj: any, prefix: string = '', depth: number = 0) => {
						if (depth > 2 || !obj || typeof obj !== 'object' || Array.isArray(obj)) return;

						Object.keys(obj).forEach(key => {
							const val = obj[key];
							const newKey = prefix ? `${prefix}_${key}` : key;

							// If value is simple or we hit max depth, count this key
							if (
								typeof val !== 'object' ||
								val === null ||
								Array.isArray(val) ||
								depth === 2
							) {
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

					// Check if column values are all simple (not complex) - for sortability
					for (const col of Array.from(keyCounts.keys())) {
						let isSortable = true;

						for (let i = 0; i < sampleSize; i++) {
							const { isComplex } = getValue(newLines[i].parsed, col);
							if (isComplex) {
								isSortable = false;
								break;
							}
						}

						columnComplexity.set(col, isSortable);
					}

					// Priority keys (include prefixes)
					const priorityKeys = [
						'id',
						'timestamp',
						'time',
						'date',
						'level',
						'severity',
						'message',
						'msg',
						'name',
						'type',
						'status',
						'user',
						'meta'
					];

					// Sort keys by priority and then by frequency
					const sortedKeys = Array.from(keyCounts.keys()).sort((a, b) => {
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

					// Build ColumnInfo array
					columnInfo = sortedKeys.map((path) => ({
						path,
						isSortable: columnComplexity.get(path) ?? false,
						displayName: path.split('_').slice(1).join('_') || path
					}));
				}

				return {
					...state,
					lines: newLines,
					totalLines: state.lines.length + lines.length,
					columnInfo
				};
			}),
		clearLines: () =>
			update(state => ({
				...state,
				lines: [],
				totalLines: 0,
				columnInfo: [],
				selectedLineId: null
			})),
		replaceLines: (newLines: JsonLine[]) =>
			update(state => ({
				...state,
				lines: newLines
			})),
		reset: () =>
			set({
				filePath: null,
				lines: [],
				totalLines: 0,
				fileSize: 0,
				format: 'JsonL',
				isLoading: false,
				error: null,
				metadata: null,
				selectedLineId: null,
				columnInfo: []
			})
	};
}

export const fileStore = createFileStore();