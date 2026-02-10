<script lang="ts">
import { fileStore } from "$lib/stores/fileStore";
import { searchStore } from "$lib/stores/searchStore";
import { sortStore } from "$lib/stores/sortStore";
import { invoke, Channel } from '@tauri-apps/api/core';
import type { JsonLine, SearchResult } from "$lib/types";
import { getValue } from "$lib/utils/valueFormat";
import VirtualList from "./VirtualList.svelte";
import Spinner from "./ui/spinner.svelte";
import { Braces, ArrowUp, ArrowDown } from "lucide-svelte";

let isSearching = $derived(!!($searchStore.query.text || $searchStore.query.json_path));

// Sort command handlers
async function sortFileLines(column: string, direction: 'asc' | 'desc') {
	sortStore.setSorting(true);
	sortStore.setColumn(column, direction);

	try {
		const channel = new Channel<JsonLine[]>();
		let isFirstChunk = true;

		channel.onmessage = (chunk) => {
			if (isFirstChunk) {
				fileStore.replaceLines(chunk);
				isFirstChunk = false;
			} else {
				fileStore.addLines(chunk);
			}
		};

		await invoke('sort_file_lines', {
			path: $fileStore.filePath,
			sortColumn: { column, direction },
			fileFormat: $fileStore.format,
			channel
		});
	} catch (error) {
		sortStore.setError(String(error));
	} finally {
		sortStore.setSorting(false);
	}
}

async function sortSearchResults(column: string, direction: 'asc' | 'desc') {
	sortStore.setSorting(true);
	sortStore.setColumn(column, direction);

	try {
		const channel = new Channel<SearchResult[]>();
		let isFirstChunk = true;

		channel.onmessage = (chunk) => {
			if (isFirstChunk) {
				searchStore.replaceResults(chunk);
				isFirstChunk = false;
			} else {
				searchStore.addResults(chunk);
			}
		};

		await invoke('sort_search_results', {
			results: $searchStore.results,
			sortColumn: { column, direction },
			channel
		});
	} catch (error) {
		sortStore.setError(String(error));
	} finally {
		sortStore.setSorting(false);
	}
}

function handleHeaderClick(colPath: string) {
	const currentCol = $sortStore.state.column;
	const currentDir = $sortStore.state.direction;

	// Toggle or set new column
	let newDirection: 'asc' | 'desc';
	if (currentCol === colPath) {
		newDirection = currentDir === 'asc' ? 'desc' : 'asc';
	} else {
		newDirection = 'asc';
	}

	// Execute sort based on current context
	if (isSearching) {
		sortSearchResults(colPath, newDirection);
	} else {
		sortFileLines(colPath, newDirection);
	}
}

let displayItems = $derived.by(() => {
    if (isSearching) {
        return $searchStore.results.map((r) => {
            let parsed = {};
            try {
                parsed = JSON.parse(r.context);
            } catch (e) {}

            return {
                id: r.line_id,
                content: r.context,
                parsed
            } as JsonLine;
        });
    }
    return $fileStore.lines;
});

let columnInfo = $derived($fileStore.columnInfo);
let gridCols = $derived(
    columnInfo.length > 0
        ? `60px repeat(${columnInfo.length}, minmax(100px, 1fr))`
        : "1fr",
);

	let scrollLeft = $state(0);
	let headerEl: HTMLDivElement | undefined = $state();

	$effect(() => {
		if (headerEl) {
			headerEl.scrollLeft = scrollLeft;
		}
	});

    // Compute Header Groups
    let headerGroups = $derived.by(() => {
        const groups: { name: string; span: number; isGroup: boolean }[] = [];
        let currentGroup = "";
        let currentSpan = 0;

        for (const col of columnInfo) {
            const parts = col.path.split("_");
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
                    groups.push({ name: col.path, span: 1, isGroup: false });
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

	let totalMinWidth = $derived(
		columnInfo.length > 0
			? 60 + columnInfo.length * 100 // 60px ID + 100px per column
			: "100%"
	);
</script>

 <div class="flex-1 flex flex-col h-full overflow-hidden">
    {#if $fileStore.isLoading || ($searchStore.isSearching && displayItems.length === 0) || $sortStore.isSorting}
        <div class="flex items-center justify-center h-full">
            <Spinner size="lg" text={$sortStore.isSorting ? "Sorting..." : isSearching ? "Searching..." : "Loading file..."} />
        </div>
    {:else if $fileStore.error || $searchStore.error}
        <div class="flex items-center justify-center h-full">
            <div class="text-lg text-destructive">Error: {$fileStore.error || $searchStore.error}</div>
        </div>
    {:else if $fileStore.lines.length === 0}
        <div class="flex items-center justify-center h-full">
            <div class="text-center">
                <h2 class="text-xl font-semibold mb-2">No file loaded</h2>
                <p class="text-muted-foreground">Open a JSON or JSON-L file to get started</p>
            </div>
        </div>
    {:else if isSearching && displayItems.length === 0 && !$searchStore.isSearching}
        <div class="flex items-center justify-center h-full">
            <div class="text-center">
                <h2 class="text-xl font-semibold mb-2">No results found</h2>
                <p class="text-muted-foreground">Try adjusting your search query</p>
            </div>
        </div>
    {:else}
        <!-- Table Header (Grouped) -->
        <div
			bind:this={headerEl}
			class="bg-muted/40 font-medium text-sm border-b border-border shadow-sm z-10 overflow-hidden"
		>
            <!-- Top Row (Groups) -->
            <div class="grid" style="grid-template-columns: {gridCols}; padding-right: 8px; min-width: {typeof totalMinWidth === 'number' ? totalMinWidth + 'px' : totalMinWidth};">
                <div class="p-2 pl-3 text-muted-foreground border-r border-border/30 flex items-end pb-1">#</div>

                {#each headerGroups as group}
                    <div
                        class="p-1 px-2 text-center text-xs uppercase tracking-wider text-muted-foreground font-semibold border-r border-border/30 bg-muted/20"
                        style="grid-column: span {group.span};"
                    >
                        {group.isGroup ? group.name : ''}
                    </div>
                {/each}

                {#if columnInfo.length === 0}
                    <div class="p-2">Content</div>
                {/if}
            </div>

            <!-- Bottom Row (Sub-keys) -->
            <div class="grid border-t border-border/30 bg-background/50" style="grid-template-columns: {gridCols}; padding-right: 8px; min-width: {typeof totalMinWidth === 'number' ? totalMinWidth + 'px' : totalMinWidth};">
                <div class="bg-muted/10 border-r border-border/30"></div> <!-- Spacer for ID -->

                {#each columnInfo as col}
                    <div
                        class="p-1 px-2 text-muted-foreground text-xs truncate border-r border-border/30
                            {col.isSortable ? 'cursor-pointer hover:bg-muted/50' : 'opacity-50'}
                            {$sortStore.state.column === col.path ? 'font-bold text-foreground' : ''}"
                        title={col.path}
                        onmousedown={() => col.isSortable && handleHeaderClick(col.path)}
                        onkeydown={(e) => col.isSortable && e.key === 'Enter' && handleHeaderClick(col.path)}
                        role="columnheader"
                        tabindex={col.isSortable ? 0 : -1}
                        aria-sort={$sortStore.state.column === col.path
                            ? $sortStore.state.direction === 'asc' ? 'ascending' : 'descending'
                            : 'none'}
                    >
                        <div class="flex items-center gap-1">
                            <span>{col.displayName}</span>
                            {#if $sortStore.state.column === col.path}
                                {#if $sortStore.state.direction === 'asc'}
                                    <ArrowUp class="w-3 h-3" />
                                {:else}
                                    <ArrowDown class="w-3 h-3" />
                                {/if}
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>
        </div>

        <!-- Virtual Body -->
        <div class="flex-1 min-h-0">
            <VirtualList
                items={displayItems}
                itemHeight={36}
                overscan={10}
				minWidth={totalMinWidth}
				bind:scrollLeft
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
						{#each columnInfo as col}
                            {@const val = getValue(item.parsed, col.path)}
							<div class="px-2 truncate border-l border-border/30 h-full flex items-center gap-1.5" title={val.text}>
                                {#if val.isComplex}
                                    <Braces class="w-3 h-3 text-muted-foreground/70 shrink-0" />
                                {/if}
								<span class="truncate">{val.text}</span>
							</div>
						{/each}

						<!-- Fallback content if no columns (e.g. primitive array) -->
						{#if columnInfo.length === 0}
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