<script lang="ts">
	export let size = 'md' as 'sm' | 'md' | 'lg';
	export let text = 'Loading...';

	const sizeClasses = {
		sm: 'w-4 h-4',
		md: 'w-6 h-6',
		lg: 'w-8 h-8'
	};
</script>

<div class="flex items-center justify-center gap-2">
	<div class="animate-spin rounded-full border-2 border-muted-foreground border-t-transparent {sizeClasses[size]}"></div>
	{#if text}
		<span class="text-sm text-muted-foreground">{text}</span>
	{/if}
</div>