<script lang="ts" generics="T">
import { onMount } from "svelte";

let {
	items = [],
	itemHeight = 60,
	containerHeight = 400,
	overscan = 5,
	minWidth = '100%',
	scrollLeft = $bindable(0),
	children,
} = $props<{
	items: T[];
	itemHeight?: number;
	containerHeight?: number;
	overscan?: number;
	minWidth?: string | number;
	scrollLeft?: number;
	children: import("svelte").Snippet<[T, number]>;
}>();

let scrollTop = $state(0);
let containerElement: HTMLElement | undefined = $state();
let viewportHeight = $state(containerHeight);

let totalHeight = $derived(items.length * itemHeight);
let startIndex = $derived(
	Math.max(0, Math.floor(scrollTop / itemHeight) - overscan),
);
let endIndex = $derived(
	Math.min(
		items.length - 1,
		Math.floor((scrollTop + viewportHeight) / itemHeight) + overscan,
	),
);
let visibleItems = $derived(items.slice(startIndex, endIndex + 1));
let offsetY = $derived(startIndex * itemHeight);

onMount(() => {
	const updateViewportHeight = () => {
		if (containerElement) {
			viewportHeight = containerElement.clientHeight;
		}
	};

	updateViewportHeight();
	const resizeObserver = new ResizeObserver(updateViewportHeight);
	if (containerElement) {
		resizeObserver.observe(containerElement);
	}

	return () => {
		resizeObserver.disconnect();
	};
});

function handleScroll(event: Event) {
	const target = event.target as HTMLElement;
	scrollTop = target.scrollTop;
	scrollLeft = target.scrollLeft;
}

export function scrollToItem(index: number) {
	if (containerElement) {
		const targetScrollTop = index * itemHeight;
		containerElement.scrollTop = targetScrollTop;
	}
}
</script>

<div
	bind:this={containerElement}
	class="virtual-scroll-container h-full"
	style="height: 100%; overflow: auto;"
	onscroll={handleScroll}
>
	<div style="height: {totalHeight}px; min-width: {typeof minWidth === 'number' ? minWidth + 'px' : minWidth}; position: relative;">
		<div
			style="transform: translateY({offsetY}px); position: absolute; top: 0; left: 0; width: 100%;"
		>
			{#each visibleItems as item, index (startIndex + index)}
				<div
					style="height: {itemHeight}px;"
					class="virtual-item"
				>
					{@render children(item, startIndex + index)}
				</div>
			{/each}
		</div>
	</div>
</div>

<style>
	.virtual-scroll-container {
		scrollbar-width: thin;
		scrollbar-color: rgb(203 213 225) transparent;
	}

	.virtual-scroll-container::-webkit-scrollbar {
		width: 8px;
		height: 8px;
	}

	.virtual-scroll-container::-webkit-scrollbar-track {
		background: transparent;
	}

	.virtual-scroll-container::-webkit-scrollbar-thumb {
		background-color: rgb(203 213 225);
		border-radius: 4px;
	}

	.virtual-scroll-container::-webkit-scrollbar-thumb:hover {
		background-color: rgb(148 163 184);
	}

	.virtual-item {
		box-sizing: border-box;
	}
</style>