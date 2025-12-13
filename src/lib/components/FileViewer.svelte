<script lang="ts">
import { fileStore } from "$lib/stores/fileStore";
import VirtualList from "./VirtualList.svelte";
import Spinner from "./ui/spinner.svelte";

// Helper to safely get value for a nested column path (e.g. "user_name")
function getValue(parsed: any, colPath: string): string {
	if (!parsed || typeof parsed !== "object") return "";

	const parts = colPath.split("_");
	let current = parsed;

	for (const part of parts) {
		if (
			current === null ||
			current === undefined ||
			typeof current !== "object"
		)
			return "";
		current = current[part];
	}

	if (current === undefined || current === null) return "";
	if (typeof current === "object") return JSON.stringify(current);
	return String(current);
}

let columns = $derived($fileStore.columns);
let gridCols = $derived(
	columns.length > 0
		? `60px repeat(${columns.length}, minmax(100px, 1fr))`
		: "1fr",
);

// Compute Header Groups
let headerGroups = $derived.by(() => {
	const groups: { name: string; span: number; isGroup: boolean }[] = [];
	let currentGroup = "";
	let currentSpan = 0;

	for (const col of columns) {
		const parts = col.split("_");
		const groupName = parts.length > 1 ? parts[0] : "";

		if (groupName && groupName === currentGroup) {
			currentSpan++;
		} else {
			if (currentGroup) {
				groups.push({ name: currentGroup, span: currentSpan, isGroup: true });
			}
			// If no group (top level key), push as single
			if (!groupName) {
				if (currentGroup) {
					// Previous was a group, this is not.
					// Push the previous group
					// (Already handled above: currentGroup check)
				}
				groups.push({ name: col, span: 1, isGroup: false });
				currentGroup = "";
				currentSpan = 0;
			} else {
				// New group
				currentGroup = groupName;
				currentSpan = 1;
			}
		}
	}
	// Push last group
	if (currentGroup) {
		groups.push({ name: currentGroup, span: currentSpan, isGroup: true });
	}

	return groups;
});
</script>

<div class="flex-1 flex flex-col h-full overflow-hidden">
	{#if $fileStore.isLoading}
		<div class="flex items-center justify-center h-full">
			<Spinner size="lg" text="Loading file..." />
		</div>
	{:else if $fileStore.error}
		<div class="flex items-center justify-center h-full">
			<div class="text-lg text-destructive">Error: {$fileStore.error}</div>
		</div>
	{:else if $fileStore.lines.length === 0}
		<div class="flex items-center justify-center h-full">
			<div class="text-center">
				<h2 class="text-xl font-semibold mb-2">No file loaded</h2>
				<p class="text-muted-foreground">Open a JSON or JSON-L file to get started</p>
			</div>
		</div>
	{:else}
		<!-- Table Header (Grouped) -->
		<div class="bg-muted/40 font-medium text-sm border-b border-border shadow-sm z-10">
			<!-- Top Row (Groups) -->
			<div class="grid" style="grid-template-columns: {gridCols}; padding-right: 8px;">
				<div class="p-2 pl-3 text-muted-foreground border-r border-border/30 flex items-end pb-1">#</div>

				{#each headerGroups as group}
					<div
						class="p-1 px-2 text-center text-xs uppercase tracking-wider text-muted-foreground font-semibold border-r border-border/30 bg-muted/20"
						style="grid-column: span {group.span};"
					>
						{group.isGroup ? group.name : ''}
					</div>
				{/each}

				{#if columns.length === 0}
					<div class="p-2">Content</div>
				{/if}
			</div>

			<!-- Bottom Row (Sub-keys) -->
			<div class="grid border-t border-border/30 bg-background/50" style="grid-template-columns: {gridCols}; padding-right: 8px;">
				<div class="bg-muted/10 border-r border-border/30"></div> <!-- Spacer for ID -->

				{#each columns as col}
					{@const parts = col.split('_')}
					{@const displayName = parts.length > 1 ? parts.slice(1).join('_') : parts[0]}
					<div class="p-1 px-2 text-muted-foreground text-xs truncate border-r border-border/30" title={col}>
						{displayName}
					</div>
				{/each}
			</div>
		</div>

		<!-- Virtual Body -->
		<div class="flex-1 min-h-0">
			<VirtualList
				items={$fileStore.lines}
				itemHeight={36}
				overscan={10}
			>
				{#snippet children(item, index)}
					<div
						class="grid hover:bg-sky-500/10 cursor-pointer h-full items-center transition-colors font-mono text-sm border-b border-border/40"
						style="grid-template-columns: {gridCols};"
						class:bg-sky-500-20={$fileStore.selectedLineId === item.id}
						onclick={() => fileStore.setSelectedLine(item.id)}
						role="button"
						tabindex="0"
						onkeydown={(e) => e.key === 'Enter' && fileStore.setSelectedLine(item.id)}
					>
						<!-- ID Column -->
						<div class="px-3 text-muted-foreground truncate">{item.id}</div>

						<!-- Dynamic Columns -->
						{#each columns as col}
							<div class="px-2 truncate border-l border-border/30 h-full flex items-center" title={getValue(item.parsed, col)}>
								{getValue(item.parsed, col)}
							</div>
						{/each}

						<!-- Fallback content if no columns (e.g. primitive array) -->
						{#if columns.length === 0}
							<div class="px-2 truncate h-full flex items-center">{item.content}</div>
						{/if}
					</div>
				{/snippet}
			</VirtualList>
		</div>
	{/if}
</div>

<style>
	/* Make sure highlighted row stands out */
	.bg-sky-500-20 {
		background-color: rgba(14, 165, 233, 0.15);
	}
</style>