<script lang="ts">
import { fileStore } from "$lib/stores/fileStore";
import { X, GripVertical } from "lucide-svelte";
import Button from "$lib/components/ui/button.svelte";
import JsonTree from "$lib/components/json-tree/JsonTree.svelte";

let { width = $bindable(400) } = $props<{ width?: number }>();

let selectedLine = $derived(
	$fileStore.selectedLineId !== null
		? $fileStore.lines.find((l) => l.id === $fileStore.selectedLineId)
		: null,
);

let isResizing = $state(false);
let startX = 0;
let startWidth = 0;

// Min/max constraints
const MIN_WIDTH = 300;
const MAX_WIDTH = 1200;

function startResize(e: MouseEvent) {
	isResizing = true;
	startX = e.clientX;
	startWidth = width;

	// Add global listeners
	window.addEventListener("mousemove", handleMouseMove);
	window.addEventListener("mouseup", stopResize);

	// Prevent selection while resizing
	document.body.style.userSelect = "none";
	document.body.style.cursor = "col-resize";
}

function handleMouseMove(e: MouseEvent) {
	if (!isResizing) return;

	const dx = startX - e.clientX; // Negative dx means growing to left (but it's right sidebar, so growing means mouse moves left)
	// Wait, sidebar is on right.
	// If mouse moves LEFT (smaller X), width INCREASES.
	// If mouse moves RIGHT (larger X), width DECREASES.
	// Delta = StartX - CurrentX.
	// Example: Start 1000, Move to 900. Delta = 100. New width = StartWidth + 100. Correct.

	const newWidth = Math.min(Math.max(startWidth + dx, MIN_WIDTH), MAX_WIDTH);
	width = newWidth;
}

function stopResize() {
	isResizing = false;
	window.removeEventListener("mousemove", handleMouseMove);
	window.removeEventListener("mouseup", stopResize);
	document.body.style.userSelect = "";
	document.body.style.cursor = "";
}

function close() {
	fileStore.setSelectedLine(null);
}
</script>

{#if selectedLine}
	<div
		class="h-full border-l border-border bg-background flex flex-col shadow-xl relative"
		style="width: {width}px;"
	>
		<!-- Resize Handle -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="absolute left-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-primary/50 transition-colors z-50 flex flex-col justify-center items-center group touch-none"
			class:bg-primary={isResizing}
			onmousedown={startResize}
		>
			<div class="h-8 w-1 rounded-full bg-border group-hover:bg-primary/50 transition-colors opacity-0 group-hover:opacity-100" class:opacity-100={isResizing}></div>
		</div>

		<div class="p-4 border-b border-border flex items-center justify-between bg-muted/10 shrink-0">
			<h3 class="font-semibold">Line #{selectedLine.id}</h3>
			<Button variant="ghost" size="icon" onclick={close} class="h-8 w-8">
				<X class="w-4 h-4" />
			</Button>
		</div>

		<div class="flex-1 overflow-y-auto p-4 custom-scrollbar min-h-0">
			<div class="bg-muted/30 rounded-lg p-3 border border-border/50">
				<JsonTree value={selectedLine.parsed} />
			</div>

			<div class="mt-6">
				<div class="flex items-center justify-between mb-2">
					<h4 class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Raw Content</h4>
					<Button variant="ghost" size="sm" class="h-6 text-xs" onclick={() => navigator.clipboard.writeText(selectedLine?.content || '')}>
						Copy
					</Button>
				</div>
				<div class="bg-black/80 text-white font-mono text-xs p-3 rounded overflow-x-auto whitespace-pre-wrap break-all">
					{selectedLine.content}
				</div>
			</div>
		</div>
	</div>
{/if}

<style>
	.custom-scrollbar {
		scrollbar-width: thin;
		scrollbar-color: rgb(203 213 225) transparent;
	}
</style>
