<script lang="ts">
	import { fileStore } from '$lib/stores/fileStore';

	function formatFileSize(bytes: number): string {
		if (bytes === 0) return '0 Bytes';
		const k = 1024;
		const sizes = ['Bytes', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
	}
</script>

<footer class="flex items-center justify-between px-4 py-2 border-t border-border bg-card text-sm">
	<div class="flex items-center gap-4">
		{#if $fileStore.metadata}
			<span>
				{$fileStore.totalLines.toLocaleString()} lines
			</span>
			<span>•</span>
			<span>
				{formatFileSize($fileStore.fileSize)}
			</span>
			<span>•</span>
			<span class="text-muted-foreground">
				{$fileStore.format}
			</span>
		{:else}
			<span class="text-muted-foreground">No file loaded</span>
		{/if}
	</div>

	<div class="flex items-center gap-4">
		{#if $fileStore.metadata}
			{#if $fileStore.selectedLineId !== null}
				<span>
					Selected: #{$fileStore.selectedLineId}
				</span>
				<span>•</span>
			{/if}
			<span>
				{$fileStore.columns.length} columns
			</span>
		{/if}
	</div>
</footer>