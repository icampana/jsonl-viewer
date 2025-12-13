import { writable } from 'svelte/store';

type Theme = 'dark' | 'light' | 'system';

function createThemeStore() {
	const { subscribe, set, update } = writable<Theme>('dark');

	return {
		subscribe,
		setTheme: (theme: Theme) => set(theme),
		toggle: () => update(theme => theme === 'dark' ? 'light' : 'dark')
	};
}

export const themeStore = createThemeStore();