<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { Search, Filter } from 'lucide-svelte';
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

	function handleSearch() {
		searchStore.setQuery({
			text: searchInput || undefined,
			json_path: jsonPathInput || undefined,
			case_sensitive: caseSensitive,
			regex: useRegex
		});
		dispatch('search');
	}

	function handleKeyboardShortcut(event: KeyboardEvent) {
		if (event.metaKey && event.key === 'f') {
			event.preventDefault();
			document.getElementById('search-input')?.focus();
		}
	}
</script>

<svelte:window on:keydown={handleKeyboardShortcut} />

<div class="flex items-center gap-2 p-4 border-b border-border bg-card">
	<div class="flex-1 flex items-center gap-2">
		<div class="relative flex-1 max-w-md">
			<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-muted-foreground" />
			<input
				id="search-input"
				type="text"
				placeholder="Search in file..."
				bind:value={searchInput}
				on:keyup={handleSearch}
				class="w-full pl-10 pr-4 py-2 border border-input bg-background rounded-md text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
			/>
		</div>
		
		<Button on:click={() => showAdvanced = !showAdvanced} variant="outline" size="sm">
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
				on:keyup={handleSearch}
				class="w-64 px-3 py-1 border border-input bg-background rounded-md text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
			/>
		</div>
		
		<div class="flex items-center gap-4">
			<label class="flex items-center gap-2 text-sm">
				<input
					type="checkbox"
					bind:checked={caseSensitive}
					on:change={handleSearch}
					class="rounded border-input"
				/>
				Case Sensitive
			</label>
			
			<label class="flex items-center gap-2 text-sm">
				<input
					type="checkbox"
					bind:checked={useRegex}
					on:change={handleSearch}
					class="rounded border-input"
				/>
				Regex
			</label>
		</div>
	</div>
 {/if}