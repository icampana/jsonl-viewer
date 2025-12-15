<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { Search, Filter, X } from 'lucide-svelte';
	import Button from '$lib/components/ui/button.svelte';
	import { searchStore } from '$lib/stores/searchStore';
	import { invoke } from '@tauri-apps/api/core';
	import { fileStore } from '$lib/stores/fileStore';
	import { cn } from '$lib/utils';

	let searchInput = '';
	let jsonPathInput = '';
	let showAdvanced = false;
	let caseSensitive = false;
	let useRegex = false;

	const dispatch = createEventDispatcher();

	let unsubscribe: () => void;

	import { onMount, onDestroy } from 'svelte';

	onMount(() => {
		unsubscribe = searchStore.subscribe(state => {
			let shouldSearch = false;

			if (state.query.text !== undefined && state.query.text !== searchInput) {
				searchInput = state.query.text;
				shouldSearch = true;
			}
			if (state.query.json_path !== undefined && state.query.json_path !== jsonPathInput) {
				jsonPathInput = state.query.json_path;
				shouldSearch = true;
			}
			// If store cleared inputs (undefined/empty), sync that too if needed,
			// but state.query default is empty string in UI logic usually.
			// Check store init: text: '', json_path: ''.
			if (!state.query.text && searchInput) {
				searchInput = '';
				shouldSearch = true;
			}
			if (!state.query.json_path && jsonPathInput) {
				jsonPathInput = '';
				shouldSearch = true;
			}

			if (shouldSearch) {
				dispatch('search');
			}
		});
	});

	onDestroy(() => {
		if (unsubscribe) unsubscribe();
	});

	let debounceTimer: ReturnType<typeof setTimeout>;

	function handleSearch() {
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			triggerSearch();
		}, 300);
	}

	function immediateSearch() {
		clearTimeout(debounceTimer);
		triggerSearch();
	}

	function triggerSearch() {
		searchStore.setQuery({
			text: searchInput || undefined,
			json_path: jsonPathInput || undefined,
			case_sensitive: caseSensitive,
			regex: useRegex
		});
		dispatch('search');
	}

	function clearSearch() {
		searchInput = '';
		jsonPathInput = '';
		triggerSearch();
		document.getElementById('search-input')?.focus();
	}

	function handleKeyboardShortcut(event: KeyboardEvent) {
		if (event.metaKey && event.key === 'f') {
			event.preventDefault();
			document.getElementById('search-input')?.focus();
		}
	}
</script>

<svelte:window onkeydown={handleKeyboardShortcut} />

<div class="flex items-center gap-2 p-4 border-b border-border bg-card">
	<div class="flex-1 flex items-center gap-2">
		<div class="relative flex-1 max-w-md">
			<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-muted-foreground" />
			<input
				id="search-input"
				type="text"
				placeholder="Search in file..."
				bind:value={searchInput}
				oninput={handleSearch}
				class="w-full pl-10 pr-4 py-2 border border-input bg-background rounded-md text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
			/>
			{#if searchInput || jsonPathInput}
				<button
					onclick={clearSearch}
					class="absolute right-3 top-1/2 transform -translate-y-1/2 text-muted-foreground hover:text-foreground p-0.5 rounded-full hover:bg-muted transition-colors"
				>
					<X class="w-4 h-4" />
				</button>
			{/if}
		</div>

		<Button onclick={() => showAdvanced = !showAdvanced} variant="outline" size="sm">
			<Filter class="w-4 h-4 mr-2" />
			Advanced
		</Button>
	</div>
</div>

 {#if showAdvanced}
	<div class="flex items-center gap-4 p-4 border-b border-border bg-muted/50">
		<div class="flex items-center gap-2">
			<label for="json-path" class="text-sm font-medium">JSONPath:</label>
			<input
				id="json-path"
				type="text"
				placeholder="e.g., $.users[*].name"
				bind:value={jsonPathInput}
				oninput={handleSearch}
				class="w-64 px-3 py-1 border border-input bg-background rounded-md text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
			/>
		</div>

		<div class="flex items-center gap-4">
			<label class="flex items-center gap-2 text-sm">
				<input
					type="checkbox"
					bind:checked={caseSensitive}
					onchange={immediateSearch}
					class="rounded border-input"
				/>
				Case Sensitive
			</label>

			<!-- TODO: Enable Regex search in future versions -->
			<!-- <label class="flex items-center gap-2 text-sm text-muted-foreground cursor-not-allowed">
				<input
					type="checkbox"
					checked={false}
					disabled
					class="rounded border-input"
				/>
				Regex (Coming Soon)
			</label> -->
		</div>
	</div>
 {/if}