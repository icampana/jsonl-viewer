<script lang="ts">
import { open, save } from "@tauri-apps/plugin-dialog";
import { invoke, Channel } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { onMount } from "svelte";
import { fileStore } from "$lib/stores/fileStore";
import { searchStore } from "$lib/stores/searchStore";
import { showSuccess, showError } from "$lib/stores/toastStore";
import type {
	JsonLine,
	FileMetadata,
	SearchResult,
	SearchStats,
} from "$lib/types";
import Header from "$lib/components/Header.svelte";
import SearchBar from "$lib/components/SearchBar.svelte";
import FileViewer from "$lib/components/FileViewer.svelte";
import DetailPanel from "$lib/components/DetailPanel.svelte";
import StatusBar from "$lib/components/StatusBar.svelte";
import Toast from "$lib/components/Toast.svelte";

async function handleExport() {
	if (!$fileStore.metadata) return;

	try {
		const extension = "csv";
		const defaultPath = `${$fileStore.metadata.path.replace(/\.[^/.]+$/, "")}.${extension}`;

		const outputPath = await save({
			defaultPath,
			filters: [
				{
					name: "CSV",
					extensions: ["csv"],
				},
				{
					name: "Excel",
					extensions: ["xlsx"],
				},
			],
		});

		if (outputPath) {
			const isExcel = outputPath.toLowerCase().endsWith(".xlsx");
			const exportFunction = isExcel ? "export_to_excel" : "export_to_csv";
			const format = isExcel ? "Excel" : "CSV";

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
	}
}

onMount(async () => {
	const unlistenOpen = await listen("menu:open-file", () => openFile());
	const unlistenExport = await listen("menu:export-file", () => handleExport());
	const unlistenClose = await listen("menu:close-file", () => fileStore.reset());

	return () => {
		unlistenOpen();
		unlistenExport();
		unlistenClose();
	};
});

async function handleSearch() {
	if (!$fileStore.metadata) return;

	searchStore.setSearching(true);
	searchStore.setResults([]); // Clear previous results
	searchStore.setError(null);

	const channel = new Channel<SearchResult[]>();

	channel.onmessage = (chunk) => {
		searchStore.addResults(chunk);
	};

	try {
		const stats = await invoke("search_in_file", {
			path: $fileStore.metadata.path,
			query: $searchStore.query,
			channel,
		});

		searchStore.setStats(stats as SearchStats);
	} catch (error) {
		console.error("Search failed:", error);
		searchStore.setError(error as string);
	} finally {
		searchStore.setSearching(false);
	}
}

async function openFile() {
	try {
		const selected = await open({
			multiple: false,
			filters: [
				{
					name: "JSON Files",
					extensions: ["json", "jsonl", "ndjson"],
				},
			],
		});

		if (selected && typeof selected === "string") {
			fileStore.reset();
			fileStore.setLoading(true);

			const channel = new Channel<JsonLine[]>();
			let buffer: JsonLine[] = [];
			let lastFlush = Date.now();
			const FLUSH_INTERVAL = 100; // ms
			const MAX_BUFFER_SIZE = 5000;

			const flushBuffer = () => {
				if (buffer.length > 0) {
					fileStore.addLines(buffer);
					buffer = [];
					lastFlush = Date.now();
				}
			};

			channel.onmessage = (message) => {
				buffer.push(...message);

				const now = Date.now();
				if (
					buffer.length >= MAX_BUFFER_SIZE ||
					now - lastFlush > FLUSH_INTERVAL
				) {
					flushBuffer();
				}
			};

			try {
				const metadata = await invoke("parse_file_streaming", {
					path: selected,
					channel,
				});

				// Final flush
				flushBuffer();

				fileStore.setMetadata(metadata as FileMetadata);
			} catch (error) {
				console.error("Parse file error:", error);
				fileStore.setError(error as string);
			} finally {
				fileStore.setLoading(false);
			}
		}
	} catch (error) {
		console.error("Failed to open file:", error);
		fileStore.setError(`Failed to open file: ${error}`);
	}
}
</script>

<svelte:head>
	<title>JSON-L Viewer</title>
</svelte:head>

<div class="flex flex-col h-screen bg-background text-foreground overflow-hidden">
	<Header {openFile} />
	<SearchBar on:search={handleSearch} />

	<!-- Main Workspace -->
	<div class="flex-1 flex min-h-0">
		<div class="flex-1 flex flex-col min-w-0">
			<FileViewer />
		</div>

		<DetailPanel />
	</div>

	<StatusBar />
	<Toast />
</div>