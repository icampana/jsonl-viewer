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
import UrlDialog from "$lib/components/UrlDialog.svelte";

let isDragging = false;
let showUrlDialog = false;
let currentSearchId = 0;

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

async function loadFile(path: string) {
    fileStore.reset();
    searchStore.reset();
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
            path: path,
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

function handleUrlLoad(event: CustomEvent<string>) {
    loadFile(event.detail);
}

onMount(() => {
    let unlisteners: (() => void)[] = [];

    const setupListeners = async () => {
        unlisteners.push(await listen("menu:open-file", () => openFile()));
        unlisteners.push(await listen("menu:open-url", () => showUrlDialog = true));
        unlisteners.push(await listen("menu:export-file", () => handleExport()));
        unlisteners.push(await listen("menu:close-file", () => fileStore.reset()));

        // Drag and Drop listeners (Tauri v2)
        // We listen to multiple variations just to be safe, but drag-drop is the v2 standard

        unlisteners.push(await listen("tauri://drag-enter", () => {
             isDragging = true;
        }));

        unlisteners.push(await listen("tauri://drag-leave", () => {
            isDragging = false;
        }));

        unlisteners.push(await listen("tauri://drag-drop", (event) => {
            console.log("File dropped (drag-drop):", event);
            isDragging = false;

            try {
                // v2 payload: { paths: string[], position: {x, y} }
                const payload = event.payload as { paths: string[] };
                const paths = payload.paths || [];

                if (paths.length > 0) {
                    const path = paths[0];
                    const lowerPath = path.toLowerCase();
                    if (lowerPath.endsWith(".json") || lowerPath.endsWith(".jsonl") || lowerPath.endsWith(".ndjson")) {
                        loadFile(path);
                    } else {
                        showError("Invalid File", "Only .json, .jsonl, and .ndjson files are supported.");
                    }
                }
            } catch (e) {
                console.error("Error handling drag-drop:", e);
                isDragging = false;
            }
        }));
    };

    setupListeners();

    return () => {
        unlisteners.forEach(unlisten => unlisten());
    };
});

async function handleSearch() {
    if (!$fileStore.metadata) return;

    currentSearchId++;
    const searchId = currentSearchId;

    searchStore.setSearching(true);
    searchStore.setResults([]); // Clear previous results
    searchStore.setError(null);

    const channel = new Channel<SearchResult[]>();

    channel.onmessage = (chunk) => {
        if (searchId === currentSearchId) {
            searchStore.addResults(chunk);
        }
    };

    try {
        const stats = await invoke("search_in_file", {
            path: $fileStore.metadata.path,
            query: $searchStore.query,
            fileFormat: $fileStore.metadata.format,
            channel,
        });

        if (searchId === currentSearchId) {
            searchStore.setStats(stats as SearchStats);
            searchStore.setSearching(false);
        }
    } catch (error) {
        if (searchId === currentSearchId) {
            console.error("Search failed:", error);
            searchStore.setError(error as string);
            searchStore.setSearching(false);
        }
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
            loadFile(selected);
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

<div class="flex flex-col h-screen bg-background text-foreground overflow-hidden relative">
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

    {#if showUrlDialog}
        <UrlDialog
            on:close={() => showUrlDialog = false}
            on:load={handleUrlLoad}
        />
    {/if}

    <!-- Drag Overlay -->
    {#if isDragging}
        <div class="absolute inset-0 z-50 bg-background/80 backdrop-blur-sm flex items-center justify-center border-4 border-primary border-dashed m-4 rounded-xl pointer-events-none">
            <div class="text-center">
                <h2 class="text-2xl font-bold mb-2">Drop it here!</h2>
                <p class="text-muted-foreground">Open your JSON file</p>
            </div>
        </div>
    {/if}
</div>