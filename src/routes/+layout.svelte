<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import '../app.css';
	import { themeStore } from '$lib/stores/themeStore';

	let { children } = $props();

	$effect(() => {
		if ($themeStore === 'dark') {
			document.documentElement.classList.add('dark');
		} else {
			document.documentElement.classList.remove('dark');
		}
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="min-h-screen bg-background text-foreground">
	{@render children()}
</div>
