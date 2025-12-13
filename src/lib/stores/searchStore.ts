import { writable, derived } from 'svelte/store';
import type { SearchQuery, SearchResult, SearchStats } from '$lib/types';

interface SearchState {
	query: SearchQuery;
	results: SearchResult[];
	stats: SearchStats | null;
	isSearching: boolean;
	error: string | null;
}

function createSearchStore() {
	const { subscribe, set, update } = writable<SearchState>({
		query: {
			text: '',
			json_path: '',
			case_sensitive: false,
			regex: false
		},
		results: [],
		stats: null,
		isSearching: false,
		error: null,
	});

	return {
		subscribe,
		setQuery: (query: Partial<SearchQuery>) =>
			update(state => ({
				...state,
				query: { ...state.query, ...query }
			})),
		setResults: (results: SearchResult[]) =>
			update(state => ({ ...state, results })),
		addResults: (newResults: SearchResult[]) =>
			update(state => ({ ...state, results: [...state.results, ...newResults] })),
		setStats: (stats: SearchStats | null) =>
			update(state => ({ ...state, stats })),
		setSearching: (searching: boolean) =>
			update(state => ({ ...state, isSearching: searching })),
		setError: (error: string | null) =>
			update(state => ({ ...state, error })),
		reset: () => set({
			query: {
				text: '',
				json_path: '',
				case_sensitive: false,
				regex: false
			},
			results: [],
			stats: null,
			isSearching: false,
			error: null,
		})
	};
}

export const searchStore = createSearchStore();