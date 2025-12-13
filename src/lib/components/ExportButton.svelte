<script lang="ts">
import { Download, FileSpreadsheet, Loader2 } from "lucide-svelte";
import Button from "$lib/components/ui/button.svelte";
import { fileStore } from "$lib/stores/fileStore";
import type { ExportFormat } from "$lib/types";
import { save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { showSuccess, showError } from "$lib/stores/toastStore";

export let disabled = false;

const exportFormats: { value: ExportFormat; label: string; icon: any }[] = [
	{ value: "Csv", label: "CSV", icon: Download },
	{ value: "Excel", label: "Excel", icon: FileSpreadsheet },
];

let exportingFormat: ExportFormat | null = null;

async function exportFile(format: ExportFormat) {
	if (!$fileStore.metadata) return;

	try {
		const extension = format === "Excel" ? "xlsx" : "csv";
		const defaultPath = `${$fileStore.metadata.path.replace(/\.[^/.]+$/, "")}.${extension}`;

		const outputPath = await save({
			defaultPath,
			filters: [
				{
					name: format,
					extensions: format === "Excel" ? ["xlsx"] : [format.toLowerCase()],
				},
			],
		});

		if (outputPath) {
			exportingFormat = format;
			const exportFunction =
				format === "Excel" ? "export_to_excel" : "export_to_csv";

			const stats = await invoke(exportFunction, {
				path: $fileStore.metadata.path,
				filter: {},
				outputPath,
			});

			showSuccess(
				"Export completed",
				`Successfully exported ${(stats as any).lines_exported} lines to ${format}`,
			);
		}
	} catch (error) {
		console.error("Export failed:", error);
		showError("Export failed", error as string);
	} finally {
		exportingFormat = null;
	}
}
</script>

<div class="flex items-center gap-2">
	{#each exportFormats as format}
		{@const IconComponent = format.icon}
		<Button
			on:click={() => exportFile(format.value)}
			variant="outline"
			size="sm"
			disabled={disabled || exportingFormat !== null}
		>
			{#if exportingFormat === format.value}
				<Loader2 class="w-4 h-4 mr-2 animate-spin" />
				Exporting...
			{:else}
				<IconComponent class="w-4 h-4 mr-2" />
				Export {format.label}
			{/if}
		</Button>
	{/each}
</div>