<script lang="ts">
import type { JsonLine } from "$lib/types";
import { Copy, ChevronDown, ChevronRight } from "lucide-svelte";
import Button from "$lib/components/ui/button.svelte";
import JsonTree from "$lib/components/json-tree/JsonTree.svelte";

let { line }: { line: JsonLine } = $props();
let expanded = $state(false);
let copied = $state(false);

async function copyToClipboard() {
	try {
		await navigator.clipboard.writeText(line.content);
		copied = true;
		setTimeout(() => {
			copied = false;
		}, 2000);
	} catch (err) {
		console.error("Failed to copy:", err);
	}
}

function toggleExpanded() {
	expanded = !expanded;
}
</script>

<div class="flex items-start gap-3 p-3 hover:bg-muted/30 transition-colors group">
	<div class="flex-shrink-0 w-16 text-sm text-muted-foreground font-mono text-right select-none opacity-50 group-hover:opacity-100 transition-opacity pt-1">
		#{line.id}
	</div>

	<div class="flex-1 min-w-0">
		<div class="flex items-center gap-2 mb-1">
			<Button
				onclick={toggleExpanded}
				variant="ghost"
				size="sm"
				class="h-6 w-6 p-0"
			>
				{#if expanded}
					<ChevronDown class="w-3 h-3" />
				{:else}
					<ChevronRight class="w-3 h-3" />
				{/if}
			</Button>

			<Button
				onclick={copyToClipboard}
				variant="ghost"
				size="sm"
				class="h-6 px-2 text-muted-foreground hover:text-foreground"
			>
				{#if copied}
					<span class="text-xs text-emerald-500 font-medium">Copied</span>
				{:else}
					<Copy class="w-3 h-3" />
				{/if}
			</Button>
		</div>

		{#if expanded}
			<div class="bg-muted/30 rounded-lg p-3 border border-border/50 overflow-hidden">
				<JsonTree value={line.parsed} />
			</div>
		{:else}
			<div class="font-mono text-sm truncate opacity-80 cursor-pointer" onclick={toggleExpanded}>
				{line.content}
			</div>
		{/if}
	</div>
</div>