<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { FolderOpen, Moon, Sun, BarChart } from 'lucide-svelte';
	import Button from '$lib/components/ui/button.svelte';
	import { themeStore } from '$lib/stores/themeStore';
	import { fileStore } from '$lib/stores/fileStore';
	import ExportButton from './ExportButton.svelte';

	export let openFile: () => void;

	const dispatch = createEventDispatcher();

	function toggleTheme() {
		themeStore.toggle();
	}

	function handleKeyboardShortcut(event: KeyboardEvent) {
		if (event.metaKey && event.key === 'o') {
			event.preventDefault();
			openFile();
		}
		if (event.metaKey && event.key === 'd') {
			event.preventDefault();
			toggleTheme();
		}
	}

	function handleExportComplete(event: CustomEvent) {
		console.log('Export completed:', event.detail);
	}

	function handleExportError(event: CustomEvent) {
		console.error('Export failed:', event.detail);
	}
</script>

<svelte:window on:keydown={handleKeyboardShortcut} />

<header class="flex items-center justify-between px-4 py-2 border-b border-border bg-card">
	<div class="flex items-center gap-4">
		<Button on:click={openFile} variant="outline" size="sm">
			<FolderOpen class="w-4 h-4 mr-2" />
			Open File
		</Button>
		<h1 class="text-lg font-semibold">JSON-L Viewer</h1>
	</div>

	<div class="flex items-center gap-2">
		<ExportButton
			disabled={!$fileStore.metadata || $fileStore.lines.length === 0}
			on:export-complete={handleExportComplete}
			on:export-error={handleExportError}
		/>

		<Button
			variant="ghost"
			size="sm"
			disabled={!$fileStore.metadata || $fileStore.lines.length === 0}
			onclick={() => dispatch('open-stats')}
			title="View Statistics"
		>
			<BarChart class="w-4 h-4" />
		</Button>

		<Button on:click={toggleTheme} variant="ghost" size="sm">
			{#if $themeStore === 'dark'}
				<Sun class="w-4 h-4" />
			{:else}
				<Moon class="w-4 h-4" />
			{/if}
		</Button>
	</div>
</header>