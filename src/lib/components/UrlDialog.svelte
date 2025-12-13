<script lang="ts">
import { createEventDispatcher } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import { fade, scale } from "svelte/transition";

const dispatch = createEventDispatcher();

let url = $state("");
let loading = $state(false);
let error = $state<string | null>(null);
let inputElement = $state<HTMLInputElement>();

function close() {
    dispatch("close");
}

async function load() {
    if (!url) return;

    loading = true;
    error = null;

    try {
        const path = await invoke("download_url_to_temp", { url });
        dispatch("load", path);
        close();
    } catch (e) {
        error = e as string;
    } finally {
        loading = false;
    }
}

function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !loading) {
        load();
    } else if (e.key === "Escape" && !loading) {
        close();
    }
}

$effect(() => {
    if (inputElement) {
        inputElement.focus();
    }
});
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm" transition:fade={{ duration: 150 }}>
    <div class="bg-card text-card-foreground border rounded-xl shadow-lg w-full max-w-md p-6" transition:scale={{ duration: 150, start: 0.95 }}>
        <h2 class="text-xl font-semibold mb-4">Open from URL</h2>

        <div class="space-y-4">
            <div class="space-y-2">
                <label for="url-input" class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                    Enter URL of JSON/JSONL file
                </label>
                <input
                    id="url-input"
                    bind:this={inputElement}
                    bind:value={url}
                    onkeydown={handleKeydown}
                    placeholder="https://example.com/data.jsonl"
                    class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                    disabled={loading}
                />
            </div>

            {#if error}
                <div class="text-sm text-destructive bg-destructive/10 p-3 rounded-md">
                    {error}
                </div>
            {/if}

            <div class="flex justify-end gap-2 pt-2">
                <button
                    onclick={close}
                    class="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-10 px-4 py-2"
                    disabled={loading}
                >
                    Cancel
                </button>
                <button
                    onclick={load}
                    class="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2"
                    disabled={loading || !url}
                >
                    {#if loading}
                        Loading...
                    {:else}
                        Load
                    {/if}
                </button>
            </div>
        </div>
    </div>
</div>
