<script lang="ts">
import { fileStore } from "$lib/stores/fileStore";
import { searchStore } from "$lib/stores/searchStore";
import VirtualList from "./VirtualList.svelte";
import Spinner from "./ui/spinner.svelte";
import type { JsonLine } from "$lib/types";
import { Braces } from "lucide-svelte";

// Helper to safely get value for a nested column path (e.g. "user_name")
function getValue(parsed: any, colPath: string): { text: string; isComplex: boolean } {
    if (!parsed || typeof parsed !== "object") return { text: "", isComplex: false };

    const parts = colPath.split("_");
    let current = parsed;

    for (const part of parts) {
        if (
            current === null ||
            current === undefined ||
            typeof current !== "object"
        )
            return { text: "", isComplex: false };
        current = current[part];
    }

    if (current === undefined || current === null) return { text: "", isComplex: false };
    return smartFormat(current);
}

function smartFormat(value: any): { text: string; isComplex: boolean } {
    if (value === null || value === undefined) return { text: "", isComplex: false };

    if (Array.isArray(value)) {
        if (value.length === 0) return { text: "[]", isComplex: true };

        // heuristic: check first item for common display keys
        const first = value[0];
        if (typeof first === 'object' && first !== null) {
             const displayKeys = ['name', 'title', 'label', 'id', 'slug', 'email', 'username', 'code', 'key', 'status'];
             const hit = displayKeys.find(k => k in first);
             if (hit) {
                 return {
                     text: value.map((v: any) => v && v[hit]).join(", "),
                     isComplex: true
                 };
             }
        }

        // fallback for mixed arrays or objects without common keys
        const text = value.map((v: any) => {
            if (typeof v === 'object' && v !== null) return JSON.stringify(v);
            return String(v);
        }).join(", ");
        return { text, isComplex: true };
    }

    if (typeof value === "object") {
        const displayKeys = ['name', 'title', 'label', 'id', 'slug', 'email', 'username', 'code', 'key', 'status'];
        const hit = displayKeys.find(k => k in value);
        if (hit) {
            return { text: String(value[hit]), isComplex: true };
        }
        return { text: JSON.stringify(value), isComplex: true };
    }

    return { text: String(value), isComplex: false };
}

let isSearching = $derived(!!($searchStore.query.text || $searchStore.query.json_path));

let displayItems = $derived.by(() => {
    if (isSearching) {
        return $searchStore.results.map(r => {
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

let columns = $derived($fileStore.columns);
let gridCols = $derived(
    columns.length > 0
        ? `60px repeat(${columns.length}, minmax(100px, 1fr))`
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

	let totalMinWidth = $derived(
		columns.length > 0
			? 60 + columns.length * 100 // 60px ID + 100px per column
			: "100%"
	);
</script>

<div class="flex-1 flex flex-col h-full overflow-hidden">
    {#if $fileStore.isLoading || ($searchStore.isSearching && displayItems.length === 0)}
        <div class="flex items-center justify-center h-full">
            <Spinner size="lg" text={isSearching ? "Searching..." : "Loading file..."} />
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

                {#if columns.length === 0}
                    <div class="p-2">Content</div>
                {/if}
            </div>

            <!-- Bottom Row (Sub-keys) -->
            <div class="grid border-t border-border/30 bg-background/50" style="grid-template-columns: {gridCols}; padding-right: 8px; min-width: {typeof totalMinWidth === 'number' ? totalMinWidth + 'px' : totalMinWidth};">
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
						{#each columns as col}
                            {@const val = getValue(item.parsed, col)}
							<div class="px-2 truncate border-l border-border/30 h-full flex items-center gap-1.5" title={val.text}>
                                {#if val.isComplex}
                                    <Braces class="w-3 h-3 text-muted-foreground/70 shrink-0" />
                                {/if}
								<span class="truncate">{val.text}</span>
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