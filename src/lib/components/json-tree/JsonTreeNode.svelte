<script lang="ts">
import { ChevronRight, ChevronDown } from "lucide-svelte";
import JsonTreeNode from "./JsonTreeNode.svelte";

let {
	keyName = "",
	value,
	isLast = true,
	depth = 0,
} = $props<{
	keyName?: string;
	value: any;
	isLast?: boolean;
	depth?: number;
}>();

let expanded = $state(false); // Default collapsed for deep trees? User might want auto-expand root.

// Auto-expand root level or simple objects
$effect(() => {
	if (depth < 1) expanded = true;
});

function getType(val: any): string {
	if (val === null) return "null";
	if (Array.isArray(val)) return "array";
	return typeof val;
}

const type = $derived(getType(value));
const isObject = $derived(type === "object" || type === "array");
const isEmpty = $derived(isObject && Object.keys(value).length === 0);

function toggle() {
	if (isObject && !isEmpty) {
		expanded = !expanded;
	}
}
</script>

<div class="font-mono text-sm leading-6">
	<div class="flex items-start hover:bg-black/5 rounded px-1 -ml-1 transition-colors">
		<!-- Indentation spacer -->
		<!-- We don't use margin-left for perf, but flat structure with spacers if needed.
			 Actually, simple nested div padding is easier for small depth. -->

		<!-- Toggle Icon -->
		{#if isObject && !isEmpty}
			<button
				onclick={toggle}
				class="mt-1 mr-1 w-4 h-4 flex items-center justify-center text-muted-foreground hover:text-foreground transition-colors"
			>
				{#if expanded}
					<ChevronDown class="w-3 h-3" />
				{:else}
					<ChevronRight class="w-3 h-3" />
				{/if}
			</button>
		{:else}
			<span class="w-5 inline-block"></span>
		{/if}

		<!-- Key & Value -->
		<div class="flex-1 break-all">

			<!-- Key (if property of object) -->
			{#if keyName}
				<span class="text-sky-600 dark:text-sky-400 font-medium">"{keyName}"</span>:
			{/if}

			<!-- Value -->
			{#if isObject}
				{@const isArray = type === 'array'}
				<span class="text-muted-foreground">{isArray ? '[' : '{'}</span>

				{#if !expanded && !isEmpty}
					<button onclick={toggle} class="text-xs text-muted-foreground px-1 hover:bg-muted rounded">
						...
					</button>
					<span class="text-muted-foreground">{isArray ? ']' : '}'}</span>
					{#if !isLast},{/if}
				{:else if isEmpty}
					<span class="text-muted-foreground">{isArray ? ']' : '}'}</span>
					{#if !isLast},{/if}
				{/if}
			{:else}
				<!-- Primitives -->
				{#if type === 'string'}
					<span class="text-emerald-600 dark:text-emerald-400">"{value}"</span>
				{:else if type === 'number'}
					<span class="text-amber-600 dark:text-amber-400">{value}</span>
				{:else if type === 'boolean'}
					<span class="text-violet-600 dark:text-violet-400">{value}</span>
				{:else if type === 'null'}
					<span class="text-rose-600 dark:text-rose-400">null</span>
				{/if}
				{#if !isLast},{/if}
			{/if}
		</div>
	</div>

	<!-- Recursive Children -->
	{#if isObject && !isEmpty && expanded}
		<div class="pl-4 border-l border-border/40 ml-2">
			{#if type === 'array'}
				{#each value as item, i}
					<JsonTreeNode
						value={item}
						isLast={i === value.length - 1}
						depth={depth + 1}
					/>
				{/each}
			{:else}
				{#each Object.entries(value) as [k, v], i}
					<JsonTreeNode
						keyName={k}
						value={v}
						isLast={i === Object.entries(value).length - 1}
						depth={depth + 1}
					/>
				{/each}
			{/if}

			<!-- Closing Brace for expanded view -->
			<div class="pl-2">
				<span class="text-muted-foreground">{type === 'array' ? ']' : '}'}</span>
				{#if !isLast},{/if}
			</div>
		</div>
	{/if}
</div>
