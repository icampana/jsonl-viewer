import { writable } from 'svelte/store';
import type { SortState } from '$lib/types';

interface SortStore {
	state: SortState;
	isSorting: boolean;
	error: string | null;
}

function createSortStore() {
	const { subscribe, set, update } = writable<SortStore>({
		state: { column: null, direction: 'asc' },
		isSorting: false,
		error: null,
	});

	return {
		subscribe,
		setColumn: (column: string | null, direction: 'asc' | 'desc') =>
			update((state) => ({
				...state,
				state: { column, direction }
			})),
		setSorting: (sorting: boolean) =>
			update((state) => ({ ...state, isSorting: sorting })),
		setError: (error: string | null) =>
			update((state) => ({ ...state, error })),
		toggleColumn: (column: string) =>
			update((state) => {
				const currentCol = state.state.column;
				const currentDir = state.state.direction;

				if (currentCol === column) {
					// Toggle direction
					return {
						...state,
						state: { column, direction: currentDir === 'asc' ? 'desc' : 'asc' }
					};
				} else {
					// New column, default asc
					return {
						...state,
						state: { column, direction: 'asc' }
					};
				}
			}),
		reset: () =>
			set({
				state: { column: null, direction: 'asc' },
				isSorting: false,
				error: null
			})
	};
}

export const sortStore = createSortStore();
