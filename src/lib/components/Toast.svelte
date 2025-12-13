<script lang="ts">
	import { toasts, removeToast, type Toast } from '$lib/stores/toastStore';
	import { CheckCircle, XCircle, Info, X } from 'lucide-svelte';
	import { fly } from 'svelte/transition';

	const icons = {
		success: CheckCircle,
		error: XCircle,
		info: Info
	};

	const colors = {
		success: 'text-green-600 bg-green-50 border-green-200',
		error: 'text-red-600 bg-red-50 border-red-200',
		info: 'text-blue-600 bg-blue-50 border-blue-200'
	};
</script>

{#each $toasts as toast (toast.id)}
	{@const IconComponent = icons[toast.type]}
	<div
		class="fixed top-4 right-4 z-50 max-w-sm p-4 rounded-lg border shadow-lg flex items-start gap-3 {colors[toast.type]}"
		transition:fly={{ x: 300, duration: 300 }}
	>
		<svelte:component this={IconComponent} class="w-5 h-5 flex-shrink-0 mt-0.5" />
		
		<div class="flex-1 min-w-0">
			<h4 class="font-semibold text-sm">{toast.title}</h4>
			{#if toast.message}
				<p class="text-sm mt-1 opacity-90">{toast.message}</p>
			{/if}
		</div>
		
		<button
			on:click={() => removeToast(toast.id)}
			class="flex-shrink-0 p-1 rounded hover:bg-black/10 transition-colors"
		>
			<X class="w-4 h-4" />
		</button>
	</div>
{/each}